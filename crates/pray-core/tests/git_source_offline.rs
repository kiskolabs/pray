#![allow(dead_code)]

#[path = "support/git_catalog.rs"]
mod git_catalog;

use git_catalog::{
    pin_cache, pinned_shallow_cache_fixture, unique_temp, write_git_catalog, write_lock_git_pin,
    write_prayfile,
};
use pray_core::embed::{resolve_project_with_options, ResolveOptions};
use pray_core::resolve::git_source_cache_directory;
use std::fs;

fn offline_options() -> ResolveOptions {
    ResolveOptions {
        offline: true,
        ..ResolveOptions::default()
    }
}

#[test]
fn offline_install_refuses_to_clone_a_missing_git_cache() {
    let root = unique_temp("pray-offline-git-clone");
    let _cache_env = pin_cache(&root);
    let origin = write_git_catalog(&root.join("origin"), "sample/base");
    let clone_url = format!("file://{}", origin.display());
    write_prayfile(
        &root,
        &format!(
            "source \"used\", \"git+{clone_url}\"\npray \"sample/base\", \"~> 1.0\", source: \"used\"\n"
        ),
    );

    let error = resolve_project_with_options(&root.join("Prayfile"), &offline_options())
        .expect_err("offline install must not clone");
    let message = error.to_string();
    assert!(
        message.contains("offline"),
        "offline error should name offline mode:\n{message}"
    );
    assert!(
        message.contains("git source"),
        "offline clone error should name the git source:\n{message}"
    );
    assert!(
        !git_source_cache_directory(&root, &clone_url)
            .join(".git")
            .is_dir(),
        "offline install must not create a project git cache"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn offline_install_does_not_read_a_file_origin_when_a_pin_is_missing() {
    let fixture = pinned_shallow_cache_fixture(false);
    assert!(
        fixture.origin.join("v1/packages").is_dir(),
        "fixture must keep the origin distribution tree"
    );
    let error = resolve_project_with_options(&fixture.root.join("Prayfile"), &offline_options())
        .expect_err("offline install must not use the origin worktree");
    let message = error.to_string();
    assert!(
        message.contains(&fixture.pinned),
        "missing-pin error should name the lock revision, not a later origin tree:\n{message}"
    );
    assert!(
        message.contains("offline"),
        "offline error should name offline mode:\n{message}"
    );
}

#[test]
fn install_does_not_use_a_file_origin_when_the_pin_cannot_be_fetched() {
    let fixture = pinned_shallow_cache_fixture(false);
    let missing = "0".repeat(40);
    write_lock_git_pin(&fixture.root, &fixture.clone_url, &missing);
    let error =
        resolve_project_with_options(&fixture.root.join("Prayfile"), &ResolveOptions::default())
            .expect_err("install must not read the origin worktree after a pin fetch fails");
    let message = error.to_string();
    assert!(
        message.contains(&missing)
            || message.contains("fetch")
            || message.contains("couldn't find")
            || message.contains("not our ref"),
        "pin fetch failure should surface, not resolve HEAD from the origin tree:\n{message}"
    );
}

#[test]
fn offline_install_seeds_from_global_cache_when_the_origin_is_gone() {
    let root = unique_temp("pray-offline-git-seed");
    let _cache_env = pin_cache(&root);
    let origin = write_git_catalog(&root.join("origin"), "sample/base");
    let clone_url = format!("file://{}", origin.display());
    write_prayfile(
        &root,
        &format!(
            "source \"used\", \"git+{clone_url}\"\npray \"sample/base\", \"~> 1.0\", source: \"used\"\n"
        ),
    );
    resolve_project_with_options(&root.join("Prayfile"), &ResolveOptions::default())
        .expect("online install should clone");
    fs::remove_dir_all(root.join(".pray/cache/git")).expect("remove project git cache");
    fs::rename(&origin, root.join("origin-away")).expect("hide origin");

    let project = resolve_project_with_options(&root.join("Prayfile"), &offline_options())
        .expect("offline install should seed from the global cache");
    assert_eq!(project.packages[0].spec.version, "1.0.0");
    let _ = fs::remove_dir_all(&root);
}
