use crate::hashing::sha256_prefixed;
use crate::trust::EmailConfirmationPolicy;
use crate::{PrayError, PrayResult};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

#[derive(Debug, Clone)]
pub struct RegistryAuthStore {
    database_path: PathBuf,
}

#[derive(Debug, Clone)]
struct StoredChallenge {
    challenge: String,
}

impl RegistryAuthStore {
    pub fn open(root: &Path) -> PrayResult<Self> {
        let database_path = root.join(".pray/auth.db");
        if let Some(parent) = database_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let store = Self { database_path };
        store.initialize()?;
        Ok(store)
    }

    pub fn register_email(
        &self,
        email: &str,
        policy: EmailConfirmationPolicy,
    ) -> PrayResult<AuthRegistrationResponse> {
        validate_email(email)?;
        let connection = self.connection()?;
        let timestamp = current_unix_timestamp()?;
        let verified = matches!(policy, EmailConfirmationPolicy::Disabled);
        let verification_code = if verified {
            None
        } else {
            Some(generate_verification_code(email, timestamp))
        };
        let policy_text = email_confirmation_policy_text(policy);

        connection.execute(
            "INSERT INTO users (email, email_verified, email_confirmation_policy, created_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(email) DO UPDATE SET email_verified = excluded.email_verified, email_confirmation_policy = excluded.email_confirmation_policy",
            rusqlite::params![email, verified, policy_text, timestamp],
        )?;
        if let Some(code) = verification_code.as_ref() {
            connection.execute(
                "INSERT INTO email_verification_codes (email, code, created_at, verified_at)
                 VALUES (?1, ?2, ?3, NULL)
                 ON CONFLICT(email) DO UPDATE SET code = excluded.code, created_at = excluded.created_at, verified_at = NULL",
                rusqlite::params![email, code, timestamp],
            )?;
        }

        Ok(AuthRegistrationResponse {
            email: email.to_string(),
            verified,
            verification_code,
        })
    }

    pub fn verify_email(&self, email: &str, code: &str) -> PrayResult<AuthVerificationResponse> {
        validate_email(email)?;
        if code.trim().is_empty() {
            return Err(PrayError::Unsupported(
                "verification code cannot be empty".to_string(),
            ));
        }
        let connection = self.connection()?;
        let stored_code: Option<String> = connection
            .query_row(
                "SELECT code FROM email_verification_codes WHERE email = ?1",
                rusqlite::params![email],
                |row| row.get(0),
            )
            .optional()?;
        let Some(stored_code) = stored_code else {
            return Err(PrayError::Resolution(format!(
                "no verification code found for {email}"
            )));
        };
        if stored_code != code {
            return Err(PrayError::Resolution(format!(
                "verification code mismatch for {email}"
            )));
        }
        let timestamp = current_unix_timestamp()?;
        connection.execute(
            "UPDATE users SET email_verified = 1 WHERE email = ?1",
            rusqlite::params![email],
        )?;
        connection.execute(
            "UPDATE email_verification_codes SET verified_at = ?2 WHERE email = ?1",
            rusqlite::params![email, timestamp],
        )?;
        Ok(AuthVerificationResponse {
            email: email.to_string(),
            verified: true,
        })
    }

    pub fn user_verified(&self, email: &str) -> PrayResult<bool> {
        validate_email(email)?;
        let connection = self.connection()?;
        let verified: Option<bool> = connection
            .query_row(
                "SELECT email_verified FROM users WHERE email = ?1",
                rusqlite::params![email],
                |row| row.get(0),
            )
            .optional()?;
        Ok(verified.unwrap_or(false))
    }

    pub fn request_passkey_challenge(
        &self,
        credential_id: &str,
    ) -> PrayResult<AuthPasskeyChallengeResponse> {
        validate_identifier(credential_id, "credential id")?;
        let connection = self.connection()?;
        let email: String = connection.query_row(
            "SELECT email FROM passkeys WHERE credential_id = ?1",
            rusqlite::params![credential_id],
            |row| row.get(0),
        )?;
        let challenge = generate_auth_challenge("passkey", credential_id)?;
        let challenge_id = generate_challenge_id(&email, credential_id, "passkey", &challenge)?;
        store_challenge(&connection, &challenge_id, &email, &challenge, "passkey")?;
        Ok(AuthPasskeyChallengeResponse {
            credential_id: credential_id.to_string(),
            challenge_id,
            challenge,
        })
    }

