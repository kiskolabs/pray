#[path = "install_network_support.rs"]
mod support;

use serde_json::Value;
use std::fs;

use support::{create_add_fixture, run_pray, temporary_directory};

#[test]
fn republish_preserves_timestamp_and_yank_until_package_content_changes() {
    let repo = temporary_directory("pray-publish-idempotency");
    let registry_root = temporary_directory("pray-publish-idempotency-root");
    create_add_fixture(&repo);

    let add = run_pray(&repo, &["add", "sample/base", "--path", "packages/base"]);
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    publish(&repo, &registry_root);
    let metadata_path = registry_root.join("v1/packages/sample/base.json");
    let mut metadata = read_metadata(&metadata_path);
    metadata["versions"][0]["published_at"] = Value::String("1234567890".to_string());
    metadata["versions"][0]["yanked"] = Value::Bool(true);
    fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&metadata).expect("serialize metadata"),
    )
    .expect("write metadata");

    publish(&repo, &registry_root);
    let unchanged = read_metadata(&metadata_path);
    assert_eq!(unchanged["versions"][0]["published_at"], 1_234_567_890_u64);
    assert_eq!(unchanged["versions"][0]["yanked"], true);
    publish(&repo, &registry_root);
    assert_eq!(read_metadata(&metadata_path), unchanged);

    let prayspec_path = repo.join("packages/base/sample-base.prayspec");
    let prayspec = fs::read_to_string(&prayspec_path).expect("read prayspec");
    fs::write(
        &prayspec_path,
        prayspec.replace("shared guidance", "revised guidance"),
    )
    .expect("change package specification");
    publish(&repo, &registry_root);
    let specification_changed = read_metadata(&metadata_path);
    assert_ne!(
        specification_changed["versions"][0]["artifact_hash"],
        unchanged["versions"][0]["artifact_hash"]
    );
    assert_ne!(
        specification_changed["versions"][0]["published_at"],
        1_234_567_890_u64
    );
    assert_eq!(specification_changed["versions"][0]["yanked"], true);

    fs::write(
        repo.join("packages/base/exports/testing-basics.md"),
        "Changed testing guidance\n",
    )
    .expect("change package content");
    publish(&repo, &registry_root);
    let changed = read_metadata(&metadata_path);
    assert_ne!(
        changed["versions"][0]["artifact_hash"],
        specification_changed["versions"][0]["artifact_hash"]
    );
}

fn publish(repo: &std::path::Path, registry_root: &std::path::Path) {
    let result = run_pray(
        repo,
        &[
            "publish",
            "--root",
            registry_root.to_str().expect("registry path"),
        ],
    );
    assert!(
        result.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn read_metadata(path: &std::path::Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("read metadata")).expect("parse metadata")
}
