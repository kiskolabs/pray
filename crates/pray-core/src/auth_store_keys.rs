use super::secrets::generate_auth_challenge;
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
        let mut connection = self.connection()?;
        let email: String = connection.query_row(
            "SELECT email FROM passkeys WHERE credential_id = ?1",
            rusqlite::params![credential_id],
            |row| row.get(0),
        )?;
        let public_key = load_passkey_public_key(&connection, credential_id)?;
        consume_challenge(
            &mut connection,
            challenge_id,
            &email,
            "passkey",
            |message| verify_signature(&public_key, message.as_bytes(), signature),
        )?;
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
        let mut connection = self.connection()?;
        let (public_key, _) = parse_ssh_ed25519_public_key(public_key)?;
        let fingerprint = ssh_key_fingerprint(&public_key);
        let email: String = connection.query_row(
            "SELECT email FROM ssh_keys WHERE fingerprint = ?1",
            rusqlite::params![fingerprint],
            |row| row.get(0),
        )?;
        consume_challenge(
            &mut connection,
            challenge_id,
            &email,
            "ssh_key",
            |message| verify_signature(&public_key, message.as_bytes(), signature),
        )?;
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
        reject_credential_reassignment(
            &connection,
            "passkeys",
            "credential_id",
            credential_id,
            email,
        )?;
        let timestamp = current_unix_timestamp()?;
        connection.execute(
        "INSERT INTO passkeys (credential_id, email, public_key, label, created_at, last_used_at)
         VALUES (?1, ?2, ?3, ?4, ?5, NULL)
         ON CONFLICT(credential_id) DO UPDATE SET public_key = excluded.public_key, label = excluded.label",
        rusqlite::params![credential_id, email, public_key, label.unwrap_or(""), timestamp],
    )?;
        Ok(AuthPasskeyEnrollmentResponse {
            email: email.to_string(),
            credential_id: credential_id.to_string(),
            enrolled: true,
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
        reject_credential_reassignment(
            &connection,
            "ssh_keys",
            "fingerprint",
            &fingerprint,
            email,
        )?;
        let timestamp = current_unix_timestamp()?;
        connection.execute(
        "INSERT INTO ssh_keys (fingerprint, email, public_key, label, created_at, last_used_at)
         VALUES (?1, ?2, ?3, ?4, ?5, NULL)
         ON CONFLICT(fingerprint) DO UPDATE SET public_key = excluded.public_key, label = excluded.label",
        rusqlite::params![fingerprint, email, public_key, label.unwrap_or(""), timestamp],
    )?;
        Ok(AuthSshKeyEnrollmentResponse {
            email: email.to_string(),
            fingerprint,
            enrolled: true,
        })
    }
}

fn consume_challenge(
    connection: &mut rusqlite::Connection,
    challenge_id: &str,
    email: &str,
    kind: &str,
    verify: impl FnOnce(&str) -> PrayResult<()>,
) -> PrayResult<()> {
    let transaction =
        connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let challenge = load_challenge(&transaction, challenge_id, email, kind)?;
    verify(&challenge.challenge)?;
    mark_challenge_used(&transaction, challenge_id)?;
    transaction.commit()?;
    Ok(())
}

fn reject_credential_reassignment(
    connection: &rusqlite::Connection,
    table: &str,
    column: &str,
    identity: &str,
    email: &str,
) -> PrayResult<()> {
    let existing: Option<String> = match (table, column) {
        ("passkeys", "credential_id") => connection
            .query_row(
                "SELECT email FROM passkeys WHERE credential_id = ?1",
                rusqlite::params![identity],
                |row| row.get(0),
            )
            .optional()?,
        ("ssh_keys", "fingerprint") => connection
            .query_row(
                "SELECT email FROM ssh_keys WHERE fingerprint = ?1",
                rusqlite::params![identity],
                |row| row.get(0),
            )
            .optional()?,
        _ => {
            return Err(PrayError::Unsupported(
                "unknown authenticator table".to_string(),
            ))
        }
    };
    match existing {
        Some(owner) if owner != email => Err(PrayError::Resolution(
            "authenticator already enrolled for another user".to_string(),
        )),
        _ => Ok(()),
    }
}
