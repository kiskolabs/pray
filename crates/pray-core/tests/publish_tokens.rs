#![cfg(feature = "auth")]

use pray_core::auth::{RegistryAuthStore, PUBLISH_SCOPE};
use pray_core::push_auth::authorize_distribution_push;
use pray_core::trust::EmailConfirmationPolicy;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn issue_resolve_and_authorize_publish_token() {
    let root = temporary_root("publish-token");
    let store = RegistryAuthStore::open(&root).expect("open");
    store
        .register_email("publisher@example.com", EmailConfirmationPolicy::Disabled)
        .expect("register");
    let issued = store
        .issue_publish_token("publisher@example.com", &[PUBLISH_SCOPE.to_string()])
        .expect("issue");
    assert!(issued.scopes.contains(&PUBLISH_SCOPE.to_string()));
    let resolved = store
        .resolve_publish_token(&issued.token)
        .expect("resolve")
        .expect("token");
    assert_eq!(resolved.email, "publisher@example.com");

    authorize_distribution_push(
        &root,
        "0.0.0.0",
        false,
        false,
        Some(&format!("Bearer {}", issued.token)),
    )
    .expect("token authorizes non-loopback push");

    store.revoke_publish_token(&issued.token).expect("revoke");
    assert!(store
        .resolve_publish_token(&issued.token)
        .expect("resolve after revoke")
        .is_none());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn missing_token_still_requires_open_push_on_public_bind() {
    let root = temporary_root("publish-token-missing");
    let error =
        authorize_distribution_push(&root, "0.0.0.0", false, false, None).expect_err("denied");
    assert!(error.to_string().contains("publish token") || error.to_string().contains("open-push"));
    let _ = fs::remove_dir_all(&root);
}

fn temporary_root(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "pray-{prefix}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("root");
    path
}
