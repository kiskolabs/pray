#![allow(dead_code)]

#[path = "support/git_catalog.rs"]
mod git_catalog;

use git_catalog::{
    git, git_head, git_succeeds, global_git_directory, pin_cache, unique_temp, write_git_catalog,
    write_lock_git_pin, write_prayfile,
};
use pray_core::embed::{resolve_project_with_options, ResolveOptions};
use pray_core::resolve::git_source_cache_directory;
use std::fs;

#[test]
fn install_materializes_a_git_free_catalog_tree() {
    let root = unique_temp("pray-git-free-catalog");
    let _cache_env = pin_cache(&root);
    let origin = write_git_catalog(&root.join("origin"), "sample/base");
    let clone_url = format!("file://{}", origin.display());
    write_prayfile(
        &root,
        &format!(
            "source \"used\", \"git+{clone_url}\"\npray \"sample/base\", \"~> 1.0\", source: \"used\"\n"
        ),
    );

    let project = resolve_project_with_options(&root.join("Prayfile"), &ResolveOptions::default())
        .expect("install");
    assert_eq!(project.packages[0].spec.version, "1.0.0");

    let catalog = git_source_cache_directory(&root, &clone_url);
    assert!(
        !catalog.join(".git").exists(),
        "project catalog must not keep a git object store"
    );
    assert!(
        catalog.join("v1/packages").is_dir() || catalog.join("prayers/v1/packages").is_dir(),
        "project catalog must contain package metadata"
    );
    assert!(
        catalog.join(".pray-revision").is_file(),
        "project catalog must record the materialized revision"
    );
    let db = global_git_directory(&root.join("cache"), &clone_url);
    assert!(
        db.join("HEAD").is_file(),
        "git objects must live in the global bare db"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn install_unshallows_when_a_lock_pin_is_missing_from_a_shallow_db() {
    let root = unique_temp("pray-git-unshallow");
    let _cache_env = pin_cache(&root);
    let origin = write_git_catalog(&root.join("origin"), "sample/base");
    fs::write(origin.join("middle.txt"), "pin\n").expect("middle");
    git(&origin, &["add", "-A"]);
    git(
        &origin,
        &[
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "-m",
            "middle",
        ],
    );
    let pinned = git_head(&origin);
    git(
        &origin,
        &["config", "uploadpack.allowReachableSHA1InWant", "true"],
    );
    git(&origin, &["config", "uploadpack.allowFilter", "true"]);
    fs::write(origin.join("later.txt"), "newer tip\n").expect("later");
    git(&origin, &["add", "-A"]);
    git(
        &origin,
        &[
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "-m",
            "later",
        ],
    );
    let clone_url = format!("file://{}", origin.display());
    let db = global_git_directory(&root.join("cache"), &clone_url);
    fs::create_dir_all(db.parent().expect("db parent")).expect("db parent");
    git(
        &root,
        &[
            "clone",
            "--bare",
            "--depth",
            "1",
            "--no-local",
            &clone_url,
            db.to_str().expect("db path"),
        ],
    );
    assert!(
        !git_succeeds(&db, &["cat-file", "-e", &pinned]),
        "fixture db must lack the pin"
    );
    write_prayfile(
        &root,
        &format!(
            "source \"used\", \"git+{clone_url}\"\npray \"sample/base\", \"~> 1.0\", source: \"used\"\n"
        ),
    );
    write_lock_git_pin(&root, &clone_url, &pinned);

    let project = resolve_project_with_options(&root.join("Prayfile"), &ResolveOptions::default())
        .expect("install should unshallow the pin");
    assert_eq!(
        project.source_revisions.get("used").map(String::as_str),
        Some(pinned.as_str())
    );
    assert!(
        git_succeeds(&db, &["cat-file", "-e", &format!("{pinned}^")]),
        "unshallow should keep the pin's parent, not only a depth-1 fetch of the pin"
    );
    let _ = fs::remove_dir_all(&root);
}