    pub fn respond_passkey_challenge(
        &self,
        credential_id: &str,
        challenge_id: &str,
        signature: &str,
    ) -> PrayResult<AuthPasskeyLoginResponse> {
        validate_identifier(credential_id, "credential id")?;
        validate_identifier(challenge_id, "challenge id")?;
        validate_signature(signature)?;
        let connection = self.connection()?;
        let email: String = connection.query_row(
            "SELECT email FROM passkeys WHERE credential_id = ?1",
            rusqlite::params![credential_id],
            |row| row.get(0),
        )?;
        let challenge = load_challenge(&connection, challenge_id, &email, "passkey")?;
        let public_key = load_passkey_public_key(&connection, credential_id)?;
        verify_signature(&public_key, challenge.challenge.as_bytes(), signature)?;
        mark_challenge_used(&connection, challenge_id)?;
        let session = self.issue_session(&email, AuthSessionKind::Passkey)?;
        Ok(AuthPasskeyLoginResponse {
            email,
            token: session.token,
        })
    }

    pub fn request_ssh_key_challenge(
        &self,
        public_key: &str,
    ) -> PrayResult<AuthSshKeyChallengeResponse> {
        validate_public_key(public_key)?;
        let connection = self.connection()?;
        let (public_key, _) = parse_ssh_ed25519_public_key(public_key)?;
        let fingerprint = ssh_key_fingerprint(&public_key);
        let email: String = connection.query_row(
            "SELECT email FROM ssh_keys WHERE fingerprint = ?1",
            rusqlite::params![fingerprint],
            |row| row.get(0),
        )?;
        let challenge = generate_auth_challenge("ssh_key", &public_key)?;
        let challenge_id = generate_challenge_id(&email, &fingerprint, "ssh_key", &challenge)?;
        store_challenge(&connection, &challenge_id, &email, &challenge, "ssh_key")?;
        Ok(AuthSshKeyChallengeResponse {
            fingerprint,
            challenge_id,
            challenge,
        })
    }

    pub fn respond_ssh_key_challenge(
        &self,
        public_key: &str,
        challenge_id: &str,
        signature: &str,
    ) -> PrayResult<AuthSshKeyLoginResponse> {
        validate_public_key(public_key)?;
        validate_identifier(challenge_id, "challenge id")?;
        validate_signature(signature)?;
        let connection = self.connection()?;
        let (public_key, _) = parse_ssh_ed25519_public_key(public_key)?;
        let fingerprint = ssh_key_fingerprint(&public_key);
        let email: String = connection.query_row(
            "SELECT email FROM ssh_keys WHERE fingerprint = ?1",
            rusqlite::params![fingerprint],
            |row| row.get(0),
        )?;
        let challenge = load_challenge(&connection, challenge_id, &email, "ssh_key")?;
        verify_signature(&public_key, challenge.challenge.as_bytes(), signature)?;
        mark_challenge_used(&connection, challenge_id)?;
        let session = self.issue_session(&email, AuthSessionKind::SshKey)?;
        Ok(AuthSshKeyLoginResponse {
            email,
            token: session.token,
        })
    }

