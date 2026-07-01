#[path = "install_network_support.rs"]
mod support;

use serde_json::Value;
use std::fs;

use support::{
    create_add_fixture, find_free_port, run_pray, run_pray_login_passkey, signing_key_from_seed,
    spawn_server, ssh_public_key_text, temporary_directory, verify_email_registration,
    wait_for_server, write_private_key_file, write_registry_client_fixture,
};

#[test]
fn beta_e2e_publishes_consumes_and_syncs_across_workspaces() {
    let workspace = temporary_directory("pray-beta-e2e");
    let source_repo = workspace.join("source");
    let registry_root = workspace.join("registry");
    let consumer_repo = workspace.join("consumer");
    let mirror_root = workspace.join("mirror");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&registry_root).expect("registry workspace");
    fs::create_dir_all(&consumer_repo).expect("consumer workspace");
    fs::create_dir_all(&mirror_root).expect("mirror workspace");
    fs::create_dir_all(registry_root.join("v1")).expect("registry v1 workspace");
    fs::create_dir_all(mirror_root.join("v1")).expect("mirror v1 workspace");
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

    let port = find_free_port();
    let server_url = format!("http://127.0.0.1:{port}");
    write_registry_client_fixture(&consumer_repo, &server_url);
    fs::write(
        mirror_root.join("v1/peers.json"),
        format!(
            r#"[
                {{
                    "name": "upstream",
                    "url": "{server_url}",
                    "public": true
                }}
            ]"#
        ),
    )
    .expect("write peer list");

    let failed_install = run_pray(&consumer_repo, &["install"]);
    assert!(!failed_install.status.success());
    assert!(matches!(failed_install.status.code(), Some(1) | Some(3)));
    let failed_stderr = String::from_utf8_lossy(&failed_install.stderr);
    assert!(
        failed_stderr.contains("Connection refused")
            || failed_stderr.contains("timed out")
            || failed_stderr.contains("Network error")
            || failed_stderr.contains("resolution error")
    );
    assert!(!consumer_repo.join("Prayfile.lock").exists());
    assert!(!consumer_repo.join("INSTRUCTIONS.md").exists());

    let auth_store =
        pray_core::auth::RegistryAuthStore::open(&registry_root).expect("open auth store");
    let publisher_email = "beta-e2e-publisher@example.com";
    let publisher_key = signing_key_from_seed(71);
    let publisher_public_key = ssh_public_key_text(&publisher_key);
    verify_email_registration(&auth_store, publisher_email);
    auth_store
        .enroll_passkey(
            publisher_email,
            "beta-e2e-publisher-passkey",
            &publisher_public_key,
            Some("publisher workstation"),
        )
        .expect("enroll publisher passkey");

    let publisher_private_key_path = write_private_key_file(
        &source_repo,
        "beta-e2e-publisher-passkey.bin",
        &publisher_key,
    );
    let mut server = spawn_server(&registry_root, port);
    wait_for_server(port);

    run_pray_login_passkey(
        &source_repo,
        &server_url,
        publisher_email,
        "beta-e2e-publisher-passkey",
        &publisher_private_key_path,
    );

    let publish = run_pray(&source_repo, &["publish", "--server", &server_url]);
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );
    let published_metadata_text =
        fs::read_to_string(registry_root.join("v1/packages/sample/base.json"))
            .expect("published metadata");
    let published_metadata: Value =
        serde_json::from_str(&published_metadata_text).expect("published metadata json");
    assert_eq!(published_metadata["name"], "sample/base");
    assert!(registry_root
        .join("v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg")
        .is_file());

    let install = run_pray(&consumer_repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed after publish: {}",
        String::from_utf8_lossy(&install.stderr)
    );
    let rendered =
        fs::read_to_string(consumer_repo.join("INSTRUCTIONS.md")).expect("rendered instructions");
    assert!(rendered.contains("Testing guidance"));
    let lockfile = fs::read_to_string(consumer_repo.join("Prayfile.lock")).expect("lockfile");
    assert!(lockfile.contains("sample/base"));

    let sync = run_pray(
        &workspace,
        &["sync", "--root", mirror_root.to_str().expect("mirror path")],
    );
    assert!(
        sync.status.success(),
        "sync failed after publish: {}",
        String::from_utf8_lossy(&sync.stderr)
    );
    let stdout = String::from_utf8_lossy(&sync.stdout);
    assert!(stdout.contains("Synchronized 1 package(s) from 1 peer(s); learned 1 peer(s)"));

    let mirror_metadata_text = fs::read_to_string(mirror_root.join("v1/packages/sample/base.json"))
        .expect("mirror metadata");
    let mirror_metadata: Value =
        serde_json::from_str(&mirror_metadata_text).expect("mirror metadata json");
    assert_eq!(mirror_metadata["name"], "sample/base");
    assert_eq!(mirror_metadata["versions"][0]["version"], "1.4.3");
    assert!(mirror_root
        .join("v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg")
        .is_file());
    let upstream_artifact =
        fs::read(registry_root.join("v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg"))
            .expect("upstream artifact");
    let mirrored_artifact =
        fs::read(mirror_root.join("v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg"))
            .expect("mirrored artifact");
    assert_eq!(mirrored_artifact, upstream_artifact);

    let _ = server.kill();
    let _ = server.wait();
}
