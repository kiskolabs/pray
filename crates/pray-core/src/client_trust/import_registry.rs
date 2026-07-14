use std::path::{Path, PathBuf};

use crate::registry::fetch_optional_distribution_bytes;
use crate::ssh_client::{is_pray_ssh_url, parse_pray_ssh_url, with_pray_ssh_session};
use crate::ssh_identity::normalize_identity;
use crate::ssh_publishers::{read_ssh_publishers, SshPublisherConfig};
use crate::{PrayError, PrayResult};

use super::policy::{
    append_missing_host_keys, append_missing_publishers, load_policy_or_default,
    mutable_rule_for_match_prefix, save_policy,
};
use super::ssh_host::fetch_host_key_fingerprints;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRegistryResult {
    pub publishers_added: usize,
    pub host_keys_added: usize,
}

pub fn import_registry_trust(
    home: &Path,
    source_url: &str,
    match_prefix: Option<&str>,
    include_host_key: bool,
) -> PrayResult<ImportRegistryResult> {
    let prefix = match_prefix.unwrap_or(source_url);
    let Some(config) = fetch_ssh_publishers(source_url)? else {
        return Err(PrayError::Unsupported(format!(
            "no v1/ssh_publishers.json found for {source_url}"
        )));
    };
    let publisher_fingerprints = publisher_fingerprints(&config);
    if publisher_fingerprints.is_empty() {
        return Err(PrayError::Unsupported(format!(
            "v1/ssh_publishers.json for {source_url} lists no publisher fingerprints"
        )));
    }

    let mut host_keys = Vec::new();
    if include_host_key && is_pray_ssh_url(source_url) {
        let target = parse_pray_ssh_url(source_url)?;
        if target.host != "stdio-host" {
            host_keys = fetch_host_key_fingerprints(&target.host, target.port)?;
        }
    }

    let mut policy = load_policy_or_default(home)?;
    let rule = mutable_rule_for_match_prefix(&mut policy, prefix);
    let publishers_added = append_missing_publishers(rule, &publisher_fingerprints);
    let host_keys_added = append_missing_host_keys(rule, &host_keys);
    save_policy(home, &policy)?;

    Ok(ImportRegistryResult {
        publishers_added,
        host_keys_added,
    })
}

fn publisher_fingerprints(config: &SshPublisherConfig) -> Vec<String> {
    config
        .publishers
        .iter()
        .map(|entry| normalize_identity(&entry.fingerprint))
        .filter(|fingerprint| !fingerprint.is_empty())
        .collect()
}

pub fn fetch_ssh_publishers(source_url: &str) -> PrayResult<Option<SshPublisherConfig>> {
    if let Some(root) = local_distribution_root(source_url) {
        return read_ssh_publishers(&root);
    }
    if is_pray_ssh_url(source_url) {
        return with_pray_ssh_session(source_url, |session| {
            use serde_json::json;
            match session.call_bytes("artifact.get", json!({ "path": "v1/ssh_publishers.json" })) {
                Ok(bytes) => {
                    let config: SshPublisherConfig =
                        serde_json::from_slice(&bytes).map_err(|error| PrayError::Parse {
                            kind: "ssh publishers",
                            message: error.to_string(),
                        })?;
                    Ok(Some(config))
                }
                Err(PrayError::Resolution(message))
                    if message.contains("404") || message.contains("not found") =>
                {
                    Ok(None)
                }
                Err(error) => Err(error),
            }
        });
    }
    if source_url.starts_with("http://") || source_url.starts_with("https://") {
        let Some(bytes) = fetch_optional_distribution_bytes(source_url, "v1/ssh_publishers.json")?
        else {
            return Ok(None);
        };
        let config: SshPublisherConfig =
            serde_json::from_slice(&bytes).map_err(|error| PrayError::Parse {
                kind: "ssh publishers",
                message: error.to_string(),
            })?;
        return Ok(Some(config));
    }

    Err(PrayError::Unsupported(format!(
        "unsupported registry source for import: {source_url}"
    )))
}

fn local_distribution_root(source_url: &str) -> Option<PathBuf> {
    let path = if let Some(path) = source_url.strip_prefix("file://") {
        PathBuf::from(path)
    } else {
        PathBuf::from(source_url)
    };
    if path.is_dir() {
        Some(path)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh_publishers::SshPublisherEntry;
    use std::fs;

    #[test]
    fn publisher_fingerprints_normalize_entries() {
        let config = SshPublisherConfig {
            publishers: vec![SshPublisherEntry {
                fingerprint: "sha256:abc".to_string(),
                id: "team-ci".to_string(),
                push: true,
            }],
        };
        assert_eq!(
            publisher_fingerprints(&config),
            vec!["SHA256:ABC".to_string()]
        );
    }

    #[test]
    fn import_registry_reads_local_publishers_file() {
        let home =
            std::env::temp_dir().join(format!("pray-import-registry-home-{}", std::process::id()));
        let root =
            std::env::temp_dir().join(format!("pray-import-registry-root-{}", std::process::id()));
        let _ = fs::remove_dir_all(&home);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("v1")).expect("v1");
        fs::write(
            root.join("v1/ssh_publishers.json"),
            r#"{"publishers":[{"fingerprint":"SHA256:deadbeef","id":"team-ci","push":true}]}"#,
        )
        .expect("publishers");

        let result = import_registry_trust(&home, root.to_str().expect("utf8"), None, false)
            .expect("import");
        assert_eq!(result.publishers_added, 1);
        assert_eq!(result.host_keys_added, 0);

        let policy = super::super::policy::load_policy(&home)
            .expect("load")
            .expect("policy");
        let rule = policy
            .rules
            .iter()
            .find(|rule| rule.match_prefix.as_deref() == Some(root.to_str().expect("utf8")))
            .expect("rule");
        assert_eq!(rule.allowed_publishers, vec!["SHA256:DEADBEEF".to_string()]);

        let _ = fs::remove_dir_all(&home);
        let _ = fs::remove_dir_all(&root);
    }
}
