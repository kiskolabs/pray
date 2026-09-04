#[path = "install_support.rs"]
mod support;

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;

use support::{run_pray, temporary_directory};

fn write_lockfile(repo: &std::path::Path, package_path: &str) {
    fs::write(
        repo.join("Prayfile.lock"),
        format!(
            r#"prayfile_lock = "1"
spec = "0.1"
generated_by = "pray test"
manifest_hash = "sha256:test"
source = []
target = []
managed_span = []
provisioned = []

[[package]]
name = "sample/base"
version = "1.4.3"
path = "{package_path}"
tree_hash = "sha256:tree"
artifact_hash = "sha256:artifact"
artifact = "path:{package_path}"
exports = []
dependencies = []
"#
        ),
    )
    .expect("lockfile");
}

#[test]
fn clean_unused_retains_only_locked_registry_entries() {
    let repo = temporary_directory("pray-clean-unused");
    let locked = repo.join(".pray/cache/registry/sample/base/1.4.3/source");
    let stale_version = repo.join(".pray/cache/registry/sample/base/1.4.2/source");
    let stale_source = repo.join(".pray/cache/registry/sample/base/1.4.3/other");
    let legacy = repo.join(".pray/cache/registry/legacy/sample/base/1.4.3");
    let staging = repo.join(".pray/cache/registry/sample/base/1.4.3/source.staging");
    for path in [&locked, &stale_version, &stale_source, &legacy, &staging] {
        fs::create_dir_all(path).expect("cache path");
        fs::write(path.join("entry"), "cached").expect("cache entry");
    }
    fs::create_dir_all(repo.join(".pray/cache/git/repository")).expect("git cache");
    fs::create_dir_all(repo.join(".pray/vendor/sample-base")).expect("vendor");
    fs::write(repo.join(".pray/state.json"), "{}").expect("state");
    write_lockfile(&repo, "./.pray/cache/registry/sample/base/1.4.3/source");

    let clean = run_pray(&repo, &["clean", "--unused"]);
    assert!(
        clean.status.success(),
        "clean failed: {}",
        String::from_utf8_lossy(&clean.stderr)
    );
    assert!(locked.exists());
    assert!(!stale_version.exists());
    assert!(!stale_source.exists());
    assert!(!legacy.exists());
    assert!(!staging.exists());
    assert!(repo.join(".pray/cache/git/repository").exists());
    assert!(repo.join(".pray/vendor/sample-base").exists());
    assert!(repo.join(".pray/state.json").exists());
}

#[test]
fn clean_unused_validates_the_lockfile_before_deleting() {
    for contents in [None, Some("not valid = [")] {
        let repo = temporary_directory("pray-clean-unused-lock");
        let cache = repo.join(".pray/cache/registry/sample/base/1.0.0/source");
        fs::create_dir_all(&cache).expect("cache");
        fs::write(cache.join("entry"), "cached").expect("cache entry");
        if let Some(contents) = contents {
            fs::write(repo.join("Prayfile.lock"), contents).expect("lockfile");
        }

        assert!(!run_pray(&repo, &["clean", "--unused"]).status.success());
        assert!(cache.exists());
    }
}

#[cfg(unix)]
#[test]
fn clean_unused_does_not_follow_registry_symlinks() {
    let repo = temporary_directory("pray-clean-unused-symlink");
    let outside = temporary_directory("pray-clean-unused-outside");
    fs::write(outside.join("keep"), "outside").expect("outside file");
    fs::create_dir_all(repo.join(".pray/cache/registry")).expect("registry");
    symlink(&outside, repo.join(".pray/cache/registry/stale")).expect("symlink");
    write_lockfile(&repo, "./packages/base");

    assert!(run_pray(&repo, &["clean", "--unused"]).status.success());
    assert!(!repo.join(".pray/cache/registry/stale").exists());
    assert!(outside.join("keep").exists());
}

#[test]
fn clean_rejects_unrelated_arguments() {
    let repo = temporary_directory("pray-clean-arguments");
    assert!(!run_pray(&repo, &["clean", "--other"]).status.success());
    assert!(!run_pray(&repo, &["clean", "unused"]).status.success());
}
