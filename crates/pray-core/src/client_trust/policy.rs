use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct ClientTrustPolicy {
    #[serde(default)]
    pub default: ClientTrustRule,
    #[serde(default)]
    pub rules: Vec<ClientTrustRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ClientTrustRule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_prefix: Option<String>,
    #[serde(default = "default_allow")]
    pub allow: bool,
    #[serde(default)]
    pub require_signed_commit: bool,
    #[serde(default)]
    pub allowed_signing_keys: Vec<String>,
    #[serde(default)]
    pub allowed_host_keys: Vec<String>,
    #[serde(default)]
    pub allowed_publishers: Vec<String>,
}

impl Default for ClientTrustRule {
    fn default() -> Self {
        Self {
            match_prefix: None,
            allow: true,
            require_signed_commit: false,
            allowed_signing_keys: Vec::new(),
            allowed_host_keys: Vec::new(),
            allowed_publishers: Vec::new(),
        }
    }
}

fn default_allow() -> bool {
    true
}

pub fn trust_policy_path(home: &Path) -> std::path::PathBuf {
    home.join("trust.toml")
}

pub fn load_policy(home: &Path) -> PrayResult<Option<ClientTrustPolicy>> {
    let path = trust_policy_path(home);
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path)?;
    let policy: ClientTrustPolicy = toml::from_str(&text).map_err(|error| PrayError::Parse {
        kind: "client trust policy",
        message: error.to_string(),
    })?;
    Ok(Some(policy))
}

pub fn load_policy_or_default(home: &Path) -> PrayResult<ClientTrustPolicy> {
    Ok(load_policy(home)?.unwrap_or_default())
}

pub fn save_policy(home: &Path, policy: &ClientTrustPolicy) -> PrayResult<()> {
    let path = trust_policy_path(home);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text =
        toml::to_string_pretty(policy).map_err(|error| PrayError::Manifest(error.to_string()))?;
    fs::write(path, text)?;
    Ok(())
}

pub fn best_rule<'a>(policy: &'a ClientTrustPolicy, source_url: &str) -> &'a ClientTrustRule {
    let mut best: Option<&ClientTrustRule> = None;
    let mut best_length = 0usize;
    for rule in &policy.rules {
        let Some(prefix) = rule.match_prefix.as_deref() else {
            continue;
        };
        if source_url.starts_with(prefix) && prefix.len() > best_length {
            best = Some(rule);
            best_length = prefix.len();
        }
    }
    best.unwrap_or(&policy.default)
}

pub fn normalize_key(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

pub fn source_scope_id(source_url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    source_url.hash(&mut hasher);
    let hash = format!("{:016x}", hasher.finish());
    let mut slug: String = source_url
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect();
    if slug.len() > 64 {
        slug.truncate(64);
    }
    format!("{slug}-{hash}")
}

pub fn mutable_rule_for_match_prefix<'a>(
    policy: &'a mut ClientTrustPolicy,
    match_prefix: &str,
) -> &'a mut ClientTrustRule {
    if let Some(index) = policy
        .rules
        .iter()
        .position(|rule| rule.match_prefix.as_deref() == Some(match_prefix))
    {
        return &mut policy.rules[index];
    }
    policy.rules.push(ClientTrustRule {
        match_prefix: Some(match_prefix.to_string()),
        ..ClientTrustRule::default()
    });
    policy.rules.last_mut().expect("rule just pushed")
}

pub fn append_missing_publishers(rule: &mut ClientTrustRule, keys: &[String]) -> usize {
    append_missing_identity_list(&mut rule.allowed_publishers, keys)
}

pub fn append_missing_host_keys(rule: &mut ClientTrustRule, keys: &[String]) -> usize {
    append_missing_identity_list(&mut rule.allowed_host_keys, keys)
}

fn append_missing_identity_list(target: &mut Vec<String>, keys: &[String]) -> usize {
    let mut added = 0usize;
    for key in keys {
        let normalized = normalize_key(key);
        if normalized.is_empty() {
            continue;
        }
        if target
            .iter()
            .any(|existing| normalize_key(existing) == normalized)
        {
            continue;
        }
        target.push(normalized);
        added += 1;
    }
    added
}

pub fn append_missing_keys(rule: &mut ClientTrustRule, keys: &[String]) -> usize {
    append_missing_identity_list(&mut rule.allowed_signing_keys, keys)
}

pub fn keys_missing_for_trust_scope(
    home: &Path,
    source_url: &str,
    keys: &[String],
    global_scope: bool,
) -> PrayResult<Vec<String>> {
    let policy = load_policy_or_default(home)?;
    let rule = if global_scope {
        &policy.default
    } else {
        best_rule(&policy, source_url)
    };
    let mut missing = Vec::new();
    for key in keys {
        let normalized = normalize_key(key);
        if normalized.is_empty() {
            continue;
        }
        if rule
            .allowed_signing_keys
            .iter()
            .any(|existing| normalize_key(existing) == normalized)
        {
            continue;
        }
        missing.push(normalized);
    }
    Ok(missing)
}

pub fn format_rule_block(scope: &str, rule: &ClientTrustRule) -> String {
    let mut out = format!("{scope}\n");
    out.push_str(&format!("  allow: {}\n", rule.allow));
    out.push_str(&format!(
        "  require_signed_commit: {}\n",
        rule.require_signed_commit
    ));
    if rule.allowed_signing_keys.is_empty() {
        out.push_str("  allowed_signing_keys: []\n");
    } else {
        out.push_str("  allowed_signing_keys:\n");
        for key in &rule.allowed_signing_keys {
            out.push_str(&format!("    - {key}\n"));
        }
    }
    if rule.allowed_host_keys.is_empty() {
        out.push_str("  allowed_host_keys: []\n");
    } else {
        out.push_str("  allowed_host_keys:\n");
        for key in &rule.allowed_host_keys {
            out.push_str(&format!("    - {key}\n"));
        }
    }
    if rule.allowed_publishers.is_empty() {
        out.push_str("  allowed_publishers: []\n");
    } else {
        out.push_str("  allowed_publishers:\n");
        for key in &rule.allowed_publishers {
            out.push_str(&format!("    - {key}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_match_prefix_wins() {
        let policy = ClientTrustPolicy {
            default: ClientTrustRule::default(),
            rules: vec![
                ClientTrustRule {
                    match_prefix: Some("https://github.com/org/".into()),
                    require_signed_commit: true,
                    ..ClientTrustRule::default()
                },
                ClientTrustRule {
                    match_prefix: Some("https://github.com/org/repo".into()),
                    allow: false,
                    ..ClientTrustRule::default()
                },
            ],
        };
        let rule = best_rule(&policy, "https://github.com/org/repo.git");
        assert!(!rule.allow);
    }
}