    pub fn issue_session(
        &self,
        email: &str,
        kind: AuthSessionKind,
    ) -> PrayResult<AuthSessionResponse> {
        validate_email(email)?;
        let connection = self.connection()?;
        let user: Option<(bool, String)> = connection
            .query_row(
                "SELECT email_verified, email_confirmation_policy FROM users WHERE email = ?1",
                rusqlite::params![email],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((verified, policy)) = user else {
            return Err(PrayError::Resolution(format!("unknown user: {email}")));
        };
        if !verified && policy != email_confirmation_policy_text(EmailConfirmationPolicy::Optional)
        {
            return Err(PrayError::Resolution(format!(
                "email confirmation required for {email}"
            )));
        }
        let timestamp = current_unix_timestamp()?;
        let token = generate_session_token(email, &kind, timestamp);
        connection.execute(
            "INSERT INTO sessions (token, email, kind, created_at, last_used_at)
             VALUES (?1, ?2, ?3, ?4, ?4)
             ON CONFLICT(token) DO UPDATE SET last_used_at = excluded.last_used_at",
            rusqlite::params![token, email, auth_session_kind_text(&kind), timestamp],
        )?;
        Ok(AuthSessionResponse {
            email: email.to_string(),
            token,
            kind,
        })
    }

    pub fn resolve_session(&self, token: &str) -> PrayResult<Option<AuthSessionResponse>> {
        if token.trim().is_empty() {
            return Ok(None);
        }
        let connection = self.connection()?;
        let session: Option<(String, String)> = connection
            .query_row(
                "SELECT email, kind FROM sessions WHERE token = ?1",
                rusqlite::params![token],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((email, kind_text)) = session else {
            return Ok(None);
        };
        let kind = parse_auth_session_kind(&kind_text)?;
        let timestamp = current_unix_timestamp()?;
        connection.execute(
            "UPDATE sessions SET last_used_at = ?2 WHERE token = ?1",
            rusqlite::params![token, timestamp],
        )?;
        Ok(Some(AuthSessionResponse {
            email,
            token: token.to_string(),
            kind,
        }))
    }

    pub fn enroll_passkey(
        &self,
        email: &str,
        credential_id: &str,
        public_key: &str,
        label: Option<&str>,
    ) -> PrayResult<AuthPasskeyEnrollmentResponse> {
        validate_email(email)?;
        validate_identifier(credential_id, "credential id")?;
        validate_public_key(public_key)?;
        let connection = self.connection()?;
        ensure_user_can_authenticate(&connection, email)?;
        let timestamp = current_unix_timestamp()?;
        connection.execute(
            "INSERT INTO passkeys (credential_id, email, public_key, label, created_at, last_used_at)
             VALUES (?1, ?2, ?3, ?4, ?5, NULL)
             ON CONFLICT(credential_id) DO UPDATE SET email = excluded.email, public_key = excluded.public_key, label = excluded.label",
            rusqlite::params![credential_id, email, public_key, label.unwrap_or(""), timestamp],
        )?;
        Ok(AuthPasskeyEnrollmentResponse {
            email: email.to_string(),
            credential_id: credential_id.to_string(),
            enrolled: true,
        })
    }

    pub fn login_with_passkey(&self, credential_id: &str) -> PrayResult<AuthPasskeyLoginResponse> {
        validate_identifier(credential_id, "credential id")?;
        let connection = self.connection()?;
        let email: Option<String> = connection
            .query_row(
                "SELECT email FROM passkeys WHERE credential_id = ?1",
                rusqlite::params![credential_id],
                |row| row.get(0),
            )
            .optional()?;
        let Some(email) = email else {
            return Err(PrayError::Resolution(format!(
                "unknown passkey credential: {credential_id}"
            )));
        };
        let session = self.issue_session(&email, AuthSessionKind::Passkey)?;
        connection.execute(
            "UPDATE passkeys SET last_used_at = ?2 WHERE credential_id = ?1",
            rusqlite::params![credential_id, current_unix_timestamp()?],
        )?;
        Ok(AuthPasskeyLoginResponse {
            email,
            token: session.token,
        })
    }

    pub fn enroll_ssh_key(
        &self,
        email: &str,
        public_key: &str,
        label: Option<&str>,
    ) -> PrayResult<AuthSshKeyEnrollmentResponse> {
        validate_email(email)?;
        validate_public_key(public_key)?;
        let connection = self.connection()?;
        ensure_user_can_authenticate(&connection, email)?;
        let (public_key, _) = parse_ssh_ed25519_public_key(public_key)?;
        let fingerprint = ssh_key_fingerprint(&public_key);
        let timestamp = current_unix_timestamp()?;
        connection.execute(
            "INSERT INTO ssh_keys (fingerprint, email, public_key, label, created_at, last_used_at)
             VALUES (?1, ?2, ?3, ?4, ?5, NULL)
             ON CONFLICT(fingerprint) DO UPDATE SET email = excluded.email, public_key = excluded.public_key, label = excluded.label",
            rusqlite::params![fingerprint, email, public_key, label.unwrap_or(""), timestamp],
        )?;
        Ok(AuthSshKeyEnrollmentResponse {
            email: email.to_string(),
            fingerprint,
            enrolled: true,
        })
    }

    pub fn login_with_ssh_key(&self, public_key: &str) -> PrayResult<AuthSshKeyLoginResponse> {
        validate_public_key(public_key)?;
        let connection = self.connection()?;
        let (public_key, _) = parse_ssh_ed25519_public_key(public_key)?;
        let fingerprint = ssh_key_fingerprint(&public_key);
        let email: Option<String> = connection
            .query_row(
                "SELECT email FROM ssh_keys WHERE fingerprint = ?1",
                rusqlite::params![fingerprint],
                |row| row.get(0),
            )
            .optional()?;
        let Some(email) = email else {
            return Err(PrayError::Resolution(format!(
                "unknown ssh key fingerprint: {fingerprint}"
            )));
        };
        let session = self.issue_session(&email, AuthSessionKind::SshKey)?;
        connection.execute(
            "UPDATE ssh_keys SET last_used_at = ?2 WHERE fingerprint = ?1",
            rusqlite::params![fingerprint, current_unix_timestamp()?],
        )?;
        Ok(AuthSshKeyLoginResponse {
            email,
            token: session.token,
        })
    }

    fn initialize(&self) -> PrayResult<()> {
        let connection = self.connection()?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (
                email TEXT PRIMARY KEY,
                email_verified INTEGER NOT NULL,
                email_confirmation_policy TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS email_verification_codes (
                email TEXT PRIMARY KEY,
                code TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                verified_at INTEGER
            );
            CREATE TABLE IF NOT EXISTS passkeys (
                credential_id TEXT PRIMARY KEY,
                email TEXT NOT NULL,
                public_key TEXT NOT NULL,
                label TEXT,
                created_at INTEGER NOT NULL,
                last_used_at INTEGER,
                FOREIGN KEY(email) REFERENCES users(email) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS ssh_keys (
                fingerprint TEXT PRIMARY KEY,
                email TEXT NOT NULL,
                public_key TEXT NOT NULL,
                label TEXT,
                created_at INTEGER NOT NULL,
                last_used_at INTEGER,
                FOREIGN KEY(email) REFERENCES users(email) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS sessions (
                token TEXT PRIMARY KEY,
                email TEXT NOT NULL,
                kind TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_used_at INTEGER,
                FOREIGN KEY(email) REFERENCES users(email) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS auth_challenges (
                challenge_id TEXT PRIMARY KEY,
                email TEXT NOT NULL,
                kind TEXT NOT NULL,
                challenge TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                used_at INTEGER,
                FOREIGN KEY(email) REFERENCES users(email) ON DELETE CASCADE
            );",
        )?;
        Ok(())
    }

    fn connection(&self) -> PrayResult<Connection> {
        let connection = Connection::open(&self.database_path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        Ok(connection)
    }
}

fn validate_email(email: &str) -> PrayResult<()> {
    let email = email.trim();
    if email.is_empty() || !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        return Err(PrayError::Unsupported(
            "email must be a non-empty address".to_string(),
        ));
    }
    Ok(())
}

fn validate_identifier(value: &str, label: &str) -> PrayResult<()> {
    if value.trim().is_empty() {
        return Err(PrayError::Unsupported(format!("{label} cannot be empty")));
    }
    Ok(())
}

fn validate_public_key(public_key: &str) -> PrayResult<()> {
    let public_key = public_key.trim();
    if public_key.is_empty() {
        return Err(PrayError::Unsupported(
            "public key cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn ensure_user_can_authenticate(connection: &Connection, email: &str) -> PrayResult<()> {
    let user: Option<(bool, String)> = connection
        .query_row(
            "SELECT email_verified, email_confirmation_policy FROM users WHERE email = ?1",
            rusqlite::params![email],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((verified, policy)) = user else {
        return Err(PrayError::Resolution(format!("unknown user: {email}")));
    };
    if verified || policy == email_confirmation_policy_text(EmailConfirmationPolicy::Optional) {
        Ok(())
    } else {
        Err(PrayError::Resolution(format!(
            "email confirmation required for {email}"
        )))
    }
}

fn email_confirmation_policy_text(policy: EmailConfirmationPolicy) -> &'static str {
    match policy {
        EmailConfirmationPolicy::Required => "required",
        EmailConfirmationPolicy::Optional => "optional",
        EmailConfirmationPolicy::Disabled => "disabled",
    }
}

fn auth_session_kind_text(kind: &AuthSessionKind) -> &'static str {
    match kind {
        AuthSessionKind::Email => "email",
        AuthSessionKind::Passkey => "passkey",
        AuthSessionKind::SshKey => "ssh_key",
    }
}

fn parse_auth_session_kind(kind: &str) -> PrayResult<AuthSessionKind> {
    match kind {
        "email" => Ok(AuthSessionKind::Email),
        "passkey" => Ok(AuthSessionKind::Passkey),
        "ssh_key" => Ok(AuthSessionKind::SshKey),
        other => Err(PrayError::Resolution(format!(
            "unknown auth session kind: {other}"
        ))),
    }
}

fn current_unix_timestamp() -> PrayResult<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PrayError::Resolution(error.to_string()))
        .map(|duration| duration.as_secs())
}

fn generate_auth_challenge(kind: &str, subject: &str) -> PrayResult<String> {
    let timestamp = current_unix_timestamp()?;
    Ok(sha256_prefixed(
        format!("{kind}\0{subject}\0{timestamp}").as_bytes(),
    ))
}

fn generate_challenge_id(
    email: &str,
    subject: &str,
    kind: &str,
    challenge: &str,
) -> PrayResult<String> {
    Ok(sha256_prefixed(
        format!("challenge\0{email}\0{subject}\0{kind}\0{challenge}").as_bytes(),
    ))
}

fn store_challenge(
    connection: &Connection,
    challenge_id: &str,
    email: &str,
    challenge: &str,
    kind: &str,
) -> PrayResult<()> {
    let timestamp = current_unix_timestamp()?;
    connection.execute(
        "INSERT INTO auth_challenges (challenge_id, email, kind, challenge, created_at, used_at)
         VALUES (?1, ?2, ?3, ?4, ?5, NULL)
         ON CONFLICT(challenge_id) DO UPDATE SET email = excluded.email, kind = excluded.kind, challenge = excluded.challenge, created_at = excluded.created_at, used_at = NULL",
        rusqlite::params![challenge_id, email, kind, challenge, timestamp],
    )?;
    Ok(())
}

fn load_challenge(
    connection: &Connection,
    challenge_id: &str,
    email: &str,
    kind: &str,
) -> PrayResult<StoredChallenge> {
    let challenge: Option<StoredChallenge> = connection
        .query_row(
            "SELECT challenge FROM auth_challenges WHERE challenge_id = ?1 AND email = ?2 AND kind = ?3 AND used_at IS NULL",
            rusqlite::params![challenge_id, email, kind],
            |row| Ok(StoredChallenge { challenge: row.get(0)? }),
        )
        .optional()?;
    challenge.ok_or_else(|| PrayError::Resolution(format!("challenge not found for {email}")))
}

fn mark_challenge_used(connection: &Connection, challenge_id: &str) -> PrayResult<()> {
    let timestamp = current_unix_timestamp()?;
    connection.execute(
        "UPDATE auth_challenges SET used_at = ?2 WHERE challenge_id = ?1",
        rusqlite::params![challenge_id, timestamp],
    )?;
    Ok(())
}

fn load_passkey_public_key(connection: &Connection, credential_id: &str) -> PrayResult<String> {
    let public_key: String = connection.query_row(
        "SELECT public_key FROM passkeys WHERE credential_id = ?1",
        rusqlite::params![credential_id],
        |row| row.get(0),
    )?;
    Ok(public_key)
}

fn validate_signature(signature: &str) -> PrayResult<()> {
    if signature.trim().is_empty() {
        return Err(PrayError::Unsupported(
            "signature cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn verify_signature(public_key: &str, message: &[u8], signature: &str) -> PrayResult<()> {
    let (_, key_bytes) = parse_ssh_ed25519_public_key(public_key)?;
    let verifying_key = VerifyingKey::from_bytes(&key_bytes).map_err(|error| PrayError::Parse {
        kind: "public key",
        message: error.to_string(),
    })?;
    let signature_bytes =
        STANDARD
            .decode(signature.as_bytes())
            .map_err(|error| PrayError::Parse {
                kind: "signature",
                message: error.to_string(),
            })?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|error| PrayError::Verify(error.to_string()))?;
    verifying_key
        .verify(message, &signature)
        .map_err(|error| PrayError::Verify(error.to_string()))
}

fn parse_ssh_ed25519_public_key(public_key: &str) -> PrayResult<(String, [u8; 32])> {
    let mut fields = public_key.split_whitespace();
    let algorithm = fields.next().ok_or_else(|| {
        PrayError::Unsupported("public key must include an algorithm".to_string())
    })?;
    if algorithm != "ssh-ed25519" {
        return Err(PrayError::Unsupported(format!(
            "unsupported public key algorithm: {algorithm}"
        )));
    }
    let key_value = fields
        .next()
        .ok_or_else(|| PrayError::Unsupported("public key must include key bytes".to_string()))?;
    let blob = STANDARD
        .decode(key_value.as_bytes())
        .map_err(|error| PrayError::Parse {
            kind: "public key",
            message: error.to_string(),
        })?;
    let mut cursor = blob.as_slice();
    let blob_algorithm = read_ssh_string(&mut cursor)?;
    if blob_algorithm != b"ssh-ed25519" {
        return Err(PrayError::Parse {
            kind: "public key",
            message: "ed25519 public key blob must start with ssh-ed25519".to_string(),
        });
    }
    let key_bytes = read_ssh_string(&mut cursor)?;
    let key_bytes: [u8; 32] = key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| PrayError::Parse {
            kind: "public key",
            message: "ed25519 public key must be 32 bytes".to_string(),
        })?;
    Ok((format!("ssh-ed25519 {key_value}"), key_bytes))
}

fn read_ssh_string(cursor: &mut &[u8]) -> PrayResult<Vec<u8>> {
    let length = read_u32_from_slice(cursor)? as usize;
    if cursor.len() < length {
        return Err(PrayError::Resolution(
            "truncated ssh public key blob".to_string(),
        ));
    }
    let (value, rest) = cursor.split_at(length);
    *cursor = rest;
    Ok(value.to_vec())
}

fn read_u32_from_slice(cursor: &mut &[u8]) -> PrayResult<u32> {
    if cursor.len() < 4 {
        return Err(PrayError::Resolution("truncated ssh field".to_string()));
    }
    let (length_bytes, rest) = cursor.split_at(4);
    *cursor = rest;
    Ok(u32::from_be_bytes(
        length_bytes.try_into().expect("length bytes"),
    ))
}

fn generate_verification_code(email: &str, timestamp: u64) -> String {
    let payload = format!("{email}\0{timestamp}");
    let hash = sha256_prefixed(payload.as_bytes());
    let hex = hash.trim_start_matches("sha256:");
    let numeric = u32::from_str_radix(&hex[..8], 16).unwrap_or(0) % 1_000_000;
    format!("{:06}", numeric)
}

fn generate_session_token(email: &str, kind: &AuthSessionKind, timestamp: u64) -> String {
    let payload = format!("{email}\0{}\0{timestamp}", auth_session_kind_text(kind));
    sha256_prefixed(payload.as_bytes())
}

fn ssh_key_fingerprint(public_key: &str) -> String {
    sha256_prefixed(public_key.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

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
