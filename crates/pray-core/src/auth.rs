#[cfg(not(feature = "auth"))]
use crate::hashing::sha256_prefixed;
#[cfg(not(feature = "auth"))]
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthRegistrationRequest {
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthVerificationRequest {
    pub email: String,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSessionRequest {
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthPasskeyEnrollmentRequest {
    pub email: String,
    pub credential_id: String,
    pub public_key: String,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthPasskeyChallengeRequest {
    pub credential_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthPasskeyChallengeResponse {
    pub credential_id: String,
    pub challenge_id: String,
    pub challenge: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthPasskeyLoginRequest {
    pub credential_id: String,
    pub challenge_id: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSshKeyEnrollmentRequest {
    pub email: String,
    pub public_key: String,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSshKeyChallengeRequest {
    pub public_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSshKeyChallengeResponse {
    pub fingerprint: String,
    pub challenge_id: String,
    pub challenge: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSshKeyLoginRequest {
    pub public_key: String,
    pub challenge_id: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthRegistrationResponse {
    pub email: String,
    pub verified: bool,
    #[serde(default)]
    pub verification_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthVerificationResponse {
    pub email: String,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthSessionKind {
    Email,
    Passkey,
    SshKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSessionResponse {
    pub email: String,
    pub token: String,
    pub kind: AuthSessionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthPasskeyEnrollmentResponse {
    pub email: String,
    pub credential_id: String,
    pub enrolled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthPasskeyLoginResponse {
    pub email: String,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthChallengeResponse {
    pub challenge_id: String,
    pub challenge: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSshKeyEnrollmentResponse {
    pub email: String,
    pub fingerprint: String,
    pub enrolled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSshKeyLoginResponse {
    pub email: String,
    pub token: String,
}

#[cfg(feature = "auth")]
pub use crate::auth_store::{
    bearer_token_from_authorization, ssh_public_key_fingerprint_text, PublishTokenRecord,
    RegistryAuthStore, PUBLISH_SCOPE,
};

#[cfg(not(feature = "auth"))]
pub fn ssh_public_key_fingerprint_text(public_key: &str) -> PrayResult<String> {
    let mut fields = public_key.split_whitespace();
    let algorithm = fields.next().ok_or_else(|| PrayError::Parse {
        kind: "public key",
        message: "public key must include an algorithm".to_string(),
    })?;
    let encoded_key = fields.next().ok_or_else(|| PrayError::Parse {
        kind: "public key",
        message: "public key must include key bytes".to_string(),
    })?;
    if algorithm != "ssh-ed25519" {
        return Err(PrayError::Unsupported(format!(
            "unsupported public key algorithm: {algorithm}"
        )));
    }

    Ok(sha256_prefixed(format!("{algorithm} {encoded_key}").as_bytes()).to_ascii_uppercase())
}

#[cfg(all(test, feature = "auth"))]
mod tests {
    use super::*;
    use crate::trust::EmailConfirmationPolicy;
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use ed25519_dalek::SigningKey;
    use std::fs;
    use std::path::PathBuf;

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
        assert_eq!(code.len(), 6);
        assert!(!store
            .user_verified("alice@example.com")
            .expect("user state"));

        let verification = store
            .verify_email("alice@example.com", code)
            .expect("verify");
        assert!(verification.verified);
        assert!(store
            .user_verified("alice@example.com")
            .expect("user state"));
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
        let passkey_login = store
            .login_with_passkey("credential-1")
            .expect("passkey login");
        assert_eq!(passkey_login.email, "dave@example.com");

        let ssh_key = store
            .enroll_ssh_key("dave@example.com", &public_key, Some("workstation"))
            .expect("ssh enrollment");
        assert!(ssh_key.enrolled);
        let ssh_login = store.login_with_ssh_key(&public_key).expect("ssh login");
        assert_eq!(ssh_login.email, "dave@example.com");
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
}
