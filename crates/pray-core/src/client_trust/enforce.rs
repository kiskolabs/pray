use std::path::Path;

use crate::{PrayError, PrayResult};

use super::git::{
    commit_signing_fingerprint, commit_signing_key, is_git_repository, is_remote_git_url,
    trust_git_run,
};
use super::policy::{best_rule, load_policy, normalize_key, ClientTrustRule};

pub fn gate_git_source(
    home: &Path,
    source_url: &str,
    repository: &Path,
    revision: Option<&str>,
) -> PrayResult<()> {
    if !is_remote_git_url(source_url) {
        return Ok(());
    }
    if !is_git_repository(repository) {
        return Ok(());
    }
    enforce_source_trust(home, source_url, repository, revision)?;
    if load_policy(home)?.is_some() {
        super::prompt::prompt_untrusted_source_consent(home, source_url, repository)?;
    }
    Ok(())
}

pub fn enforce_source_trust(
    home: &Path,
    source_url: &str,
    repository: &Path,
    revision: Option<&str>,
) -> PrayResult<()> {
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
        enforce_signed_commit(
            home,
            source_url,
            repository,
            revision,
            &rule.allowed_signing_keys,
        )?;
    }
    Ok(())
}

fn enforce_signed_commit(
    home: &Path,
    source_url: &str,
    repository: &Path,
    revision: Option<&str>,
    allowed_signing_keys: &[String],
) -> PrayResult<()> {
    let target = commit_ref(revision)?;
    trust_git_run(home, source_url, repository, &["verify-commit", target]).map_err(|error| {
        PrayError::Integrity(format!(
            "trust policy requires signed commit, but {target} failed signature verification in {}: {error}",
            repository.display()
        ))
    })?;

    if allowed_signing_keys.is_empty() {
        return Ok(());
    }

    let key = commit_signing_key(home, source_url, repository, revision)
        .map(|value| normalize_key(&value));
    let fingerprint = commit_signing_fingerprint(home, source_url, repository, revision)
        .map(|value| normalize_key(&value));
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

    Err(PrayError::Integrity(format!(
        "trust policy signer mismatch: {target} signing key/fingerprint is not in allowed_signing_keys"
    )))
}

fn commit_ref(revision: Option<&str>) -> PrayResult<&str> {
    let Some(revision) = revision else {
        return Ok("HEAD");
    };
    if revision.is_empty() || revision.starts_with('-') {
        return Err(PrayError::Integrity(format!(
            "refusing trust verification for unsafe revision {revision:?}"
        )));
    }
    Ok(revision)
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
    let key =
        commit_signing_key(home, source_url, repository, None).map(|value| normalize_key(&value));
    let fingerprint = commit_signing_fingerprint(home, source_url, repository, None)
        .map(|value| normalize_key(&value));
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
    std::env::var_os(name).is_some_and(|value| {
        let text = value.to_string_lossy();
        matches!(
            text.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}
