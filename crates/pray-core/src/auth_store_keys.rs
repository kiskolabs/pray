use super::support::*;
use super::RegistryAuthStore;
use crate::auth::{
    AuthPasskeyChallengeResponse, AuthPasskeyEnrollmentResponse, AuthPasskeyLoginResponse,
    AuthSessionKind, AuthSshKeyChallengeResponse, AuthSshKeyEnrollmentResponse,
    AuthSshKeyLoginResponse,
};
use crate::{PrayError, PrayResult};
use rusqlite::OptionalExtension;

impl RegistryAuthStore {
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
    pub fn login_with_passkey(
        &self,
        credential_id: &str,
    ) -> PrayResult<AuthPasskeyLoginResponse> {
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
}
