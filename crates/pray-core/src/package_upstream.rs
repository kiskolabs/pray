use crate::literal::{parse_literal, split_top_level, LiteralValue};
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageUpstream {
    pub name: String,
    pub constraint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockedUpstream {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub tree_hash: String,
    pub artifact_hash: String,
}

pub fn parse_upstream(rest: &str) -> PrayResult<PackageUpstream> {
    let mut positional = Vec::new();
    for segment in split_top_level(rest.trim().trim_end_matches(','), ',') {
        if !segment.is_empty() {
            positional.push(parse_literal(&segment)?);
        }
    }
    let name = string_from_value(positional.first().ok_or_else(|| PrayError::Parse {
        kind: "prayspec",
        message: "missing upstream name".to_string(),
    })?)?;
    let constraint = positional
        .get(1)
        .map(string_from_value)
        .transpose()?
        .unwrap_or_else(|| "*".to_string());
    if name.is_empty() {
        return Err(PrayError::Parse {
            kind: "prayspec",
            message: "missing upstream name".to_string(),
        });
    }
    Ok(PackageUpstream { name, constraint })
}

pub fn is_identity_path(path: &str) -> bool {
    Path::new(path).extension().and_then(|value| value.to_str()) == Some("prayspec")
}

pub fn content_paths(files: &[String]) -> Vec<String> {
    files
        .iter()
        .filter(|path| !is_identity_path(path))
        .cloned()
        .collect()
}

pub fn is_clean_replica(
    old_content: &BTreeMap<String, Vec<u8>>,
    local_content: &BTreeMap<String, Vec<u8>>,
) -> bool {
    local_content == old_content
}

pub fn merge_content_files(
    old_content: &BTreeMap<String, Vec<u8>>,
    new_content: &BTreeMap<String, Vec<u8>>,
    local_content: &BTreeMap<String, Vec<u8>>,
) -> PrayResult<BTreeMap<String, Vec<u8>>> {
    if is_clean_replica(old_content, local_content) {
        return Ok(new_content.clone());
    }
    let mut paths = BTreeMap::new();
    for path in old_content
        .keys()
        .chain(new_content.keys())
        .chain(local_content.keys())
    {
        paths.insert(path.clone(), ());
    }
    let mut merged = BTreeMap::new();
    for path in paths.keys() {
        let old = old_content.get(path);
        let new = new_content.get(path);
        let local = local_content.get(path);
        match (old, new, local) {
            (_, Some(new_bytes), Some(local_bytes)) if local_bytes == new_bytes => {
                merged.insert(path.clone(), new_bytes.clone());
            }
            (Some(old_bytes), Some(new_bytes), Some(local_bytes)) if local_bytes == old_bytes => {
                merged.insert(path.clone(), new_bytes.clone());
            }
            (Some(old_bytes), Some(new_bytes), Some(local_bytes)) if old_bytes == new_bytes => {
                merged.insert(path.clone(), local_bytes.clone());
            }
            (Some(old_bytes), None, Some(local_bytes)) if local_bytes == old_bytes => {}
            (Some(old_bytes), Some(new_bytes), None) if old_bytes == new_bytes => {}
            (None, Some(new_bytes), None) => {
                merged.insert(path.clone(), new_bytes.clone());
            }
            (None, None, Some(local_bytes)) => {
                merged.insert(path.clone(), local_bytes.clone());
            }
            (Some(_), Some(new_bytes), None) => {
                merged.insert(path.clone(), new_bytes.clone());
            }
            _ => {
                return Err(PrayError::Resolution(format!(
                    "upstream merge conflict in {path}"
                )));
            }
        }
    }
    Ok(merged)
}

pub fn next_upstream_constraint(current: &str, new_version: &str) -> String {
    let trimmed = current.trim();
    if trimmed == "*"
        || trimmed.starts_with("~>")
        || trimmed.starts_with('^')
        || trimmed.starts_with(">=")
        || trimmed.starts_with('>')
        || trimmed.starts_with("<=")
        || trimmed.starts_with('<')
    {
        return current.to_string();
    }
    format!("= {new_version}")
}

fn string_from_value(value: &LiteralValue) -> PrayResult<String> {
    value
        .as_string()
        .map(str::to_string)
        .ok_or_else(|| PrayError::Parse {
            kind: "prayspec",
            message: format!("expected string-like literal, found {:?}", value),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_replica_takes_new_tree() {
        let old = BTreeMap::from([("exports/a.md".to_string(), b"old".to_vec())]);
        let new = BTreeMap::from([("exports/a.md".to_string(), b"new".to_vec())]);
        let merged = merge_content_files(&old, &new, &old).expect("merge");
        assert_eq!(
            merged.get("exports/a.md").map(Vec::as_slice),
            Some(&b"new"[..])
        );
    }

    #[test]
    fn local_edit_against_upstream_change_conflicts() {
        let old = BTreeMap::from([("exports/a.md".to_string(), b"old".to_vec())]);
        let new = BTreeMap::from([("exports/a.md".to_string(), b"new".to_vec())]);
        let local = BTreeMap::from([("exports/a.md".to_string(), b"edit".to_vec())]);
        let error = merge_content_files(&old, &new, &local).expect_err("conflict");
        assert!(error.to_string().contains("exports/a.md"));
    }

    #[test]
    fn local_edit_kept_when_upstream_unchanged() {
        let old = BTreeMap::from([("exports/a.md".to_string(), b"old".to_vec())]);
        let local = BTreeMap::from([("exports/a.md".to_string(), b"edit".to_vec())]);
        let merged = merge_content_files(&old, &old, &local).expect("merge");
        assert_eq!(
            merged.get("exports/a.md").map(Vec::as_slice),
            Some(&b"edit"[..])
        );
    }

    #[test]
    fn exact_constraint_rewrites_to_new_version() {
        assert_eq!(next_upstream_constraint("= 1.4.3", "1.4.4"), "= 1.4.4");
        assert_eq!(next_upstream_constraint("1.4.3", "1.4.4"), "= 1.4.4");
        assert_eq!(next_upstream_constraint("~> 1.4", "1.4.4"), "~> 1.4");
    }
}
