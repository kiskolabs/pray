#![cfg(feature = "auth")]

use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use pray_core::auth::{AuthSessionKind, RegistryAuthStore};
use pray_core::trust::EmailConfirmationPolicy;
use std::fs;
use std::path::PathBuf;

#[test]
fn registers_and_verifies_email_with_required_confirmation() {
    let root = temporary_directory("pray-auth-required");
    let store = RegistryAuthStore::open(&root).expect("open store");
    let registration = store
        .register_email("alice@example.com", EmailConfirmationPolicy::Required)
        .expect("register");
    assert!(!registration.verified);
    let code = registration
        .verification_code
        .as_ref()
        .expect("verification code");

    let verification = store
        .verify_email("alice@example.com", code)
        .expect("verify");
    assert!(verification.verified);
    assert!(store
        .user_verified("alice@example.com")
        .expect("user state"));
    let repeated = store
        .register_email("alice@example.com", EmailConfirmationPolicy::Required)
        .expect("repeat registration");
    assert!(repeated.verified);
    assert!(repeated.verification_code.is_none());
}

#[test]
fn registers_email_without_confirmation_when_disabled() {
    let root = temporary_directory("pray-auth-disabled");
    let store = RegistryAuthStore::open(&root).expect("open store");

    let registration = store
        .register_email("bob@example.com", EmailConfirmationPolicy::Disabled)
        .expect("register");
    assert!(registration.verified);
    assert!(registration.verification_code.is_none());
    assert!(store.user_verified("bob@example.com").expect("user state"));
}

#[test]
fn issues_session_for_optional_email_without_confirmation() {
    let root = temporary_directory("pray-auth-session");
    let store = RegistryAuthStore::open(&root).expect("open store");

    store
        .register_email("carol@example.com", EmailConfirmationPolicy::Optional)
        .expect("register");
    let session = store
        .issue_session("carol@example.com", AuthSessionKind::Email)
        .expect("session");
    assert_eq!(session.email, "carol@example.com");
    assert!(session.token.starts_with("sha256:"));
    assert_eq!(session.kind, AuthSessionKind::Email);
    assert_eq!(
        store
            .resolve_session(&session.token)
            .expect("resolve session")
            .map(|session| session.email),
        Some("carol@example.com".to_string())
    );
}

#[test]
fn enrolls_and_logs_in_with_passkey_and_ssh_key() {
    let root = temporary_directory("pray-auth-keys");
    let store = RegistryAuthStore::open(&root).expect("open store");

    let signing_key = signing_key_from_seed(17);
    let public_key = ssh_public_key_text(&signing_key);

    store
        .register_email("dave@example.com", EmailConfirmationPolicy::Optional)
        .expect("register");
    let passkey = store
        .enroll_passkey(
            "dave@example.com",
            "credential-1",
            &public_key,
            Some("laptop passkey"),
        )
        .expect("passkey enrollment");
    assert!(passkey.enrolled);
    let challenge = store
        .request_passkey_challenge("credential-1")
        .expect("passkey challenge");
    let signature = STANDARD.encode(signing_key.sign(challenge.challenge.as_bytes()).to_bytes());
    let passkey_login = store
        .respond_passkey_challenge("credential-1", &challenge.challenge_id, &signature)
        .expect("passkey login");
    assert_eq!(passkey_login.email, "dave@example.com");

    let ssh_key = store
        .enroll_ssh_key("dave@example.com", &public_key, Some("workstation"))
        .expect("ssh enrollment");
    assert!(ssh_key.enrolled);
    let ssh_challenge = store
        .request_ssh_key_challenge(&public_key)
        .expect("ssh challenge");
    let ssh_signature = STANDARD.encode(
        signing_key
            .sign(ssh_challenge.challenge.as_bytes())
            .to_bytes(),
    );
    let ssh_login = store
        .respond_ssh_key_challenge(&public_key, &ssh_challenge.challenge_id, &ssh_signature)
        .expect("ssh login");
    assert_eq!(ssh_login.email, "dave@example.com");
    store
        .register_email("erin@example.com", EmailConfirmationPolicy::Optional)
        .expect("second user");
    store
        .enroll_passkey("erin@example.com", "credential-1", &public_key, None)
        .expect_err("reassignment");
}

fn ssh_public_key_text(signing_key: &SigningKey) -> String {
    let mut blob = Vec::new();
    write_ssh_string(&mut blob, b"ssh-ed25519");
    write_ssh_string(&mut blob, &signing_key.verifying_key().to_bytes());
    format!("ssh-ed25519 {}", STANDARD.encode(blob))
}

fn write_ssh_string(buffer: &mut Vec<u8>, bytes: &[u8]) {
    buffer.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buffer.extend_from_slice(bytes);
}

fn signing_key_from_seed(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn temporary_directory(prefix: &str) -> PathBuf {
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
