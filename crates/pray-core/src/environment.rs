use crate::manifest::{Manifest, ManifestPackage};
use crate::{PrayError, PrayResult};
use std::collections::BTreeSet;

pub fn package_matches_environment(groups: &[String], environment: Option<&str>) -> bool {
    if groups.is_empty() {
        return true;
    }
    let Some(selected) = environment else {
        return false;
    };
    groups.iter().any(|group| group == selected)
}

pub fn collect_group_names(manifest: &Manifest) -> BTreeSet<String> {
    manifest
        .packages
        .iter()
        .flat_map(|package| package.groups.iter().cloned())
        .collect()
}

pub fn validate_environment(manifest: &Manifest, environment: Option<&str>) -> PrayResult<()> {
    let Some(selected) = environment else {
        return Ok(());
    };
    if selected.is_empty() {
        return Err(PrayError::Resolution(
            "environment name cannot be empty".to_string(),
        ));
    }
    let known_groups = collect_group_names(manifest);
    if known_groups.is_empty() {
        return Err(PrayError::Resolution(format!(
            "unknown environment {selected}; Prayfile defines no groups"
        )));
    }
    if !known_groups.contains(selected) {
        let mut names: Vec<String> = known_groups.into_iter().collect();
        names.sort();
        return Err(PrayError::Resolution(format!(
            "unknown environment {selected}; available groups are {}",
            names.join(", ")
        )));
    }
    Ok(())
}

pub fn packages_for_render<'a>(
    packages: &'a [crate::resolve::ResolvedPackage],
    environment: Option<&str>,
) -> Vec<&'a crate::resolve::ResolvedPackage> {
    packages
        .iter()
        .filter(|package| package_matches_environment(&package.declaration.groups, environment))
        .collect()
}

pub fn should_render_package(declaration: &ManifestPackage, environment: Option<&str>) -> bool {
    package_matches_environment(&declaration.groups, environment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ManifestPackage;

    fn package_with_groups(groups: &[&str]) -> ManifestPackage {
        ManifestPackage {
            name: "sample/base".to_string(),
            constraint: "*".to_string(),
            source: None,
            exports: Vec::new(),
            targets: Vec::new(),
            features: Vec::new(),
            groups: groups.iter().map(|group| (*group).to_string()).collect(),
            optional: false,
            path: None,
            git: None,
            tag: None,
            rev: None,
            tarball: None,
            oci: None,
        }
    }

    #[test]
    fn ungrouped_packages_always_render() {
        let package = package_with_groups(&[]);
        assert!(should_render_package(&package, None));
        assert!(should_render_package(&package, Some("development")));
    }

    #[test]
    fn grouped_packages_render_only_for_selected_environment() {
        let package = package_with_groups(&["development", "test"]);
        assert!(!should_render_package(&package, None));
        assert!(should_render_package(&package, Some("development")));
        assert!(should_render_package(&package, Some("test")));
        assert!(!should_render_package(&package, Some("production")));
    }
}
