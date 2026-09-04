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
manifest_hash = "sha256:0000000000000000000000000000000000000000000000000000000000000000"
source = []
target = []
managed_span = []
provisioned = []

[[package]]
name = "sample/base"
version = "1.4.3"
path = "{package_path}"
tree_hash = "sha256:1111111111111111111111111111111111111111111111111111111111111111"
artifact_hash = "sha256:2222222222222222222222222222222222222222222222222222222222222222"
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
fn clean_unused_requires_a_readable_parseable_lockfile() {
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

#[test]
fn clean_unused_rejects_incomplete_lockfile_before_deleting() {
    let repo = temporary_directory("pray-clean-unused-incomplete-lock");
    let cache = repo.join(".pray/cache/registry/sample/base/1.0.0/source");
    fs::create_dir_all(&cache).expect("cache");
    fs::write(cache.join("entry"), "cached").expect("cache entry");
    write_lockfile(&repo, "./.pray/cache/registry/sample/base/1.0.0/source");
    let lockfile_path = repo.join("Prayfile.lock");
    let lockfile = fs::read_to_string(&lockfile_path)
        .expect("lockfile")
        .replace(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "sha256:incomplete",
        );
    fs::write(lockfile_path, lockfile).expect("incomplete lockfile");

    assert!(!run_pray(&repo, &["clean", "--unused"]).status.success());
    assert!(cache.exists());
}

#[test]
fn clean_uses_the_selected_project_root() {
    let repo = temporary_directory("pray-clean-project-root");
    let nested = repo.join("nested");
    fs::create_dir_all(&nested).expect("nested directory");
    for relative in [".pray/cache", ".pray/vendor"] {
        fs::create_dir_all(repo.join(relative)).expect("local state");
    }
    fs::write(repo.join(".pray/state.json"), "{}").expect("state");

    let clean = run_pray(
        &nested,
        &["--path", repo.to_str().expect("utf-8 path"), "clean"],
    );
    assert!(
        clean.status.success(),
        "clean failed: {}",
        String::from_utf8_lossy(&clean.stderr)
    );
    assert!(!repo.join(".pray/cache").exists());
    assert!(!repo.join(".pray/vendor").exists());
    assert!(!repo.join(".pray/state.json").exists());
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
