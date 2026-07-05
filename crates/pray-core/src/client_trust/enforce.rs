use std::path::Path;

use crate::{PrayError, PrayResult};

use super::git::{
    commit_signing_fingerprint, commit_signing_key, is_remote_git_url, trust_git_run,
};
use super::policy::{best_rule, load_policy, normalize_key, ClientTrustRule};

pub fn gate_git_source(home: &Path, source_url: &str, repository: &Path) -> PrayResult<()> {
    if !is_remote_git_url(source_url) {
        return Ok(());
    }
    if !repository.join(".git").is_dir() {
        return Ok(());
    }
    enforce_source_trust(home, source_url, repository)?;
    if load_policy(home)?.is_some() {
        super::prompt::prompt_untrusted_source_consent(home, source_url, repository)?;
    }
    Ok(())
}

pub fn enforce_source_trust(home: &Path, source_url: &str, repository: &Path) -> PrayResult<()> {
    let Some(policy) = load_policy(home)? else {
        return Ok(());
    };
    let rule = best_rule(&policy, source_url);
    if !rule.allow {
        return Err(PrayError::Integrity(format!(
            "source is blocked by trust policy: {source_url}"
        )));
    }
    if rule.require_signed_commit {
        enforce_signed_commit(home, source_url, repository, &rule.allowed_signing_keys)?;
    }
    Ok(())
}

fn enforce_signed_commit(
    home: &Path,
    source_url: &str,
    repository: &Path,
    allowed_signing_keys: &[String],
) -> PrayResult<()> {
    trust_git_run(home, source_url, repository, &["verify-commit", "HEAD"]).map_err(|error| {
        PrayError::Integrity(format!(
            "trust policy requires signed commit, but HEAD failed signature verification in {}: {error}",
            repository.display()
        ))
    })?;

    if allowed_signing_keys.is_empty() {
        return Ok(());
    }

    let key = commit_signing_key(home, source_url, repository).map(|value| normalize_key(&value));
    let fingerprint =
        commit_signing_fingerprint(home, source_url, repository).map(|value| normalize_key(&value));
    let allowed: Vec<String> = allowed_signing_keys
        .iter()
        .map(|value| normalize_key(value))
        .collect();
    let key_ok = key
        .as_ref()
        .is_some_and(|value| allowed.iter().any(|allowed_key| allowed_key == value));
    let fingerprint_ok = fingerprint
        .as_ref()
        .is_some_and(|value| allowed.iter().any(|allowed_key| allowed_key == value));
    if key_ok || fingerprint_ok {
        return Ok(());
    }

    Err(PrayError::Integrity(
        "trust policy signer mismatch: HEAD signing key/fingerprint is not in allowed_signing_keys"
            .into(),
    ))
}

pub fn signer_matches_allowed(
    home: &Path,
    source_url: &str,
    rule: &ClientTrustRule,
    repository: &Path,
) -> bool {
    if rule.allowed_signing_keys.is_empty() {
        return false;
    }
    let key = commit_signing_key(home, source_url, repository).map(|value| normalize_key(&value));
    let fingerprint =
        commit_signing_fingerprint(home, source_url, repository).map(|value| normalize_key(&value));
    let allowed: Vec<String> = rule
        .allowed_signing_keys
        .iter()
        .map(|value| normalize_key(value))
        .collect();
    key.as_ref()
        .is_some_and(|value| allowed.iter().any(|allowed_key| allowed_key == value))
        || fingerprint
            .as_ref()
            .is_some_and(|value| allowed.iter().any(|allowed_key| allowed_key == value))
}

pub fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}
