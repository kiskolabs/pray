use super::*;
use pray_core::auth::RegistryAuthStore;
use pray_core::trust::EmailConfirmationPolicy;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

#[test]
fn registration_delivers_code_outside_http_body() {
    let root = temporary_root("register");
    write_trust(&root, true, true);
    let response = auth_register_response(&root, br#"{"email":"alice@example.com"}"#)
        .expect("registration response");
    let body: serde_json::Value = serde_json::from_slice(&response.body).expect("json");
    let delivered = fs::read_to_string(root.join(".pray/verification-deliveries.jsonl"))
        .expect("delivery file");

    assert_eq!(response.status, 201);
    assert_eq!(body.get("verification_code"), None);
    assert!(delivered.contains("\"email\":\"alice@example.com\""));
    assert!(delivered.contains("\"code\":\""));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(root.join(".pray/verification-deliveries.jsonl"))
            .expect("delivery metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }
}

#[test]
fn email_session_and_public_key_enrollment_require_authenticated_workflow() {
    let root = temporary_root("protected");
    write_trust(&root, true, true);
    let session = auth_session_response(&root, br#"{"email":"alice@example.com"}"#)
        .expect("session response");
    let passkey = auth_passkey_enroll_response(
        &root,
        None,
        serde_json::to_vec(&json!({
            "email": "alice@example.com",
            "credential_id": "credential-1",
            "public_key": ssh_public_key(),
        }))
        .expect("passkey body")
        .as_slice(),
    )
    .expect("passkey response");
    let ssh_key = auth_ssh_key_enroll_response(
        &root,
        None,
        serde_json::to_vec(&json!({
            "email": "alice@example.com",
            "public_key": ssh_public_key(),
        }))
        .expect("ssh body")
        .as_slice(),
    )
    .expect("ssh response");

    assert_eq!(session.status, 403);
    assert_eq!(passkey.status, 403);
    assert_eq!(ssh_key.status, 403);
}

#[test]
fn verified_bearer_enrolls_passkey_and_ssh_key() {
    let root = temporary_root("enroll");
    write_trust(&root, true, true);
    auth_register_response(&root, br#"{"email":"alice@example.com"}"#).expect("register");
    let code = latest_delivery_code(&root);
    let verify = auth_verify_response(
        &root,
        serde_json::to_vec(&json!({
            "email": "alice@example.com",
            "code": code,
        }))
        .expect("verify body")
        .as_slice(),
    )
    .expect("verify");
    let verify_body: serde_json::Value = serde_json::from_slice(&verify.body).expect("json");
    let token = verify_body["token"].as_str().expect("token");
    let authorization = format!("Bearer {token}");
    let public_key = ssh_public_key();

    let passkey = auth_passkey_enroll_response(
        &root,
        Some(&authorization),
        serde_json::to_vec(&json!({
            "email": "alice@example.com",
            "credential_id": "credential-1",
            "public_key": public_key,
        }))
        .expect("passkey body")
        .as_slice(),
    )
    .expect("passkey");
    let ssh_key = auth_ssh_key_enroll_response(
        &root,
        Some(&authorization),
        serde_json::to_vec(&json!({
            "email": "alice@example.com",
            "public_key": public_key,
        }))
        .expect("ssh body")
        .as_slice(),
    )
    .expect("ssh");
    let mismatch = auth_passkey_enroll_response(
        &root,
        Some(&authorization),
        serde_json::to_vec(&json!({
            "email": "mallory@example.com",
            "credential_id": "credential-2",
            "public_key": public_key,
        }))
        .expect("mismatch body")
        .as_slice(),
    )
    .expect("mismatch");

    assert_eq!(verify.status, 200);
    assert_eq!(verify_body["kind"], "email");
    assert_eq!(passkey.status, 200);
    assert_eq!(ssh_key.status, 200);
    assert_eq!(mismatch.status, 403);
}

#[test]
fn disabled_key_methods_reject_login_challenges() {
    let root = temporary_root("disabled");
    let store = RegistryAuthStore::open(&root).expect("auth store");
    store
        .register_email("alice@example.com", EmailConfirmationPolicy::Disabled)
        .expect("registration");
    store
        .enroll_passkey("alice@example.com", "credential-1", &ssh_public_key(), None)
        .expect("passkey");

    let response = auth_passkey_challenge_response(&root, br#"{"credential_id":"credential-1"}"#)
        .expect("challenge response");

    assert_eq!(response.status, 403);
}

fn latest_delivery_code(root: &std::path::Path) -> String {
    let text = fs::read_to_string(root.join(".pray/verification-deliveries.jsonl")).expect("file");
    let line = text.lines().last().expect("line");
    let value: serde_json::Value = serde_json::from_str(line).expect("jsonl");
    value["code"].as_str().expect("code").to_string()
}

fn write_trust(root: &std::path::Path, passkeys: bool, ssh_keys: bool) {
    fs::write(
        root.join("v1/trust.json"),
        serde_json::to_vec(&json!({
            "email_confirmation": "required",
            "passkeys_enabled": passkeys,
            "ssh_keys_enabled": ssh_keys,
            "ssh_agent_signing_enabled": true
        }))
        .expect("trust json"),
    )
    .expect("write trust");
}

fn temporary_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "pray-server-auth-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("v1")).expect("root");
    root
}

fn ssh_public_key() -> String {
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcH".to_string()
}
