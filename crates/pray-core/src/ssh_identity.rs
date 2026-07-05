use crate::PrayResult;

const USER_FINGERPRINT_ENV_VARS: &[&str] = &[
    "PRAY_SSH_USER_FINGERPRINT",
    "SSH_USER_FINGERPRINT",
    "PRAY_SSH_PUBLISHER",
];

pub fn normalize_identity(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

pub fn active_ssh_user_fingerprint() -> Option<String> {
    for name in USER_FINGERPRINT_ENV_VARS {
        if let Ok(value) = std::env::var(name) {
            let normalized = normalize_identity(&value);
            if !normalized.is_empty() {
                return Some(normalized);
            }
        }
    }
    None
}

pub fn signing_identity(label: &str, fingerprint: Option<&str>) -> String {
    if let Some(fingerprint) = fingerprint {
        let normalized = normalize_identity(fingerprint);
        if looks_like_ssh_fingerprint(&normalized) {
            return normalized;
        }
    }
    label.trim().to_string()
}

pub fn package_signing_identity(
    signer: Option<&str>,
    signer_fingerprint: Option<&str>,
) -> Option<String> {
    if let Some(fingerprint) = signer_fingerprint {
        let normalized = normalize_identity(fingerprint);
        if looks_like_ssh_fingerprint(&normalized) {
            return Some(normalized);
        }
    }
    signer
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub fn looks_like_ssh_fingerprint(value: &str) -> bool {
    let upper = normalize_identity(value);
    upper.starts_with("SHA256:")
        || upper.starts_with("SHA512:")
        || upper.starts_with("MD5:")
}

pub fn fingerprint_matches_allowed(identity: &str, allowed: &[String]) -> bool {
    if allowed.is_empty() {
        return false;
    }
    let normalized = normalize_identity(identity);
    allowed
        .iter()
        .any(|entry| normalize_identity(entry) == normalized)
}

pub fn read_env_identity(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| normalize_identity(&value))
        .filter(|value| !value.is_empty())
}

pub fn ssh_public_key_fingerprint(public_key: &str) -> PrayResult<String> {
    crate::auth::ssh_public_key_fingerprint_text(public_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signing_identity_prefers_fingerprint() {
        assert_eq!(
            signing_identity("alice@example.com", Some("sha256:abc")),
            "SHA256:ABC"
        );
        assert_eq!(
            signing_identity("alice@example.com", None),
            "alice@example.com"
        );
    }

    #[test]
    fn package_signing_identity_prefers_fingerprint_field() {
        assert_eq!(
            package_signing_identity(Some("alice@example.com"), Some("sha256:deadbeef"),),
            Some("SHA256:DEADBEEF".to_string())
        );
    }

    #[test]
    fn package_signing_identity_ignores_non_fingerprint_field() {
        assert_eq!(
            package_signing_identity(Some("alice@example.com"), Some("alice@example.com"),),
            Some("alice@example.com".to_string())
        );
    }
}
