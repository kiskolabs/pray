use super::support::current_unix_timestamp;
use crate::hashing::sha256_prefixed;
use crate::{PrayError, PrayResult};

pub(super) const AUTH_CHALLENGE_TTL_SECONDS: u64 = 5 * 60;
pub(super) const VERIFICATION_CODE_TTL_SECONDS: u64 = 15 * 60;
pub(super) const SESSION_TTL_SECONDS: u64 = 30 * 24 * 60 * 60;
pub(super) const PUBLISH_TOKEN_TTL_SECONDS: u64 = 90 * 24 * 60 * 60;
pub(super) const MAX_VERIFICATION_ATTEMPTS: u32 = 5;
pub(super) const VERIFICATION_FAILED: &str = "verification failed";

pub(super) fn generate_auth_challenge(_kind: &str, _subject: &str) -> PrayResult<String> {
    random_secret("challenge")
}

pub(super) fn generate_verification_code() -> PrayResult<String> {
    random_secret("verify")
}

pub(super) fn generate_session_token() -> PrayResult<String> {
    random_secret("session")
}

pub(super) fn generate_publish_token() -> PrayResult<String> {
    random_secret("publish")
}

pub(super) fn stored_token(token: &str) -> String {
    sha256_prefixed(token.as_bytes())
}

pub(super) fn constant_time_eq(left: &str, right: &str) -> bool {
    let left_bytes = left.as_bytes();
    let right_bytes = right.as_bytes();
    if left_bytes.len() != right_bytes.len() {
        return false;
    }
    let mut difference = 0u8;
    for (left_byte, right_byte) in left_bytes.iter().zip(right_bytes.iter()) {
        difference |= left_byte ^ right_byte;
    }
    difference == 0
}

pub(super) fn record_expired(created_at: u64, ttl_seconds: u64) -> PrayResult<bool> {
    Ok(current_unix_timestamp()?.saturating_sub(created_at) > ttl_seconds)
}

fn random_secret(domain: &str) -> PrayResult<String> {
    let mut bytes = [0u8; 32];
    fill_random(&mut bytes)?;
    let mut payload = domain.as_bytes().to_vec();
    payload.extend_from_slice(&bytes);
    Ok(sha256_prefixed(&payload))
}

fn fill_random(bytes: &mut [u8]) -> PrayResult<()> {
    getrandom::fill(bytes).map_err(|_| {
        PrayError::Resolution("operating system random source unavailable".to_string())
    })
}
