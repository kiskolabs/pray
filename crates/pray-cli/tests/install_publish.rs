#[path = "install_network_support.rs"]
mod support;

use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use support::{
    create_add_fixture, find_free_port, run_pray, run_pray_login_passkey, signing_key_from_seed,
    spawn_server, ssh_public_key_text, temporary_directory, verify_email_registration,
    wait_for_server, write_private_key_file,
};

#[test]
fn publish_writes_torrent_manifest_sidecar_for_registry_artifacts() {
    let repo = temporary_directory("pray-publish-torrent");
    let registry_root = temporary_directory("pray-publish-torrent-root");
    create_add_fixture(&repo);

    let add = run_pray(&repo, &["add", "sample/base", "--path", "packages/base"]);
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let publish = run_pray(
        &repo,
        &[
            "publish",
            "--root",
            registry_root.to_str().expect("registry path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    let artifact_path =
        registry_root.join("v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg");
    assert!(artifact_path.is_file());

    let manifest_path = PathBuf::from(format!("{}.praytorrent.json", artifact_path.display()));
    assert!(manifest_path.is_file());

    let manifest_text = fs::read_to_string(&manifest_path).expect("torrent manifest");
    let manifest: Value = serde_json::from_str(&manifest_text).expect("torrent manifest json");
    assert_eq!(manifest["spec"], "pray-torrent-v1");
    assert_eq!(manifest["name"], "sample/base");
    assert_eq!(manifest["version"], "1.4.3");
    assert_eq!(
        manifest["artifact_url"],
        "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg"
    );
    assert!(manifest["artifact_hash"]
        .as_str()
        .expect("artifact hash")
        .starts_with("sha256:"));
    assert!(!manifest["pieces"].as_array().expect("pieces").is_empty());
    assert!(manifest["sources"]
        .as_array()
        .expect("sources")
        .contains(&Value::String(
            "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg".to_string()
        )));

    let package_metadata_path = registry_root.join("v1/packages/sample/base.json");
    let package_metadata_text =
        fs::read_to_string(&package_metadata_path).expect("package metadata");
    let package_metadata: Value =
        serde_json::from_str(&package_metadata_text).expect("package metadata json");
    let derived = &package_metadata["versions"][0]["derived_metadata"];
    assert!(derived["summary"]
        .as_str()
        .expect("summary")
        .contains("shared guidance"));
    assert!(derived["summary"]
        .as_str()
        .expect("summary")
        .contains("Testing guidance"));
    assert!(!derived["topics"].as_array().expect("topics").is_empty());
    assert!(!derived["embeddings"]
        .as_array()
        .expect("embeddings")
        .is_empty());
}

#[test]
fn publish_recovers_after_destination_root_is_repaired() {
    let workspace = temporary_directory("pray-publish-recovery");
    let source_repo = workspace.join("source");
    let first_root = workspace.join("registry-a");
    let repaired_root = workspace.join("registry-b");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&first_root).expect("first registry workspace");

    create_add_fixture(&source_repo);
    let add = run_pray(
        &source_repo,
        &["add", "sample/base", "--path", "packages/base"],
    );
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let broken_root = workspace.join("broken-root");
    fs::write(&broken_root, "not a directory\n").expect("broken destination root file");

    let failed_publish = run_pray(
        &source_repo,
        &[
            "publish",
            "--root",
            first_root.to_str().expect("first root path"),
            "--root",
            broken_root.to_str().expect("broken root path"),
        ],
    );
    assert!(!failed_publish.status.success());
    assert!(matches!(failed_publish.status.code(), Some(1) | Some(3)));
    let failed_stderr = String::from_utf8_lossy(&failed_publish.stderr);
    assert!(
        failed_stderr.contains("Not a directory")
            || failed_stderr.contains("not a directory")
            || failed_stderr.contains("No such file or directory")
    );

    let first_root_index = first_root.join("v1/index.json");
    assert!(first_root_index.exists());
    let first_root_index_text = fs::read_to_string(&first_root_index).expect("first root index");
    assert!(first_root_index_text.contains("sample/base"));

    fs::remove_file(&broken_root).expect("remove broken destination root file");
    fs::create_dir_all(&repaired_root).expect("repaired registry workspace");

    let recovered_publish = run_pray(
        &source_repo,
        &[
            "publish",
            "--root",
            first_root.to_str().expect("first root path"),
            "--root",
            repaired_root.to_str().expect("repaired root path"),
        ],
    );
    assert!(
        recovered_publish.status.success(),
        "publish failed after repair: {}",
        String::from_utf8_lossy(&recovered_publish.stderr)
    );

    for root in [&first_root, &repaired_root] {
        let index_text = fs::read_to_string(root.join("v1/index.json")).expect("registry index");
        assert!(index_text.contains("sample/base"));
        let metadata_text = fs::read_to_string(root.join("v1/packages/sample/base.json"))
            .expect("package metadata");
        let metadata: Value = serde_json::from_str(&metadata_text).expect("metadata json");
        assert_eq!(metadata["name"], "sample/base");
        assert_eq!(metadata["versions"][0]["version"], "1.4.3");
        assert!(root
            .join("v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg")
            .is_file());
    }
}

#[test]
fn publish_recovers_after_server_restart_and_uploads_over_http() {
    let workspace = temporary_directory("pray-publish-network-recovery");
    let source_repo = workspace.join("source");
    let registry_root = workspace.join("registry");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&registry_root).expect("registry workspace");
    fs::create_dir_all(registry_root.join("v1")).expect("registry v1 workspace");
    fs::write(
        registry_root.join("v1/index.json"),
        r#"{
            "spec": "prayfile-distribution-1",
            "packages": []
        }"#,
    )
    .expect("write registry index");
    fs::write(
        registry_root.join("v1/trust.json"),
        r#"{
            "email_confirmation": "required",
            "passkeys_enabled": true,
            "ssh_keys_enabled": true,
            "ssh_agent_signing_enabled": true
        }"#,
    )
    .expect("write trust settings");

    create_add_fixture(&source_repo);
    let add = run_pray(
        &source_repo,
        &["add", "sample/base", "--path", "packages/base"],
    );
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let auth_store =
        pray_core::auth::RegistryAuthStore::open(&registry_root).expect("open auth store");
    let publisher_email = "publish-network-recovery@example.com";
    let signing_key = signing_key_from_seed(51);
    let public_key = ssh_public_key_text(&signing_key);
    verify_email_registration(&auth_store, publisher_email);
    auth_store
        .enroll_passkey(
            publisher_email,
            "publish-network-recovery-passkey",
            &public_key,
            Some("publisher workstation"),
        )
        .expect("enroll passkey");

    let private_key_path = write_private_key_file(
        &source_repo,
        "publish-network-recovery-passkey.bin",
        &signing_key,
    );
    let port = find_free_port();
    let server_url = format!("http://127.0.0.1:{port}");
    let mut server = spawn_server(&registry_root, port);
    wait_for_server(port);

    run_pray_login_passkey(
        &source_repo,
        &server_url,
        publisher_email,
        "publish-network-recovery-passkey",
        &private_key_path,
    );

    let _ = server.kill();
    let _ = server.wait();

    let failed_publish = run_pray(&source_repo, &["publish", "--server", &server_url]);
    assert!(!failed_publish.status.success());
    let failed_stderr = String::from_utf8_lossy(&failed_publish.stderr);
    assert!(
        failed_stderr.contains("Network error")
            || failed_stderr.contains("Connection refused")
            || failed_stderr.contains("timed out")
            || failed_stderr.contains("No such file")
            || failed_stderr.contains("unknown publish flag")
    );
    assert!(!registry_root.join("v1/packages/sample/base.json").exists());

    let mut server = spawn_server(&registry_root, port);
    wait_for_server(port);

    let recovered_publish = run_pray(&source_repo, &["publish", "--server", &server_url]);
    assert!(
        recovered_publish.status.success(),
        "publish failed after restart: {}",
        String::from_utf8_lossy(&recovered_publish.stderr)
    );

    let metadata_text = fs::read_to_string(registry_root.join("v1/packages/sample/base.json"))
        .expect("package metadata");
    let metadata: Value = serde_json::from_str(&metadata_text).expect("metadata json");
    assert_eq!(metadata["name"], "sample/base");
    assert_eq!(metadata["versions"][0]["signer"], publisher_email);
    assert!(metadata["versions"][0]["derived_metadata"]["summary"]
        .as_str()
        .expect("summary")
        .contains("shared guidance"));
    assert!(registry_root
        .join("v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg")
        .is_file());

    let _ = server.kill();
    let _ = server.wait();
}
