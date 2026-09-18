#[path = "install_support.rs"]
mod support;

use std::fs;
use support::{create_add_fixture, run_pray, temporary_directory};

fn publisher_repo(prefix: &str) -> std::path::PathBuf {
    let repo = temporary_directory(prefix);
    create_add_fixture(&repo);
    let add = run_pray(&repo, &["add", "sample/base", "--path", "packages/base"]);
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    let prayfile = fs::read_to_string(repo.join("Prayfile")).expect("prayfile");
    fs::write(
        repo.join("Prayfile"),
        format!("prayfile \"1\"\npublish \"prayers\", path: \"prayers\"\n{prayfile}"),
    )
    .expect("rewrite");
    repo
}

#[test]
fn publish_without_flags_fails_when_no_remote_is_declared() {
    let repo = temporary_directory("pray-publish-no-remote");
    create_add_fixture(&repo);
    let output = run_pray(&repo, &["publish"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--root PATH or --server URL"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn publish_uses_declared_path_remote() {
    let repo = publisher_repo("pray-publish-remote");
    let output = run_pray(&repo, &["publish"]);
    assert!(
        output.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(repo.join("prayers/v1/index.json").is_file());
    assert!(repo.join("prayers/v1/packages/sample/base.json").is_file());
}

#[test]
fn publish_dry_run_does_not_write() {
    let repo = publisher_repo("pray-publish-dry-run");
    let output = run_pray(&repo, &["publish", "--dry-run"]);
    assert!(
        output.status.success(),
        "dry-run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sample/base"));
    assert!(stdout.contains("prayers"));
    assert!(!repo.join("prayers/v1/index.json").exists());
}

#[test]
fn publish_rejects_undeclared_root() {
    let repo = publisher_repo("pray-publish-allowlist");
    let output = run_pray(&repo, &["publish", "--root", "other"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not a declared publish remote"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn yank_uses_declared_path_remote() {
    let repo = publisher_repo("pray-yank-to");
    let published = run_pray(&repo, &["publish"]);
    assert!(
        published.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&published.stderr)
    );
    let yank = run_pray(&repo, &["yank", "sample/base", "1.4.3", "--to", "prayers"]);
    assert!(
        yank.status.success(),
        "yank failed: {}",
        String::from_utf8_lossy(&yank.stderr)
    );
}

#[test]
fn repo_init_adds_a_publish_remote_to_an_existing_prayfile() {
    let repo = temporary_directory("pray-repo-init-publish");
    fs::write(repo.join("Prayfile"), "prayfile \"1\"\n").expect("prayfile");
    let output = run_pray(&repo, &["repo", "init"]);
    assert!(
        output.status.success(),
        "repo init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = fs::read_to_string(repo.join("Prayfile")).expect("prayfile");
    assert!(text.contains("publish \"prayers\", path: \"prayers\""));
}
