#[path = "auth_store_keys.rs"]
mod keys;
#[path = "auth_store_support.rs"]
mod support;
#[path = "auth_store_tokens.rs"]
mod tokens;

use support::*;

pub use tokens::{bearer_token_from_authorization, PublishTokenRecord, PUBLISH_SCOPE};

use crate::auth::{
    AuthRegistrationResponse, AuthSessionKind, AuthSessionResponse, AuthVerificationResponse,
};
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
        Ok(())
    }
    fn connection(&self) -> PrayResult<Connection> {
        let connection = Connection::open(&self.database_path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        Ok(connection)
    }
}

pub use support::ssh_public_key_fingerprint_text;
