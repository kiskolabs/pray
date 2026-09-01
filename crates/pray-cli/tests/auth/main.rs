use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::Signer;
use pray_core::auth::RegistryAuthStore;
use pray_core::trust::EmailConfirmationPolicy;
use pray_core::PrayError;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

mod support;

use support::{
    extract_json_string, fetch_http_post, fetch_http_post_with_authorization, latest_delivery_code,
    signing_key_from_seed, ssh_public_key_text,
};

#[test]
fn exercises_registration_session_passkey_and_ssh_key_over_http() {
    let workspace = temporary_directory("pray-auth-http");
    let registry_root = workspace.join("registry");
    fs::create_dir_all(registry_root.join("v1")).expect("registry dirs");
    fs::write(
        registry_root.join("v1/index.json"),
        r#"{
            "spec": "prayfile-distribution-1",
            "packages": []
        }"#,
    )
    .expect("write index");
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

    let port = find_free_port();
    let mut server = Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "serve",
            "--root",
            registry_root.to_str().expect("registry path"),
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn server");
    wait_for_server(port);

    let base_url = format!("http://127.0.0.1:{port}");
    let signing_key = signing_key_from_seed(7);
    let public_key = ssh_public_key_text(&signing_key);

    let register = fetch_http_post(
        &format!("{base_url}/v1/auth/register"),
        r#"{"email":"alice@example.com"}"#,
    );
    assert_eq!(register.status, 201);
    assert!(!register.body.contains("verification_code"));
    let code = latest_delivery_code(&registry_root, "alice@example.com");

    let verify = fetch_http_post(
        &format!("{base_url}/v1/auth/verify"),
        &format!(r#"{{"email":"alice@example.com","code":"{}"}}"#, code),
    );
    assert_eq!(verify.status, 200);
    assert!(verify.body.contains("\"verified\":true"));
    let token = extract_json_string(&verify.body, "token");
    assert!(token.starts_with("sha256:"));
    assert_eq!(extract_json_string(&verify.body, "kind"), "email");

    let session = fetch_http_post(
        &format!("{base_url}/v1/auth/session"),
        r#"{"email":"alice@example.com"}"#,
    );
    assert_eq!(session.status, 403);

    let unauthenticated = fetch_http_post(
        &format!("{base_url}/v1/auth/passkeys/enroll"),
        &format!(
            r#"{{"email":"alice@example.com","credential_id":"credential-1","public_key":"{}"}}"#,
            public_key
        ),
    );
    assert_eq!(unauthenticated.status, 403);

    let passkey_enroll = fetch_http_post_with_authorization(
        &format!("{base_url}/v1/auth/passkeys/enroll"),
        &format!(
            r#"{{"email":"alice@example.com","credential_id":"credential-1","public_key":"{}"}}"#,
            public_key
        ),
        Some(&token),
    );
    assert_eq!(passkey_enroll.status, 200);

    let passkey_challenge = fetch_http_post(
        &format!("{base_url}/v1/auth/passkeys/challenge"),
        r#"{"credential_id":"credential-1"}"#,
    );
    assert_eq!(passkey_challenge.status, 200);
    let passkey_challenge_id = extract_json_string(&passkey_challenge.body, "challenge_id");
    let passkey_challenge_value = extract_json_string(&passkey_challenge.body, "challenge");
    let passkey_signature = STANDARD.encode(
        signing_key
            .sign(passkey_challenge_value.as_bytes())
            .to_bytes(),
    );
    let passkey_login = fetch_http_post(
        &format!("{base_url}/v1/auth/passkeys/login"),
        &format!(
            r#"{{"credential_id":"credential-1","challenge_id":"{}","signature":"{}"}}"#,
            passkey_challenge_id, passkey_signature
        ),
    );
    assert_eq!(passkey_login.status, 200);
    assert_eq!(
        extract_json_string(&passkey_login.body, "email"),
        "alice@example.com"
    );
    assert!(extract_json_string(&passkey_login.body, "token").starts_with("sha256:"));

    let ssh_enroll = fetch_http_post_with_authorization(
        &format!("{base_url}/v1/auth/ssh-keys/enroll"),
        &format!(
            r#"{{"email":"alice@example.com","public_key":"{}"}}"#,
            public_key
        ),
        Some(&token),
    );
    assert_eq!(ssh_enroll.status, 200);

    let ssh_challenge = fetch_http_post(
        &format!("{base_url}/v1/auth/ssh-keys/challenge"),
        &format!(r#"{{"public_key":"{}"}}"#, public_key),
    );
    assert_eq!(ssh_challenge.status, 200);
    let ssh_challenge_id = extract_json_string(&ssh_challenge.body, "challenge_id");
    let ssh_challenge_value = extract_json_string(&ssh_challenge.body, "challenge");
    let ssh_signature =
        STANDARD.encode(signing_key.sign(ssh_challenge_value.as_bytes()).to_bytes());
    let ssh_login = fetch_http_post(
        &format!("{base_url}/v1/auth/ssh-keys/login"),
        &format!(
            r#"{{"public_key":"{}","challenge_id":"{}","signature":"{}"}}"#,
            public_key, ssh_challenge_id, ssh_signature
        ),
    );
    assert_eq!(ssh_login.status, 200);
    assert_eq!(
        extract_json_string(&ssh_login.body, "email"),
        "alice@example.com"
    );
    assert!(extract_json_string(&ssh_login.body, "token").starts_with("sha256:"));

    let _ = server.kill();
    let _ = server.wait();
}

#[test]
fn rejects_invalid_passkey_and_ssh_signatures() {
    let workspace = temporary_directory("pray-auth-invalid-signature");
    let store = RegistryAuthStore::open(&workspace).expect("open auth store");
    let signing_key = signing_key_from_seed(7);
    let wrong_key = signing_key_from_seed(8);
    let public_key = ssh_public_key_text(&signing_key);

    let registration = store
        .register_email("alice@example.com", EmailConfirmationPolicy::Disabled)
        .expect("register");
    assert!(registration.verified);

    store
        .enroll_passkey(
            "alice@example.com",
            "credential-1",
            &public_key,
            Some("laptop passkey"),
        )
        .expect("passkey enrollment");
    store
        .enroll_ssh_key("alice@example.com", &public_key, Some("workstation"))
        .expect("ssh enrollment");

    let passkey_challenge = store
        .request_passkey_challenge("credential-1")
        .expect("passkey challenge");
    let invalid_passkey_signature = STANDARD.encode(
        wrong_key
            .sign(passkey_challenge.challenge.as_bytes())
            .to_bytes(),
    );
    let passkey_error = store
        .respond_passkey_challenge(
            "credential-1",
            &passkey_challenge.challenge_id,
            &invalid_passkey_signature,
        )
        .expect_err("invalid passkey signature should fail");
    assert!(matches!(passkey_error, PrayError::Verify(_)));

    let ssh_challenge = store
        .request_ssh_key_challenge(&public_key)
        .expect("ssh challenge");
    let invalid_ssh_signature = STANDARD.encode(
        wrong_key
            .sign(ssh_challenge.challenge.as_bytes())
            .to_bytes(),
    );
    let ssh_error = store
        .respond_ssh_key_challenge(
            &public_key,
            &ssh_challenge.challenge_id,
            &invalid_ssh_signature,
        )
        .expect_err("invalid ssh signature should fail");
    assert!(matches!(ssh_error, PrayError::Verify(_)));
}

fn temporary_directory(prefix: &str) -> std::path::PathBuf {
    let unique = format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    );
    let path = std::env::temp_dir().join(unique);
    fs::create_dir_all(&path).expect("temporary directory");
    path
}

fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve port");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);
    port
}

fn wait_for_server(port: u16) {
    for _ in 0..50 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("server did not start on port {port}");
}
