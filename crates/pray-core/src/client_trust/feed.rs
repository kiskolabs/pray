use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::{PrayError, PrayResult};
use serde::Deserialize;

pub const DEFAULT_COMPROMISED_KEYS_SOURCE: &str =
    "https://raw.githubusercontent.com/bmx-rs/trust-lists/main/compromised-keys.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompromisedKeyEntry {
    pub key: String,
    pub reason: Option<String>,
    pub reference: Option<String>,
    pub reported_at: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct CompromisedTomlFeed {
    #[serde(default)]
    keys: Vec<CompromisedTomlEntry>,
}

#[derive(Debug, Deserialize, Default)]
struct CompromisedTomlEntry {
    #[serde(default)]
    value: String,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    reference: Option<String>,
    #[serde(default)]
    reported_at: Option<String>,
}

pub fn parse_compromised_toml(body: &str) -> Vec<CompromisedKeyEntry> {
    let feed: CompromisedTomlFeed = toml::from_str(body).unwrap_or_default();
    feed.keys
        .into_iter()
        .filter_map(|entry| {
            let key = normalize_key(&entry.value);
            if key.is_empty() {
                return None;
            }
            Some(CompromisedKeyEntry {
                key,
                reason: entry.reason,
                reference: entry.reference,
                reported_at: entry.reported_at,
            })
        })
        .collect()
}

pub fn parse_compromised_txt(body: &str) -> Vec<CompromisedKeyEntry> {
    let mut entries = Vec::new();
    for raw_line in body.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, '#');
        let head = parts.next().unwrap_or_default().trim();
        if head.is_empty() {
            continue;
        }
        let key = normalize_key(head.split_whitespace().next().unwrap_or_default());
        if key.is_empty() {
            continue;
        }
        let reason = parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        entries.push(CompromisedKeyEntry {
            key,
            reason,
            reference: None,
            reported_at: None,
        });
    }
    entries
}

pub fn parse_compromised_feed(body: &str, source_hint: &str) -> Vec<CompromisedKeyEntry> {
    let lower = source_hint.to_ascii_lowercase();
    if lower.ends_with(".txt") {
        parse_compromised_txt(body)
    } else {
        parse_compromised_toml(body)
    }
}

pub fn trusted_keys_by_scope(home: &Path) -> PrayResult<BTreeMap<String, BTreeSet<String>>> {
    let path = super::policy::trust_policy_path(home);
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let text = fs::read_to_string(&path)?;
    let policy: super::policy::ClientTrustPolicy =
        toml::from_str(&text).map_err(|error| PrayError::Parse {
            kind: "client trust policy",
            message: error.to_string(),
        })?;

    let mut output: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for key in policy.default.allowed_signing_keys {
        let normalized = normalize_key(&key);
        if !normalized.is_empty() {
            output
                .entry(normalized)
                .or_default()
                .insert("global/default".to_string());
        }
    }
    for rule in policy.rules {
        let scope = format!(
            "local:{}",
            rule.match_prefix.unwrap_or_else(|| "-".to_string())
        );
        for key in rule.allowed_signing_keys {
            let normalized = normalize_key(&key);
            if !normalized.is_empty() {
                output.entry(normalized).or_default().insert(scope.clone());
            }
        }
    }
    Ok(output)
}

pub fn check_compromised_keys(
    home: &Path,
    entries: &[CompromisedKeyEntry],
) -> PrayResult<Vec<(String, BTreeSet<String>, Vec<CompromisedKeyEntry>)>> {
    let trusted = trusted_keys_by_scope(home)?;
    if trusted.is_empty() {
        return Ok(Vec::new());
    }

    let mut compromised_by_key: BTreeMap<String, Vec<CompromisedKeyEntry>> = BTreeMap::new();
    for entry in entries {
        compromised_by_key
            .entry(entry.key.clone())
            .or_default()
            .push(entry.clone());
    }

    let mut hits = Vec::new();
    for (key, scopes) in trusted {
        let Some(matches) = compromised_by_key.get(&key) else {
            continue;
        };
        hits.push((key, scopes, matches.clone()));
    }
    Ok(hits)
}

fn normalize_key(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_toml_and_txt_feeds() {
        let toml_body = r#"
[[keys]]
value = "sha256:abc"
reason = "exposure"
"#;
        let entries = parse_compromised_toml(toml_body);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "SHA256:ABC");

        let txt_body = "sha256:def # leaked\n";
        let txt_entries = parse_compromised_txt(txt_body);
        assert_eq!(txt_entries.len(), 1);
        assert_eq!(txt_entries[0].key, "SHA256:DEF");
    }
}
