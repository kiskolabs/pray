use crate::manifest::{ManifestPackage, ManifestSource};
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;

pub(crate) fn implied_source_name(
    declaration: &ManifestPackage,
    sources: &BTreeMap<String, ManifestSource>,
) -> PrayResult<Option<String>> {
    if let Some(name) = &declaration.source {
        return Ok(Some(name.clone()));
    }
    if let Some(namespace) = declaration.name.split_once('/').map(|(name, _)| name) {
        if sources.contains_key(namespace) {
            return Ok(Some(namespace.to_string()));
        }
    }
    if !declaration.name.contains('/') {
        let path_sources: Vec<&String> = sources
            .iter()
            .filter(|(_, source)| source.kind == "path")
            .map(|(name, _)| name)
            .collect();
        if path_sources.len() == 1 {
            return Ok(Some(path_sources[0].clone()));
        }
    }
    match sources.len() {
        0 => Ok(None),
        1 => Ok(sources.keys().next().cloned()),
        _ => Err(PrayError::Resolution(format!(
            "package {} requires source: when multiple sources are declared and the package namespace does not match a source",
            declaration.name
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(name: &str, kind: &str, url: &str) -> ManifestSource {
        ManifestSource {
            name: name.to_string(),
            kind: kind.to_string(),
            url: url.to_string(),
            subdir: None,
            rev: None,
            tag: None,
        }
    }

    fn package(name: &str) -> ManifestPackage {
        ManifestPackage {
            name: name.to_string(),
            constraint: "*".to_string(),
            ..Default::default()
        }
    }

    fn sources(entries: &[ManifestSource]) -> BTreeMap<String, ManifestSource> {
        entries
            .iter()
            .cloned()
            .map(|entry| (entry.name.clone(), entry))
            .collect()
    }

    #[test]
    fn unqualified_name_uses_the_unique_path_source() {
        let catalog = sources(&[
            source("amkisko", "git", "git+https://example.com/prayers.git"),
            source("local", "path", "prayers"),
        ]);
        let name = implied_source_name(&package("project"), &catalog).expect("source");
        assert_eq!(name.as_deref(), Some("local"));
    }

    #[test]
    fn namespace_still_matches_a_git_source() {
        let catalog = sources(&[
            source("amkisko", "git", "git+https://example.com/prayers.git"),
            source("local", "path", "prayers"),
        ]);
        let name = implied_source_name(&package("amkisko/rules"), &catalog).expect("source");
        assert_eq!(name.as_deref(), Some("amkisko"));
    }

    #[test]
    fn several_path_sources_still_require_source() {
        let catalog = sources(&[
            source("local", "path", "prayers"),
            source("vendor", "path", "vendor"),
        ]);
        let error = implied_source_name(&package("project"), &catalog).expect_err("source");
        assert!(error.to_string().contains("requires source:"));
    }

    #[test]
    fn namespaced_name_selects_a_path_source_when_several_exist() {
        let catalog = sources(&[
            source("local", "path", "prayers"),
            source("vendor", "path", "vendor"),
        ]);
        let name = implied_source_name(&package("local/project"), &catalog).expect("source");
        assert_eq!(name.as_deref(), Some("local"));
    }
}
