use crate::registry::{RegistryIndex, RegistryPackageMetadata};
use crate::registry_http::{http_get, join_url};
use crate::{PrayError, PrayResult};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrySearchHit {
    pub name: String,
    pub summary: Option<String>,
}

pub fn search_registry_index_names(index: &RegistryIndex, query: &str) -> Vec<String> {
    let needle = query.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return index.packages.clone();
    }
    let mut matches: Vec<String> = index
        .packages
        .iter()
        .filter(|name| name.to_ascii_lowercase().contains(&needle))
        .cloned()
        .collect();
    matches.sort();
    matches
}

pub fn latest_non_yanked_summary(metadata: &RegistryPackageMetadata) -> Option<String> {
    let mut best: Option<&crate::registry::RegistryPackageVersion> = None;
    for version in &metadata.versions {
        if version.yanked {
            continue;
        }
        match best {
            Some(existing)
                if crate::registry::version_is_greater_than(
                    &version.version,
                    &existing.version,
                )
                .unwrap_or(false) =>
            {
                best = Some(version);
            }
            None => best = Some(version),
            _ => {}
        }
    }
    best.and_then(|version| {
        version
            .derived_metadata
            .as_ref()
            .map(|derived| derived.summary.clone())
            .filter(|summary| !summary.trim().is_empty())
    })
}

pub fn search_local_registry(
    root: &Path,
    query: &str,
    include_summary: bool,
) -> PrayResult<Vec<RegistrySearchHit>> {
    let index_path = root.join("v1/index.json");
    let index_text = fs::read_to_string(&index_path).map_err(|error| {
        PrayError::Resolution(format!("failed to read {}: {error}", index_path.display()))
    })?;
    let index: RegistryIndex =
        serde_json::from_str(&index_text).map_err(|error| PrayError::Parse {
            kind: "registry index",
            message: error.to_string(),
        })?;
    let names = search_registry_index_names(&index, query);
    let mut hits = Vec::with_capacity(names.len());
    for name in names {
        let summary = if include_summary {
            let metadata_path = root.join(format!("v1/packages/{name}.json"));
            if metadata_path.is_file() {
                let text = fs::read_to_string(&metadata_path)?;
                let metadata: RegistryPackageMetadata =
                    serde_json::from_str(&text).map_err(|error| PrayError::Parse {
                        kind: "registry metadata",
                        message: error.to_string(),
                    })?;
                latest_non_yanked_summary(&metadata)
            } else {
                None
            }
        } else {
            None
        };
        hits.push(RegistrySearchHit { name, summary });
    }
    Ok(hits)
}

pub fn search_http_registry(
    source_url: &str,
    query: &str,
    include_summary: bool,
) -> PrayResult<Vec<RegistrySearchHit>> {
    let index_url = join_url(source_url, "v1/index.json");
    let index_bytes = http_get(&index_url)?;
    let index: RegistryIndex =
        serde_json::from_slice(&index_bytes).map_err(|error| PrayError::Parse {
            kind: "registry index",
            message: error.to_string(),
        })?;
    let names = search_registry_index_names(&index, query);
    let mut hits = Vec::with_capacity(names.len());
    for name in names {
        let summary = if include_summary {
            let metadata_url = join_url(source_url, &format!("v1/packages/{name}.json"));
            match http_get(&metadata_url) {
                Ok(bytes) => {
                    let metadata: RegistryPackageMetadata = serde_json::from_slice(&bytes)
                        .map_err(|error| PrayError::Parse {
                            kind: "registry metadata",
                            message: error.to_string(),
                        })?;
                    latest_non_yanked_summary(&metadata)
                }
                Err(_) => None,
            }
        } else {
            None
        };
        hits.push(RegistrySearchHit { name, summary });
    }
    Ok(hits)
}
