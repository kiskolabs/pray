use std::path::Path;

use crate::{PrayError, PrayResult};

use super::git::repository_signing_keys;
use super::policy::{
    append_missing_keys, best_rule, format_rule_block, load_policy_or_default,
    mutable_rule_for_match_prefix, normalize_key, save_policy, ClientTrustPolicy, ClientTrustRule,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustListScope {
    All,
    Global,
    Local,
}

pub fn show_policy_toml(home: &Path) -> PrayResult<String> {
    let policy = load_policy_or_default(home)?;
    let text =
        toml::to_string_pretty(&policy).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(text)
}

pub fn list_policy(
    home: &Path,
    scope: TrustListScope,
    source_url: Option<&str>,
) -> PrayResult<String> {
    let policy = load_policy_or_default(home)?;
    let mut output = String::new();

    if let Some(source) = source_url {
        output.push_str(&format!("source: {source}\n\n"));
        if matches!(scope, TrustListScope::All | TrustListScope::Global) {
            output.push_str(&format_rule_block("scope: global", &policy.default));
            output.push('\n');
        }
        if matches!(scope, TrustListScope::All | TrustListScope::Local) {
            let mut matched: Vec<&ClientTrustRule> = policy
                .rules
                .iter()
                .filter(|rule| {
                    rule.match_prefix
                        .as_deref()
                        .is_some_and(|prefix| source.starts_with(prefix))
                })
                .collect();
            matched.sort_by_key(|rule| {
                std::cmp::Reverse(rule.match_prefix.as_deref().map(str::len).unwrap_or(0))
            });
            if matched.is_empty() {
                output.push_str("scope: local\n  (no matching rules)\n");
            } else {
                for rule in matched {
                    let prefix = rule.match_prefix.as_deref().unwrap_or("-");
                    output.push_str(&format_rule_block(
                        &format!("scope: local ({prefix})"),
                        rule,
                    ));
                }
            }
            output.push('\n');
        }
        if matches!(scope, TrustListScope::All) {
            let effective = best_rule(&policy, source);
            if let Some(prefix) = effective.match_prefix.as_deref() {
                output.push_str(&format!("effective_scope: local ({prefix})\n"));
            } else {
                output.push_str("effective_scope: global\n");
            }
        }
        return Ok(output.trim_end().to_string());
    }

    if matches!(scope, TrustListScope::All | TrustListScope::Global) {
        output.push_str(&format_rule_block("scope: global", &policy.default));
        output.push('\n');
    }
    if matches!(scope, TrustListScope::All | TrustListScope::Local) {
        if policy.rules.is_empty() {
            output.push_str("scope: local\n  (no rules)\n");
        } else {
            let mut rules: Vec<&ClientTrustRule> = policy.rules.iter().collect();
            rules.sort_by(|left, right| left.match_prefix.cmp(&right.match_prefix));
            for rule in rules {
                let prefix = rule.match_prefix.as_deref().unwrap_or("-");
                output.push_str(&format_rule_block(
                    &format!("scope: local ({prefix})"),
                    rule,
                ));
            }
        }
    }
    Ok(output.trim_end().to_string())
}

pub fn add_allowed_signing_key(
    home: &Path,
    key: &str,
    match_prefix: Option<&str>,
) -> PrayResult<()> {
    let normalized = normalize_key(key);
    if normalized.is_empty() {
        return Err(PrayError::Unsupported("signing key is empty".into()));
    }

    let mut policy = load_policy_or_default(home)?;
    let rule = if let Some(prefix) = match_prefix {
        mutable_rule_for_match_prefix(&mut policy, prefix)
    } else {
        &mut policy.default
    };
    if !rule
        .allowed_signing_keys
        .iter()
        .any(|existing| normalize_key(existing) == normalized)
    {
        rule.allowed_signing_keys.push(normalized);
    }
    save_policy(home, &policy)
}

pub fn remove_allowed_signing_key(
    home: &Path,
    key: &str,
    match_prefix: Option<&str>,
) -> PrayResult<()> {
    let normalized = normalize_key(key);
    if normalized.is_empty() {
        return Err(PrayError::Unsupported("signing key is empty".into()));
    }

    let mut policy = load_policy_or_default(home)?;
    let rule = if let Some(prefix) = match_prefix {
        mutable_rule_for_match_prefix(&mut policy, prefix)
    } else {
        &mut policy.default
    };
    let before = rule.allowed_signing_keys.len();
    rule.allowed_signing_keys
        .retain(|existing| normalize_key(existing) != normalized);
    if rule.allowed_signing_keys.len() == before {
        return Err(PrayError::Unsupported(format!(
            "signing key not found in allowed_signing_keys for {}",
            match_prefix.unwrap_or("<default>")
        )));
    }
    save_policy(home, &policy)
}

pub fn set_require_signed_commit(home: &Path, match_prefix: &str, enabled: bool) -> PrayResult<()> {
    if match_prefix.trim().is_empty() {
        return Err(PrayError::Unsupported("match-prefix is empty".into()));
    }
    let mut policy = load_policy_or_default(home)?;
    let rule = mutable_rule_for_match_prefix(&mut policy, match_prefix);
    rule.require_signed_commit = enabled;
    save_policy(home, &policy)
}

pub fn set_allow(home: &Path, match_prefix: &str, allow: bool) -> PrayResult<()> {
    if match_prefix.trim().is_empty() {
        return Err(PrayError::Unsupported("match-prefix is empty".into()));
    }
    let mut policy = load_policy_or_default(home)?;
    let rule = mutable_rule_for_match_prefix(&mut policy, match_prefix);
    rule.allow = allow;
    save_policy(home, &policy)
}

pub fn import_signing_keys_from_repository(
    home: &Path,
    source_url: &str,
    repository: &Path,
    match_prefix: Option<&str>,
) -> PrayResult<usize> {
    let keys = repository_signing_keys(home, source_url, repository);
    if keys.is_empty() {
        return Err(PrayError::Unsupported(format!(
            "no commit signing key/fingerprint found for HEAD in {}",
            repository.display()
        )));
    }
    let mut policy = load_policy_or_default(home)?;
    let rule = if let Some(prefix) = match_prefix {
        mutable_rule_for_match_prefix(&mut policy, prefix)
    } else {
        &mut policy.default
    };
    let added = append_missing_keys(rule, &keys);
    save_policy(home, &policy)?;
    Ok(added)
}

pub fn ensure_policy_file(home: &Path) -> PrayResult<ClientTrustPolicy> {
    let policy = load_policy_or_default(home)?;
    if super::policy::trust_policy_path(home).is_file() {
        return Ok(policy);
    }
    save_policy(home, &policy)?;
    Ok(policy)
}
