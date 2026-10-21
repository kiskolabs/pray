#![allow(dead_code)]

#[path = "support/git_catalog.rs"]
mod git_catalog;

use git_catalog::{git, global_git_directory, pinned_shallow_cache_fixture};
use pray_core::embed::{resolve_project_with_options, ResolveOptions};
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn install_repairs_a_broken_cache_without_touching_the_host_repository() {
    let fixture = pinned_shallow_cache_fixture(false);
    let host_remote = "https://example.com/host.git";
    fs::write(fixture.root.join("README.md"), "host\n").expect("host file");
    git(&fixture.root, &["init", "--template=", "-b", "main"]);
    git(&fixture.root, &["remote", "add", "origin", host_remote]);
    let db = global_git_directory(&fixture.root.join("cache"), &fixture.clone_url);
    fs::remove_dir_all(db.join("objects")).expect("break cache");

    resolve_project_with_options(&fixture.root.join("Prayfile"), &ResolveOptions::default())
        .expect("install should rebuild a broken shared bare cache");

    assert_eq!(
        origin_url(&fixture.root),
        host_remote,
        "host origin URL changed"
    );
    assert!(
        db.join("HEAD").is_file() && db.join("objects").is_dir(),
        "shared bare cache should exist after repair"
    );
}

fn origin_url(repository: &Path) -> String {
    let output = Command::new("git")
        .current_dir(repository)
        .args(["remote", "get-url", "origin"])
        .output()
        .expect("git");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
