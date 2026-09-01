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
