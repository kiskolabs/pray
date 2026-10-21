#![allow(dead_code)]

#[path = "support/git_catalog.rs"]
mod git_catalog;

use git_catalog::{
    git, git_head, global_git_directory, pin_cache, unique_temp, write_git_catalog, write_prayfile,
};
use pray_core::embed::{resolve_project_with_options, ResolveOptions};
use pray_core::resolve::{
    git_source_cache_directory, git_source_cache_directory_with_subdir,
    git_source_cached_repository,
};
use std::fs;
use std::path::Path;

#[test]
fn install_quarantines_legacy_full_clone_at_url_key() {
    let root = unique_temp("pray-git-legacy-coexist");
    let _cache_env = pin_cache(&root);
    let origin = write_git_catalog(&root.join("origin"), "sample/base");
    let clone_url = format!("file://{}", origin.display());

    let legacy = git_source_cache_directory(&root, &clone_url);
    fs::create_dir_all(&legacy).expect("legacy dir");
    git(&legacy, &["init", "-b", "main"]);
    git(&legacy, &["remote", "add", "origin", &clone_url]);
    fs::write(legacy.join("stale.txt"), "legacy full clone\n").expect("stale");
    git(&legacy, &["add", "-A"]);
    git(
        &legacy,
        &[
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "-m",
            "legacy",
        ],
    );
    assert_ne!(git_head(&legacy), git_head(&origin));

    write_prayfile(
        &root,
        &format!(
            "source \"used\", \"git+{clone_url}\"\n\
             pray \"sample/base\", \"~> 1.0\", source: \"used\"\n"
        ),
    );

    let project = resolve_project_with_options(&root.join("Prayfile"), &ResolveOptions::default())
        .expect("install with legacy clone present");
    assert_eq!(project.packages[0].spec.version, "1.0.0");

    let catalog = git_source_cache_directory(&root, &clone_url);
    assert!(
        catalog.join(".pray-revision").is_file(),
        "stub catalog must be written at the URL key"
    );
    assert!(
        !catalog.join(".git").exists(),
        "URL-key catalog must not remain a full clone"
    );
    assert!(
        !legacy_checkout_active(&root),
        "legacy full clone must be quarantined or replaced"
    );

    let cached = git_source_cached_repository(&root, &clone_url).expect("shared bare");
    let shared = global_git_directory(&root.join("cache"), &clone_url);
    assert_eq!(cached, shared);
    assert!(cached.join("HEAD").is_file());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn lookup_prefers_stub_marker_over_leftover_full_clone() {
    let root = unique_temp("pray-git-stub-over-legacy");
    let _cache_env = pin_cache(&root);
    let clone_url = "file://example-distribution";
    let shared = global_git_directory(&root.join("cache"), clone_url);
    fs::create_dir_all(&shared).expect("shared");
    git(&shared, &["init", "--bare", "-b", "main"]);

    let leftover = git_source_cache_directory_with_subdir(&root, clone_url, Some("left"));
    fs::create_dir_all(&leftover).expect("leftover");
    git(&leftover, &["init", "-b", "main"]);
    git(&leftover, &["remote", "add", "origin", clone_url]);

    let stub = git_source_cache_directory(&root, clone_url);
    fs::create_dir_all(&stub).expect("stub");
    fs::write(stub.join(".pray-git-dir"), shared.to_str().expect("utf8")).expect("marker");
    fs::write(stub.join(".pray-revision"), "abc").expect("revision");

    assert_eq!(
        git_source_cached_repository(&root, clone_url).as_deref(),
        Some(shared.as_path()),
        "stub .pray-git-dir must win over a leftover full clone"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn lookup_rejects_stub_marker_outside_expected_global_bare() {
    let root = unique_temp("pray-git-stub-wrong-bare");
    let _cache_env = pin_cache(&root);
    let clone_url = "file://example-distribution";
    let foreign = root.join("foreign-bare");
    fs::create_dir_all(&foreign).expect("foreign");
    git(&foreign, &["init", "--bare", "-b", "main"]);

    let stub = git_source_cache_directory(&root, clone_url);
    fs::create_dir_all(&stub).expect("stub");
    fs::write(stub.join(".pray-git-dir"), foreign.to_str().expect("utf8")).expect("marker");
    fs::write(stub.join(".pray-revision"), "abc").expect("revision");

    assert!(
        git_source_cached_repository(&root, clone_url).is_none(),
        "stub must not bind trust to a foreign git directory"
    );
    let _ = fs::remove_dir_all(&root);
}

fn legacy_checkout_active(root: &Path) -> bool {
    let cache_root = root.join(".pray/cache/git");
    let Ok(entries) = fs::read_dir(cache_root) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        !name.ends_with(".legacy-git") && path.join(".git").exists()
    })
}
