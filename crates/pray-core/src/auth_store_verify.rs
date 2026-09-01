use super::secrets::*;
use super::support::*;
use super::RegistryAuthStore;
use crate::auth::AuthVerificationResponse;
use crate::{PrayError, PrayResult};
use rusqlite::OptionalExtension;

impl RegistryAuthStore {
    pub fn verify_email(&self, email: &str, code: &str) -> PrayResult<AuthVerificationResponse> {
        validate_email(email)?;
        let connection = self.connection()?;
        let stored: Option<(String, u64, i64)> = connection
            .query_row(
                "SELECT code, created_at, failed_attempts FROM email_verification_codes WHERE email = ?1",
                rusqlite::params![email],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        let Some((stored_code, created_at, failed_attempts)) = stored else {
            return verification_failed();
        };
        if failed_attempts >= i64::from(MAX_VERIFICATION_ATTEMPTS)
            || record_expired(created_at, VERIFICATION_CODE_TTL_SECONDS)?
            || code.trim().is_empty()
            || !constant_time_eq(&stored_code, &stored_token(code))
        {
            connection.execute(
                "UPDATE email_verification_codes SET failed_attempts = failed_attempts + 1 WHERE email = ?1",
                rusqlite::params![email],
            )?;
            return verification_failed();
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
}

fn verification_failed() -> PrayResult<AuthVerificationResponse> {
    Err(PrayError::Resolution(VERIFICATION_FAILED.to_string()))
}
