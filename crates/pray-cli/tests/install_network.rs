#[path = "install_network_support.rs"]
mod support;

use pray_core::auth::RegistryAuthStore;
use serde_json::Value;

use support::{
    create_add_fixture, find_free_port, run_pray, run_pray_login_passkey, signing_key_from_seed,
    spawn_server, ssh_public_key_text, temporary_directory, verify_email_registration,
    wait_for_server, write_private_key_file, write_registry_client_fixture,
};

#[test]
fn install_recovers_after_distribution_point_restart_and_consumes_published_package() {
    let workspace = temporary_directory("pray-install-distribution-recovery");
    let source_repo = workspace.join("source");
    let registry_root = workspace.join("registry");
    let consumer_repo = workspace.join("consumer");
    std::fs::create_dir_all(&source_repo).expect("source workspace");
    std::fs::create_dir_all(&registry_root).expect("registry workspace");
    std::fs::create_dir_all(&consumer_repo).expect("consumer workspace");
    std::fs::create_dir_all(registry_root.join("v1")).expect("registry v1 workspace");
    std::fs::write(
        registry_root.join("v1/index.json"),
        r#"{
            "spec": "prayfile-distribution-1",
            "packages": []
        }"#,
    )
    .expect("write registry index");
    std::fs::write(
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

    let auth_store = RegistryAuthStore::open(&registry_root).expect("open auth store");
    let publisher_email = "install-distribution-publisher@example.com";
    let consumer_email = "install-distribution-consumer@example.com";
    let publisher_key = signing_key_from_seed(61);
    let consumer_key = signing_key_from_seed(62);
    let publisher_public_key = ssh_public_key_text(&publisher_key);
    let consumer_public_key = ssh_public_key_text(&consumer_key);

    verify_email_registration(&auth_store, publisher_email);
    verify_email_registration(&auth_store, consumer_email);
    auth_store
        .enroll_passkey(
            publisher_email,
            "install-distribution-publisher-passkey",
            &publisher_public_key,
            Some("publisher workstation"),
        )
        .expect("enroll publisher passkey");
    auth_store
        .enroll_passkey(
            consumer_email,
            "install-distribution-consumer-passkey",
            &consumer_public_key,
            Some("consumer workstation"),
        )
        .expect("enroll consumer passkey");

    let publisher_private_key_path = write_private_key_file(
        &source_repo,
        "install-distribution-publisher-passkey.bin",
        &publisher_key,
    );
    let consumer_private_key_path = write_private_key_file(
        &consumer_repo,
        "install-distribution-consumer-passkey.bin",
        &consumer_key,
    );

    let port = find_free_port();
    let server_url = format!("http://127.0.0.1:{port}");
    let mut server = spawn_server(&registry_root, port);
    wait_for_server(port);

    write_registry_client_fixture(&consumer_repo, &server_url);

    run_pray_login_passkey(
        &source_repo,
        &server_url,
        publisher_email,
        "install-distribution-publisher-passkey",
        &publisher_private_key_path,
    );
    run_pray_login_passkey(
        &consumer_repo,
        &server_url,
        consumer_email,
        "install-distribution-consumer-passkey",
        &consumer_private_key_path,
    );

    let published = run_pray(&source_repo, &["publish", "--server", &server_url]);
    assert!(
        published.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&published.stderr)
    );
    let published_metadata_text =
        std::fs::read_to_string(registry_root.join("v1/packages/sample/base.json"))
            .expect("published metadata");
    let published_metadata: Value =
        serde_json::from_str(&published_metadata_text).expect("published metadata json");
    assert_eq!(published_metadata["name"], "sample/base");
    assert!(registry_root
        .join("v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg")
        .is_file());

    let _ = server.kill();
    let _ = server.wait();

    let failed_install = run_pray(&consumer_repo, &["install"]);
    assert!(!failed_install.status.success());
    assert!(matches!(failed_install.status.code(), Some(1) | Some(3)));
    let failed_stderr = String::from_utf8_lossy(&failed_install.stderr);
    assert!(
        failed_stderr.contains("Network error")
            || failed_stderr.contains("Connection refused")
            || failed_stderr.contains("timed out")
            || failed_stderr.contains("resolution error")
            || failed_stderr.contains("No such file")
    );
    assert!(!consumer_repo.join("Prayfile.lock").exists());
    assert!(!consumer_repo.join("INSTRUCTIONS.md").exists());

    let mut server = spawn_server(&registry_root, port);
    wait_for_server(port);

    let recovered_install = run_pray(&consumer_repo, &["install"]);
    assert!(
        recovered_install.status.success(),
        "install failed after restart: {}",
        String::from_utf8_lossy(&recovered_install.stderr)
    );
    let rendered = std::fs::read_to_string(consumer_repo.join("INSTRUCTIONS.md"))
        .expect("rendered instructions");
    assert!(rendered.contains("Testing guidance"));
    let lockfile = std::fs::read_to_string(consumer_repo.join("Prayfile.lock")).expect("lockfile");
    assert!(lockfile.contains("sample/base"));

    let _ = server.kill();
    let _ = server.wait();
}
