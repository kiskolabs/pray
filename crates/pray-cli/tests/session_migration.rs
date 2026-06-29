mod session_migration_support;

use session_migration_support::*;

use pray_core::auth::RegistryAuthStore;
use serde_json::Value;
use std::fs;

#[test]
fn login_upgrades_legacy_single_session_document_and_publish_uses_latest_session() {
    let workspace = temporary_directory("pray-session-migration");
    let source_repo = workspace.join("source");
    let auth_root = workspace.join("auth");
    let registry_root = workspace.join("registry");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&auth_root).expect("auth workspace");
    fs::create_dir_all(&registry_root).expect("registry workspace");
    write_auth_registry_fixture(&auth_root);
    create_publish_fixture(&source_repo);

    let store = RegistryAuthStore::open(&auth_root).expect("open auth store");
    let upgraded_email = "upgraded@example.com";
    let signing_key = signing_key_from_seed(41);
    let public_key = ssh_public_key_text(&signing_key);

    verify_email_registration(&store, upgraded_email);
    store
        .enroll_passkey(
            upgraded_email,
            "upgraded-passkey",
            &public_key,
            Some("desktop"),
        )
        .expect("enroll passkey");

    let legacy_email = "legacy@example.com";
    let legacy_session_path = source_repo.join(".pray/session.json");
    fs::create_dir_all(legacy_session_path.parent().expect("session parent")).expect("session dir");
    fs::write(
        &legacy_session_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "server_url": "http://127.0.0.1:7440",
            "email": legacy_email,
            "token": "sha256:legacy-session-token",
            "kind": "passkey"
        }))
        .expect("serialize legacy session"),
    )
    .expect("write legacy session");

    let initial_publish = run_pray(
        &source_repo,
        &[
            "publish",
            "--root",
            registry_root.to_str().expect("registry path"),
        ],
    );
    assert!(
        initial_publish.status.success(),
        "initial publish failed: {}",
        String::from_utf8_lossy(&initial_publish.stderr)
    );
    let initial_metadata = published_metadata(&registry_root);
    assert_eq!(initial_metadata["versions"][0]["signer"], legacy_email);

    let port = find_free_port();
    let server_url = format!("http://127.0.0.1:{port}");
    let mut server = spawn_server(&auth_root, port);
    wait_for_server(port);

    let private_key_path =
        write_private_key_file(&source_repo, "upgraded-passkey.bin", &signing_key);
    let login = run_pray(
        &source_repo,
        &[
            "login",
            "--server",
            &server_url,
            "--email",
            upgraded_email,
            "--credential-id",
            "upgraded-passkey",
            "--passkey-key",
            private_key_path.to_str().expect("private key path"),
        ],
    );
    assert!(
        login.status.success(),
        "login failed after upgrade: {}",
        String::from_utf8_lossy(&login.stderr)
    );

    let session_text = fs::read_to_string(&legacy_session_path).expect("upgraded session file");
    let session_json: Value = serde_json::from_str(&session_text).expect("session json");
    let sessions = session_json
        .get("sessions")
        .and_then(Value::as_array)
        .expect("multiple sessions after upgrade");
    assert_eq!(sessions.len(), 2);
    assert!(session_emails(&session_json).contains(&legacy_email.to_string()));
    assert!(session_emails(&session_json).contains(&upgraded_email.to_string()));

    let upgraded_publish = run_pray(
        &source_repo,
        &[
            "publish",
            "--root",
            registry_root.to_str().expect("registry path"),
        ],
    );
    assert!(
        upgraded_publish.status.success(),
        "upgraded publish failed: {}",
        String::from_utf8_lossy(&upgraded_publish.stderr)
    );
    let upgraded_metadata = published_metadata(&registry_root);
    assert_eq!(upgraded_metadata["versions"][0]["signer"], upgraded_email);

    let _ = server.kill();
    let _ = server.wait();
}
