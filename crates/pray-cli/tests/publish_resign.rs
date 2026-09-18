#[path = "install_network_support.rs"]
mod support;

use serde_json::Value;
use std::fs;
use std::io::Read;
use std::path::Path;

use support::{
    create_add_fixture, run_pray_as, signing_key_from_seed, ssh_public_key_text,
    temporary_directory, write_private_key_file,
};

#[test]
fn explicit_resign_upgrades_and_rotates_a_version_without_losing_state() {
    let repo = temporary_directory("pray-publish-resign");
    let root = temporary_directory("pray-publish-resign-root");
    create_add_fixture(&repo);
    run(
        &repo,
        &["add", "sample/base", "--path", "packages/base"],
        "first",
    );
    let root_path = root.to_str().expect("root path");
    let metadata_path = root.join("v1/packages/sample/base.json");
    let artifact_path = root.join("v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg");

    run(&repo, &["publish", "--root", root_path], "first");
    run(
        &repo,
        &["yank", "sample/base", "1.4.3", "--root", root_path],
        "first",
    );
    let original = read_version(&metadata_path);
    assert!(original["signature"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));

    let first_key = signing_key_from_seed(11);
    let first_key_path = write_private_key_file(&repo, "first-key.bin", &first_key);
    let first_key_path = first_key_path.to_str().expect("first key path");
    run(
        &repo,
        &[
            "publish",
            "--root",
            root_path,
            "--resign",
            "--signing-key",
            first_key_path,
        ],
        "second",
    );
    let upgraded = read_version(&metadata_path);
    assert_eq!(upgraded["signer"], "second");
    assert_eq!(
        upgraded["signer_public_key"],
        ssh_public_key_text(&first_key)
    );
    assert!(upgraded["signature"]
        .as_str()
        .unwrap()
        .starts_with("ed25519:"));
    assert_eq!(upgraded["artifact_hash"], original["artifact_hash"]);
    assert_eq!(upgraded["published_at"], original["published_at"]);
    assert_eq!(upgraded["yanked"], true);

    let second_key = signing_key_from_seed(22);
    let second_key_path = write_private_key_file(&repo, "second-key.bin", &second_key);
    let second_key_path = second_key_path.to_str().expect("second key path");
    run(
        &repo,
        &[
            "publish",
            "--root",
            root_path,
            "--signing-key",
            second_key_path,
        ],
        "third",
    );
    assert_eq!(read_version(&metadata_path), upgraded);
    run(
        &repo,
        &[
            "publish",
            "--root",
            root_path,
            "--resign",
            "--signing-key",
            second_key_path,
        ],
        "third",
    );
    let rotated = read_version(&metadata_path);
    assert_eq!(rotated["signer"], "third");
    assert_eq!(
        rotated["signer_public_key"],
        ssh_public_key_text(&second_key)
    );
    assert_eq!(rotated["artifact_hash"], original["artifact_hash"]);
    assert_eq!(rotated["published_at"], original["published_at"]);
    assert_eq!(rotated["yanked"], true);
    assert_eq!(
        archive_export(&artifact_path),
        fs::read(repo.join("packages/base/exports/testing-basics.md")).expect("source export")
    );
}

#[test]
fn resign_uses_local_package_bytes_when_registry_artifact_is_self_consistent_but_false() {
    let repo = temporary_directory("pray-publish-resign-forged");
    let root = temporary_directory("pray-publish-resign-forged-root");
    let alternate_root = temporary_directory("pray-publish-resign-forged-alternate");
    create_add_fixture(&repo);
    run(
        &repo,
        &["add", "sample/base", "--path", "packages/base"],
        "first",
    );
    let root_path = root.to_str().expect("root path");
    run(&repo, &["publish", "--root", root_path], "first");
    let metadata_path = root.join("v1/packages/sample/base.json");
    let artifact_path = root.join("v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg");
    let source_path = repo.join("packages/base/exports/testing-basics.md");
    let source_bytes = fs::read(&source_path).expect("source export");

    fs::write(&source_path, "altered registry export\n").expect("alter source");
    run(
        &repo,
        &[
            "publish",
            "--root",
            alternate_root.to_str().expect("alternate root"),
        ],
        "first",
    );
    fs::write(&source_path, &source_bytes).expect("restore source");
    let altered_artifact =
        fs::read(alternate_root.join("v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg"))
            .expect("altered artifact");
    fs::write(&artifact_path, &altered_artifact).expect("replace stored artifact");
    let mut metadata: Value = serde_json::from_slice(&fs::read(&metadata_path).expect("metadata"))
        .expect("metadata json");
    let version = &mut metadata["versions"][0];
    let tree_hash = version["tree_hash"].as_str().expect("tree hash");
    let altered_hash = pray_core::hashing::sha256_prefixed(&altered_artifact);
    let altered_signature =
        pray_core::registry::registry_artifact_signature(&altered_artifact, tree_hash, "first");
    version["artifact_hash"] = Value::String(altered_hash.clone());
    version["signature"] = Value::String(altered_signature);
    fs::write(
        &metadata_path,
        serde_json::to_vec(&metadata).expect("metadata json"),
    )
    .expect("write false metadata");

    let key = signing_key_from_seed(33);
    let key_path = write_private_key_file(&repo, "recovery-key.bin", &key);
    run(
        &repo,
        &[
            "publish",
            "--root",
            root_path,
            "--resign",
            "--signing-key",
            key_path.to_str().expect("key path"),
        ],
        "second",
    );
    assert_ne!(read_version(&metadata_path)["artifact_hash"], altered_hash);
    assert_eq!(archive_export(&artifact_path), source_bytes);
}

