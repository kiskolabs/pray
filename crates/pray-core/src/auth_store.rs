#[path = "auth_store_keys.rs"]
mod keys;
#[path = "auth_store_secrets.rs"]
mod secrets;
#[path = "auth_store_support.rs"]
mod support;
#[path = "auth_store_tokens.rs"]
mod tokens;
#[path = "auth_store_verify.rs"]
mod verify;

use secrets::*;
use support::*;

pub use tokens::{bearer_token_from_authorization, PublishTokenRecord, PUBLISH_SCOPE};

use crate::auth::{AuthRegistrationResponse, AuthSessionKind, AuthSessionResponse};
use crate::trust::EmailConfirmationPolicy;
use crate::{PrayError, PrayResult};
use rusqlite::{Connection, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RegistryAuthStore {
    database_path: PathBuf,
}

impl RegistryAuthStore {
    pub fn open(root: &Path) -> PrayResult<Self> {
        let database_path = root.join(".pray/auth.db");
        if let Some(parent) = database_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let store = Self { database_path };
        store.initialize()?;
        restrict_auth_file_permissions(&store.database_path)?;
        Ok(store)
    }
    pub fn register_email(
        &self,
        email: &str,
        policy: EmailConfirmationPolicy,
    ) -> PrayResult<AuthRegistrationResponse> {
        validate_email(email)?;
        let connection = self.connection()?;
        let existing: Option<bool> = connection
            .query_row(
                "SELECT email_verified FROM users WHERE email = ?1",
                rusqlite::params![email],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(verified) = existing {
            return Ok(AuthRegistrationResponse {
                email: email.to_string(),
                verified,
                verification_code: None,
            });
        }
        let timestamp = current_unix_timestamp()?;
        let verified = matches!(policy, EmailConfirmationPolicy::Disabled);
        let verification_code = if verified {
            None
        } else {
            Some(generate_verification_code()?)
        };
        let policy_text = email_confirmation_policy_text(policy);

        connection.execute(
        "INSERT INTO users (email, email_verified, email_confirmation_policy, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![email, verified, policy_text, timestamp],
    )?;
        if let Some(code) = verification_code.as_ref() {
            connection.execute(
            "INSERT INTO email_verification_codes (email, code, created_at, verified_at, failed_attempts)
             VALUES (?1, ?2, ?3, NULL, 0)
             ON CONFLICT(email) DO UPDATE SET code = excluded.code, created_at = excluded.created_at, verified_at = NULL, failed_attempts = 0",
            rusqlite::params![email, stored_token(code), timestamp],
        )?;
        }

        Ok(AuthRegistrationResponse {
            email: email.to_string(),
            verified,
            verification_code,
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
        let token = generate_session_token()?;
        let stored_token = stored_token(&token);
        connection.execute(
            "INSERT INTO sessions (token, email, kind, created_at, last_used_at)
             VALUES (?1, ?2, ?3, ?4, ?4)
             ON CONFLICT(token) DO UPDATE SET last_used_at = excluded.last_used_at",
            rusqlite::params![
                stored_token,
                email,
                auth_session_kind_text(&kind),
                timestamp
            ],
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
        let stored_token = stored_token(token);
        let session: Option<(String, String, u64)> = connection
            .query_row(
                "SELECT email, kind, created_at FROM sessions WHERE token = ?1",
                rusqlite::params![stored_token],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        let Some((email, kind_text, created_at)) = session else {
            return Ok(None);
        };
        if record_expired(created_at, SESSION_TTL_SECONDS)? {
            return Ok(None);
        }
        let kind = parse_auth_session_kind(&kind_text)?;
        let timestamp = current_unix_timestamp()?;
        connection.execute(
            "UPDATE sessions SET last_used_at = ?2 WHERE token = ?1",
            rusqlite::params![stored_token, timestamp],
        )?;
        Ok(Some(AuthSessionResponse {
            email,
            token: token.to_string(),
            kind,
        }))
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
            verified_at INTEGER,
            failed_attempts INTEGER NOT NULL DEFAULT 0
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
        );
        CREATE TABLE IF NOT EXISTS publish_tokens (
            token TEXT PRIMARY KEY,
            email TEXT NOT NULL,
            scopes TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            last_used_at INTEGER,
            FOREIGN KEY(email) REFERENCES users(email) ON DELETE CASCADE
        );",
        )?;
        let _ = connection.execute(
            "ALTER TABLE email_verification_codes ADD COLUMN failed_attempts INTEGER NOT NULL DEFAULT 0",
            [],
        );
        Ok(())
    }
    fn connection(&self) -> PrayResult<Connection> {
        let connection = Connection::open(&self.database_path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.busy_timeout(std::time::Duration::from_millis(5_000))?;
        Ok(connection)
    }
}

fn restrict_auth_file_permissions(database_path: &Path) -> PrayResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Some(parent) = database_path.parent() {
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        }
        fs::set_permissions(database_path, fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    {
        let _ = database_path;
    }
    Ok(())
}

pub use support::ssh_public_key_fingerprint_text;
