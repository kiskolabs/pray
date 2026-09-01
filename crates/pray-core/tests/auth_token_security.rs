#![cfg(feature = "auth")]

use pray_core::auth::{AuthSessionKind, RegistryAuthStore, PUBLISH_SCOPE};
use pray_core::trust::EmailConfirmationPolicy;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn issued_tokens_are_unique_and_stored_as_hashes() {
    let root = temporary_root("random");
    let store = verified_store(&root);

    let first_session = store
        .issue_session("bob@example.com", AuthSessionKind::Email)
        .expect("first session");
    let second_session = store
        .issue_session("bob@example.com", AuthSessionKind::Email)
        .expect("second session");
    let first_publish = store
        .issue_publish_token("bob@example.com", &[PUBLISH_SCOPE.to_string()])
        .expect("first publish token");
    let second_publish = store
        .issue_publish_token("bob@example.com", &[PUBLISH_SCOPE.to_string()])
        .expect("second publish token");

    assert_ne!(first_session.token, second_session.token);
    assert_ne!(first_publish.token, second_publish.token);
    let connection = Connection::open(root.join(".pray/auth.db")).expect("database");
    let stored_session: String = connection
        .query_row("SELECT token FROM sessions LIMIT 1", [], |row| row.get(0))
        .expect("stored session");
    assert_ne!(stored_session, first_session.token);
    assert!(stored_session.starts_with("sha256:"));
}

#[test]
fn expired_sessions_and_verification_codes_are_rejected() {
    let root = temporary_root("expiry");
    let store = RegistryAuthStore::open(&root).expect("store");
    let registration = store
        .register_email("alice@example.com", EmailConfirmationPolicy::Required)
        .expect("registration");
    let code = registration.verification_code.expect("code");
    let connection = Connection::open(root.join(".pray/auth.db")).expect("database");
    connection
        .execute("UPDATE email_verification_codes SET created_at = 0", [])
        .expect("expire code");
    assert!(store.verify_email("alice@example.com", &code).is_err());

    let store = verified_store(&root);
    let session = store
        .issue_session("bob@example.com", AuthSessionKind::Email)
        .expect("session");
    connection
        .execute("UPDATE sessions SET created_at = 0", [])
        .expect("expire session");
    assert!(store
        .resolve_session(&session.token)
        .expect("resolve")
        .is_none());
}

#[test]
fn verification_codes_are_hashed_and_guessing_is_uniform() {
    let root = temporary_root("verify");
    let store = RegistryAuthStore::open(&root).expect("store");
    let registration = store
        .register_email("alice@example.com", EmailConfirmationPolicy::Required)
        .expect("registration");
    let code = registration.verification_code.expect("code");
    let connection = Connection::open(root.join(".pray/auth.db")).expect("database");
    let stored_code: String = connection
        .query_row(
            "SELECT code FROM email_verification_codes WHERE email = ?1",
            ["alice@example.com"],
            |row| row.get(0),
        )
        .expect("stored code");
    assert_ne!(stored_code, code);
    assert!(stored_code.starts_with("sha256:"));

    let missing = store
        .verify_email("missing@example.com", "guess")
        .expect_err("missing");
    let mismatch = store
        .verify_email("alice@example.com", "guess")
        .expect_err("mismatch");
    assert_eq!(missing.to_string(), mismatch.to_string());
    assert!(missing.to_string().contains("verification failed"));

    for _ in 0..5 {
        let _ = store.verify_email("alice@example.com", "guess");
    }
    assert!(store.verify_email("alice@example.com", &code).is_err());
}

#[cfg(unix)]
#[test]
fn auth_database_uses_owner_only_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let root = temporary_root("mode");
    let _store = RegistryAuthStore::open(&root).expect("store");
    let database_mode = fs::metadata(root.join(".pray/auth.db"))
        .expect("db")
        .permissions()
        .mode()
        & 0o777;
    let directory_mode = fs::metadata(root.join(".pray"))
        .expect("dir")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(database_mode, 0o600);
    assert_eq!(directory_mode, 0o700);
}

fn verified_store(root: &Path) -> RegistryAuthStore {
    let store = RegistryAuthStore::open(root).expect("store");
    store
        .register_email("bob@example.com", EmailConfirmationPolicy::Disabled)
        .expect("registration");
    store
}

fn temporary_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "pray-auth-token-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(&root).expect("root");
    root
}
