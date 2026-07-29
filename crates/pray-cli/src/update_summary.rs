use pray_core::lockfile::{LockedPackage, Lockfile};
use pray_core::PrayResult;

pub(crate) struct UpdateSummaryReport {
    pub(crate) lines: Vec<String>,
    pub(crate) updated_packages: Vec<serde_json::Value>,
}

pub(crate) fn build_update_summary(
    previous: Option<&Lockfile>,
    updated: &Lockfile,
    selected_package: Option<&str>,
    project: &pray_core::resolve::ResolvedProject,
) -> PrayResult<UpdateSummaryReport> {
    let previous_packages: std::collections::BTreeMap<&str, &LockedPackage> = previous
        .into_iter()
        .flat_map(|lockfile| lockfile.package.iter())
        .map(|package| (package.name.as_str(), package))
        .collect();
    let package_sources: std::collections::BTreeMap<&str, String> = project
        .packages
        .iter()
        .map(|package| {
            (
                package.declaration.name.as_str(),
                package_source_label(&package.declaration),
            )
        })
        .collect();
    let package_targets: std::collections::BTreeMap<&str, Vec<String>> = project
        .packages
        .iter()
        .map(|package| {
            (
                package.declaration.name.as_str(),
                package_target_names(package, project),
            )
        })
        .collect();
    let target_outputs: std::collections::BTreeMap<&str, Vec<String>> = project
        .manifest
        .targets
        .iter()
        .map(|target| (target.name.as_str(), target.outputs.clone()))
        .collect();

    let mut lines = Vec::new();
    let mut structured_updates = Vec::new();

    if let Some(previous) = previous {
        for source in &updated.source {
            let previous_revision = previous
                .source
                .iter()
                .find(|locked_source| locked_source.name == source.name)
                .and_then(|locked_source| locked_source.revision.as_deref());
            let updated_revision = source.revision.as_deref();
            if previous_revision != updated_revision {
                lines.push(format!(
                    "Updated source {} revision {} -> {}",
                    source.name,
                    previous_revision.unwrap_or("none"),
                    updated_revision.unwrap_or("none")
                ));
            }
        }
    }

    for package in &updated.package {
        if let Some(selected_package) = selected_package {
            if package.name != selected_package {
                continue;
            }
        }
        let Some(previous_package) = previous_packages.get(package.name.as_str()) else {
            lines.push(format!(
                "Updated package {} (new) -> {}",
                package.name, package.version
            ));
            continue;
        };
        let version_changed = previous_package.version != package.version;
        let artifact_changed = previous_package.artifact_hash != package.artifact_hash;
        let tree_changed = previous_package.tree_hash != package.tree_hash;
        if !version_changed && !artifact_changed && !tree_changed {
            continue;
        }

        let source = package_sources
            .get(package.name.as_str())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let exports = package.exports.clone();
        let targets = package_targets
            .get(package.name.as_str())
            .cloned()
            .unwrap_or_default();
        let rendered_files: Vec<String> = targets
            .iter()
            .flat_map(|target_name| {
                target_outputs
                    .get(target_name.as_str())
                    .into_iter()
                    .flatten()
            })
            .cloned()
            .collect();
        let dependents = package_dependents(project, package.name.as_str());

        if version_changed {
            lines.push(format!(
                "Updated package {} {} -> {}",
                package.name, previous_package.version, package.version
            ));
        } else {
            lines.push(format!(
                "Refreshed package {} at {} (registry content changed)",
                package.name, package.version
            ));
        }
        lines.push(format!("  source: {source}"));
        lines.push(format!("  exports affected: {}", join_or_none(&exports)));
        lines.push(format!("  targets affected: {}", join_or_none(&targets)));
        lines.push(format!(
            "  rendered files affected: {}",
            join_or_none(&rendered_files)
        ));
        if !dependents.is_empty() {
            lines.push(format!(
                "  dependent packages affected: {}",
                join_or_none(&dependents)
            ));
        }
        lines.push("  warnings: none".to_string());
        structured_updates.push(serde_json::json!({
            "name": package.name,
            "from_version": previous_package.version,
            "to_version": package.version,
            "artifact_hash_changed": artifact_changed,
            "tree_hash_changed": tree_changed,
            "source": source,
            "exports_affected": exports,
            "targets_affected": targets,
            "rendered_files_affected": rendered_files,
            "dependent_packages_affected": dependents,
            "warnings": [],
        }));
    }

    Ok(UpdateSummaryReport {
        lines,
        updated_packages: structured_updates,
    })
}

fn package_dependents(
    project: &pray_core::resolve::ResolvedProject,
    selected_package: &str,
) -> Vec<String> {
    project
        .packages
        .iter()
        .filter(|package| {
            package
                .spec
                .dependencies
                .iter()
                .any(|dependency| dependency.name == selected_package)
        })
        .map(|package| package.declaration.name.clone())
        .collect()
}

fn package_source_label(declaration: &pray_core::manifest::ManifestPackage) -> String {
    if let Some(path) = &declaration.path {
        return format!("path:{path}");
    }
    if let Some(source) = &declaration.source {
        return format!("source:{source}");
    }
    "default".to_string()
}

fn package_target_names(
    package: &pray_core::resolve::ResolvedPackage,
    project: &pray_core::resolve::ResolvedProject,
) -> Vec<String> {
    if !package.declaration.targets.is_empty() {
        return package.declaration.targets.clone();
    }
    project
        .manifest
        .targets
        .iter()
        .map(|target| target.name.clone())
        .collect()
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }
    values.join(", ")
}