#[test]
fn resign_without_key_refuses_before_changing_the_version() {
    let repo = temporary_directory("pray-publish-resign-no-key");
    let root = temporary_directory("pray-publish-resign-no-key-root");
    create_add_fixture(&repo);
    run(
        &repo,
        &["add", "sample/base", "--path", "packages/base"],
        "first",
    );
    let root_path = root.to_str().expect("root path");
    run(&repo, &["publish", "--root", root_path], "first");
    let metadata_path = root.join("v1/packages/sample/base.json");
    let original = fs::read(&metadata_path).expect("metadata");

    let result = run_pray_as(
        &repo,
        &["publish", "--root", root_path, "--resign"],
        "second",
    );
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("signing key"));
    assert_eq!(
        fs::read(metadata_path).expect("metadata after refusal"),
        original
    );
}

#[test]
fn resign_rejects_server_and_dry_run_destinations_before_writing() {
    let repo = temporary_directory("pray-publish-resign-options");
    let root = temporary_directory("pray-publish-resign-options-root");
    create_add_fixture(&repo);
    run(
        &repo,
        &["add", "sample/base", "--path", "packages/base"],
        "first",
    );
    let root_path = root.to_str().expect("root path");
    run(&repo, &["publish", "--root", root_path], "first");
    let metadata_path = root.join("v1/packages/sample/base.json");
    let original = fs::read(&metadata_path).expect("metadata");
    let key = signing_key_from_seed(44);
    let key_path = write_private_key_file(&repo, "options-key.bin", &key);
    let key_path = key_path.to_str().expect("key path");

    let server = run_pray_as(
        &repo,
        &[
            "publish",
            "--server",
            "https://example.invalid",
            "--resign",
            "--signing-key",
            key_path,
        ],
        "second",
    );
    assert!(!server.status.success());
    assert!(String::from_utf8_lossy(&server.stderr).contains("local root"));

    let dry_run = run_pray_as(
        &repo,
        &[
            "publish",
            "--root",
            root_path,
            "--resign",
            "--dry-run",
            "--signing-key",
            key_path,
        ],
        "second",
    );
    assert!(!dry_run.status.success());
    assert!(String::from_utf8_lossy(&dry_run.stderr).contains("--dry-run"));
    assert_eq!(
        fs::read(metadata_path).expect("metadata after refusal"),
        original
    );
}

fn run(repo: &Path, arguments: &[&str], signer: &str) {
    let output = run_pray_as(repo, arguments, signer);
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn read_version(path: &Path) -> Value {
    let metadata: Value =
        serde_json::from_slice(&fs::read(path).expect("metadata")).expect("metadata json");
    metadata["versions"][0].clone()
}

fn archive_export(path: &Path) -> Vec<u8> {
    let compressed = fs::File::open(path).expect("artifact");
    let decoder = zstd::stream::read::Decoder::new(compressed).expect("zstd decoder");
    let mut archive = tar::Archive::new(decoder);
    for entry in archive.entries().expect("archive entries") {
        let mut entry = entry.expect("archive entry");
        if entry.path().expect("entry path") == Path::new("exports/testing-basics.md") {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("export bytes");
            return bytes;
        }
    }
    panic!("export missing from archive")
}
