use super::secrets::{record_expired, AUTH_CHALLENGE_TTL_SECONDS};
use crate::auth::AuthSessionKind;
use crate::hashing::sha256_prefixed;
use crate::trust::EmailConfirmationPolicy;
use crate::{PrayError, PrayResult};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use rusqlite::{Connection, OptionalExtension};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) struct StoredChallenge {
    pub(super) challenge: String,
}

pub(super) fn validate_email(email: &str) -> PrayResult<()> {
    let email = email.trim();
    if email.is_empty() || !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        return Err(PrayError::Unsupported(
            "email must be a non-empty address".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn validate_identifier(value: &str, label: &str) -> PrayResult<()> {
    if value.trim().is_empty() {
        return Err(PrayError::Unsupported(format!("{label} cannot be empty")));
    }
    Ok(())
}

pub(super) fn validate_public_key(public_key: &str) -> PrayResult<()> {
    let public_key = public_key.trim();
    if public_key.is_empty() {
        return Err(PrayError::Unsupported(
            "public key cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn ensure_user_can_authenticate(connection: &Connection, email: &str) -> PrayResult<()> {
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

pub(super) fn email_confirmation_policy_text(policy: EmailConfirmationPolicy) -> &'static str {
    match policy {
        EmailConfirmationPolicy::Required => "required",
        EmailConfirmationPolicy::Optional => "optional",
        EmailConfirmationPolicy::Disabled => "disabled",
    }
}

pub(super) fn auth_session_kind_text(kind: &AuthSessionKind) -> &'static str {
    match kind {
        AuthSessionKind::Email => "email",
        AuthSessionKind::Passkey => "passkey",
        AuthSessionKind::SshKey => "ssh_key",
    }
}

pub(super) fn parse_auth_session_kind(kind: &str) -> PrayResult<AuthSessionKind> {
    match kind {
        "email" => Ok(AuthSessionKind::Email),
        "passkey" => Ok(AuthSessionKind::Passkey),
        "ssh_key" => Ok(AuthSessionKind::SshKey),
        other => Err(PrayError::Resolution(format!(
            "unknown auth session kind: {other}"
        ))),
    }
}

pub(super) fn current_unix_timestamp() -> PrayResult<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PrayError::Resolution(error.to_string()))
        .map(|duration| duration.as_secs())
}

pub(super) fn generate_challenge_id(
    email: &str,
    subject: &str,
    kind: &str,
    challenge: &str,
) -> PrayResult<String> {
    Ok(sha256_prefixed(
        format!("challenge\0{email}\0{subject}\0{kind}\0{challenge}").as_bytes(),
    ))
}

pub(super) fn store_challenge(
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

pub(super) fn load_challenge(
    connection: &Connection,
    challenge_id: &str,
    email: &str,
    kind: &str,
) -> PrayResult<StoredChallenge> {
    let challenge: Option<(StoredChallenge, u64)> = connection
    .query_row(
        "SELECT challenge, created_at FROM auth_challenges WHERE challenge_id = ?1 AND email = ?2 AND kind = ?3 AND used_at IS NULL",
        rusqlite::params![challenge_id, email, kind],
        |row| Ok((StoredChallenge { challenge: row.get(0)? }, row.get(1)?)),
    )
    .optional()?;
    let Some((challenge, created_at)) = challenge else {
        return Err(PrayError::Resolution(format!(
            "challenge not found for {email}"
        )));
    };
    if record_expired(created_at, AUTH_CHALLENGE_TTL_SECONDS)? {
        return Err(PrayError::Resolution(format!(
            "challenge expired for {email}"
        )));
    }
    Ok(challenge)
}

pub(super) fn mark_challenge_used(connection: &Connection, challenge_id: &str) -> PrayResult<()> {
    let timestamp = current_unix_timestamp()?;
    connection.execute(
        "UPDATE auth_challenges SET used_at = ?2 WHERE challenge_id = ?1",
        rusqlite::params![challenge_id, timestamp],
    )?;
    Ok(())
}

pub(super) fn load_passkey_public_key(
    connection: &Connection,
    credential_id: &str,
) -> PrayResult<String> {
    let public_key: String = connection.query_row(
        "SELECT public_key FROM passkeys WHERE credential_id = ?1",
        rusqlite::params![credential_id],
        |row| row.get(0),
    )?;
    Ok(public_key)
}

pub(super) fn validate_signature(signature: &str) -> PrayResult<()> {
    if signature.trim().is_empty() {
        return Err(PrayError::Unsupported(
            "signature cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn verify_signature(
    public_key: &str,
    message: &[u8],
    signature: &str,
) -> PrayResult<()> {
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

pub(super) fn parse_ssh_ed25519_public_key(public_key: &str) -> PrayResult<(String, [u8; 32])> {
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

pub(super) fn read_ssh_string(cursor: &mut &[u8]) -> PrayResult<Vec<u8>> {
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

pub(super) fn read_u32_from_slice(cursor: &mut &[u8]) -> PrayResult<u32> {
    if cursor.len() < 4 {
        return Err(PrayError::Resolution("truncated ssh field".to_string()));
    }
    let (length_bytes, rest) = cursor.split_at(4);
    *cursor = rest;
    Ok(u32::from_be_bytes(
        length_bytes.try_into().expect("length bytes"),
    ))
}

pub fn ssh_public_key_fingerprint_text(public_key: &str) -> PrayResult<String> {
    let (canonical, _) = parse_ssh_ed25519_public_key(public_key)?;
    Ok(normalize_ssh_fingerprint(&ssh_key_fingerprint(&canonical)))
}

pub(super) fn normalize_ssh_fingerprint(fingerprint: &str) -> String {
    fingerprint.trim().to_ascii_uppercase()
}

pub(super) fn ssh_key_fingerprint(public_key: &str) -> String {
    sha256_prefixed(public_key.as_bytes())
}
