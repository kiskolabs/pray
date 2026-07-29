use crate::apply_report::{materialization_preview_to_json, MaterializationMode, MaterializationPreview};
use crate::update_summary::build_update_summary;
use pray_core::lockfile::Lockfile;
use pray_core::registry::version_is_greater_than;
use pray_core::resolve::ResolvedProject;
use pray_core::{PrayError, PrayResult};

pub(crate) fn merge_selected_package_update(
    previous: &Lockfile,
    updated: &Lockfile,
    selected_package: &str,
) -> Lockfile {
    let mut merged = updated.clone();
    for package in &mut merged.package {
        if package.name == selected_package {
            continue;
        }
        if let Some(previous_package) = previous
            .package
            .iter()
            .find(|locked_package| locked_package.name == package.name)
        {
            package.version = previous_package.version.clone();
        }
    }
    merged
}

pub(crate) fn constraint_blocked_packages_json(
    project: &ResolvedProject,
) -> PrayResult<Vec<serde_json::Value>> {
    let mut packages = Vec::new();
    for package in &project.packages {
        let Some(registry_latest_version) = &package.registry_latest_version else {
            continue;
        };
        if registry_latest_version == &package.spec.version {
            continue;
        }
        if version_is_greater_than(registry_latest_version, &package.spec.version)? {
            packages.push(serde_json::json!({
                "name": package.declaration.name,
                "resolved_version": package.spec.version,
                "registry_latest_version": registry_latest_version,
                "constraint": package.declaration.constraint,
            }));
        }
    }
    packages.sort_by(|left, right| {
        left["name"]
            .as_str()
            .unwrap_or_default()
            .cmp(right["name"].as_str().unwrap_or_default())
    });
    Ok(packages)
}

fn constraint_blocked_package_lines(project: &ResolvedProject) -> PrayResult<Vec<String>> {
    let mut lines = Vec::new();
    for package in &project.packages {
        let Some(registry_latest_version) = &package.registry_latest_version else {
            continue;
        };
        if registry_latest_version == &package.spec.version {
            continue;
        }
        if version_is_greater_than(registry_latest_version, &package.spec.version)? {
            lines.push(format!(
                "Available package {} {} -> {} (blocked by {})",
                package.declaration.name,
                package.spec.version,
                registry_latest_version,
                package.declaration.constraint,
            ));
        }
    }
    lines.sort();
    Ok(lines)
}

pub(crate) fn print_constraint_blocked_packages(
    project: &ResolvedProject,
    title: &str,
    print_title: bool,
) -> PrayResult<bool> {
    let lines = constraint_blocked_package_lines(project)?;
    if lines.is_empty() {
        return Ok(false);
    }
    if print_title {
        println!("{title}");
    }
    for line in lines {
        println!("{line}");
    }
    Ok(true)
}

pub(crate) fn print_update_summary(
    previous: Option<&Lockfile>,
    updated: &Lockfile,
    selected_package: Option<&str>,
    project: &pray_core::resolve::ResolvedProject,
    title: &str,
) -> PrayResult<bool> {
    let report = build_update_summary(previous, updated, selected_package, project)?;
    if report.lines.is_empty() {
        return Ok(false);
    }

    println!("{title}");
    for line in report.lines {
        println!("{line}");
    }
    Ok(true)
}

pub(crate) fn print_update_json_report(
    manifest_constraint_updates: &[serde_json::Value],
    install_preview: Option<&MaterializationPreview>,
    previous: Option<&Lockfile>,
    updated: &Lockfile,
    selected_package: Option<&str>,
    project: &ResolvedProject,
) -> PrayResult<()> {
    let summary = build_update_summary(previous, updated, selected_package, project)?;
    let constraint_blocked_packages = constraint_blocked_packages_json(project)?;
    let status = if manifest_constraint_updates.is_empty()
        && summary.updated_packages.is_empty()
        && install_preview.is_none()
        && constraint_blocked_packages.is_empty()
    {
        "up_to_date"
    } else {
        "updated"
    };
    let mut output = serde_json::json!({
        "status": status,
        "manifest_constraint_updates": manifest_constraint_updates,
        "updated_packages": summary.updated_packages,
        "constraint_blocked_packages": constraint_blocked_packages,
    });
    if let Some(preview) = install_preview {
        output["install"] = materialization_preview_to_json(preview, MaterializationMode::Install);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| PrayError::Manifest(error.to_string()))?
    );
    Ok(())
}
