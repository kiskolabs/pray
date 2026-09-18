#[path = "install_network_support.rs"]
mod support;

use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

use support::{create_add_fixture, temporary_directory};

/// A second publisher re-running publish over unchanged packages must leave the catalog
/// alone. Rewriting `signer` and `signature` there restates authorship of bytes nobody
/// rebuilt, and in a shared distribution repository it buries the real release in churn.
#[test]
fn republish_by_another_signer_leaves_unchanged_versions_alone() {
    let repo = temporary_directory("pray-publish-cross-signer");
    let registry_root = temporary_directory("pray-publish-cross-signer-root");
    create_add_fixture(&repo);

    let add = publish_as(
        &repo,
        &["add", "sample/base", "--path", "packages/base"],
        "amkisko",
    );
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    publish(&repo, &registry_root, "amkisko");
    let metadata_path = registry_root.join("v1/packages/sample/base.json");
    let first = read_metadata(&metadata_path);
    assert_eq!(first["versions"][0]["signer"], "amkisko");

    publish(&repo, &registry_root, "vesan");
    let second = read_metadata(&metadata_path);
    assert_eq!(
        second, first,
        "a different signer rewrote a version whose artifact, tree, and prayspec were unchanged"
    );

    // The row is only preserved while the content matches; a real change still republishes
    // under whoever is publishing now.
    fs::write(
        repo.join("packages/base/exports/testing-basics.md"),
        "Changed testing guidance\n",
    )
    .expect("change package content");
    publish(&repo, &registry_root, "vesan");
    let changed = read_metadata(&metadata_path);
    assert_eq!(changed["versions"][0]["signer"], "vesan");
    assert_ne!(
        changed["versions"][0]["artifact_hash"],
        first["versions"][0]["artifact_hash"]
    );
}

fn publish(repo: &Path, registry_root: &Path, signer: &str) {
    let result = publish_as(
        repo,
        &[
            "publish",
            "--root",
            registry_root.to_str().expect("registry path"),
        ],
        signer,
    );
    assert!(
        result.status.success(),
        "publish as {signer} failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn publish_as(repo: &Path, arguments: &[&str], signer: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_pray"))
        .args(arguments)
        .current_dir(repo)
        .env("PRAY_HOME", repo.join(".pray-user"))
        .env("PRAY_SIGNER", signer)
        .env_remove("PRAY_SSH_USER_FINGERPRINT")
        .env_remove("SSH_USER_FINGERPRINT")
        .env_remove("PRAY_SSH_PUBLISHER")
        .env_remove("PRAY_SESSION_TOKEN")
        .output()
        .expect("run pray command")
}

fn read_metadata(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("read metadata")).expect("parse metadata")
}
