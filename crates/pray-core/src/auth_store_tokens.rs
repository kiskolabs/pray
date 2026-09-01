use super::secrets::*;
use super::support::*;
use super::RegistryAuthStore;
use crate::{PrayError, PrayResult};
use rusqlite::OptionalExtension;

pub const PUBLISH_SCOPE: &str = "publish";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishTokenRecord {
    pub email: String,
    pub token: String,
    pub scopes: Vec<String>,
}

impl RegistryAuthStore {
    pub fn ensure_publish_tokens_table(&self) -> PrayResult<()> {
        let connection = self.connection()?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS publish_tokens (
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

    pub fn issue_publish_token(
        &self,
        email: &str,
        scopes: &[String],
    ) -> PrayResult<PublishTokenRecord> {
        validate_email(email)?;
        self.ensure_publish_tokens_table()?;
        let scopes = normalize_scopes(scopes)?;
        let connection = self.connection()?;
        let exists: Option<String> = connection
            .query_row(
                "SELECT email FROM users WHERE email = ?1",
                rusqlite::params![email],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            return Err(PrayError::Resolution(format!("unknown user: {email}")));
        }
        let timestamp = current_unix_timestamp()?;
        let token = generate_publish_token()?;
        let stored_token = stored_token(&token);
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO publish_tokens (token, email, scopes, created_at, last_used_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            rusqlite::params![stored_token, email, scopes.join(","), timestamp],
        )?;
        Ok(PublishTokenRecord {
            email: email.to_string(),
            token,
            scopes,
        })
    }

    pub fn resolve_publish_token(&self, token: &str) -> PrayResult<Option<PublishTokenRecord>> {
        if token.trim().is_empty() {
            return Ok(None);
        }
        self.ensure_publish_tokens_table()?;
        let connection = self.connection()?;
        let stored_token = stored_token(token);
        let row: Option<(String, String, u64)> = connection
            .query_row(
                "SELECT email, scopes, created_at FROM publish_tokens WHERE token = ?1",
                rusqlite::params![stored_token],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        let Some((email, scopes_text, created_at)) = row else {
            return Ok(None);
        };
        if record_expired(created_at, PUBLISH_TOKEN_TTL_SECONDS)? {
            return Ok(None);
        }
        let scopes = parse_scopes(&scopes_text);
        if !scopes.iter().any(|scope| scope == PUBLISH_SCOPE) {
            return Err(PrayError::Resolution(
                "publish token missing publish scope".to_string(),
            ));
        }
        let timestamp = current_unix_timestamp()?;
        connection.execute(
            "UPDATE publish_tokens SET last_used_at = ?2 WHERE token = ?1",
            rusqlite::params![stored_token, timestamp],
        )?;
        Ok(Some(PublishTokenRecord {
            email,
            token: token.to_string(),
            scopes,
        }))
    }

    pub fn revoke_publish_token(&self, token: &str) -> PrayResult<()> {
        self.ensure_publish_tokens_table()?;
        let connection = self.connection()?;
        let stored_token = stored_token(token);
        let deleted = connection.execute(
            "DELETE FROM publish_tokens WHERE token = ?1",
            rusqlite::params![stored_token],
        )?;
        if deleted == 0 {
            return Err(PrayError::Resolution("publish token not found".to_string()));
        }
        Ok(())
    }
}

pub fn bearer_token_from_authorization(header: Option<&str>) -> Option<String> {
    let header = header?.trim();
    let token = header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))?;
    let token = token.trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn normalize_scopes(scopes: &[String]) -> PrayResult<Vec<String>> {
    let mut normalized = Vec::new();
    for scope in scopes {
        let scope = scope.trim().to_ascii_lowercase();
        if scope.is_empty() {
            continue;
        }
        if scope != PUBLISH_SCOPE && scope != "publish-new" && scope != "publish-update" {
            return Err(PrayError::Unsupported(format!(
                "unsupported publish token scope: {scope}"
            )));
        }
        if !normalized.iter().any(|existing| existing == &scope) {
            normalized.push(scope);
        }
    }
    if !normalized.iter().any(|scope| scope == PUBLISH_SCOPE) {
        normalized.insert(0, PUBLISH_SCOPE.to_string());
    }
    Ok(normalized)
}

fn parse_scopes(scopes_text: &str) -> Vec<String> {
    scopes_text
        .split(',')
        .map(str::trim)
        .filter(|scope| !scope.is_empty())
        .map(|scope| scope.to_ascii_lowercase())
        .collect()
}
