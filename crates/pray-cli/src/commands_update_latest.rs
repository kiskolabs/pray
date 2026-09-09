use super::{constraint_preview_options, update_resolve_options, write_update};
use crate::project_paths::{manifest_path, resolve_project_with_options};
use pray_core::constraint::{latest_constraint_for_package, version_satisfies};
use pray_core::manifest::{parse_manifest, read_manifest_text, replace_package_declaration};
use pray_core::{PrayError, PrayResult};

pub(super) fn update_latest_command(
    package: Option<String>,
    json: bool,
    dry_run: bool,
) -> PrayResult<()> {
    let manifest_path = manifest_path();
    let manifest_text = read_manifest_text(&manifest_path)?;
    let preview_options = constraint_preview_options();
    let project = resolve_project_with_options(&manifest_path, &preview_options)?;

    if let Some(package_name) = &package {
        if !project
            .manifest
            .packages
            .iter()
            .any(|declaration| declaration.name == *package_name)
        {
            return Err(PrayError::Manifest(format!(
                "package {package_name} not found"
            )));
        }
    }

    let mut updated_text = manifest_text.clone();
    let mut manifest_updates = Vec::new();

    for resolved in &project.packages {
        if let Some(package_name) = &package {
            if resolved.declaration.name != *package_name {
                continue;
            }
        }
        let Some(registry_latest_version) = &resolved.registry_latest_version else {
            continue;
        };
        if version_satisfies(registry_latest_version, &resolved.declaration.constraint)? {
            continue;
        }
        let new_constraint = latest_constraint_for_package(
            &resolved.declaration.constraint,
            registry_latest_version,
        )?;
        if !version_satisfies(registry_latest_version, &new_constraint)? {
            return Err(PrayError::Resolution(format!(
                "derived constraint {new_constraint} does not admit registry latest {registry_latest_version} for {}",
                resolved.declaration.name
            )));
        }
        let mut updated_declaration = resolved.declaration.clone();
        let previous_constraint = updated_declaration.constraint.clone();
        updated_declaration.constraint = new_constraint.clone();
        updated_text = replace_package_declaration(&updated_text, &updated_declaration)?;
        manifest_updates.push((
            resolved.declaration.name.clone(),
            previous_constraint,
            new_constraint,
            registry_latest_version.clone(),
        ));
    }

    let manifest_constraint_updates: Vec<serde_json::Value> = manifest_updates
        .iter()
        .map(
            |(name, previous_constraint, new_constraint, registry_latest_version)| {
                serde_json::json!({
                    "name": name,
                    "from_constraint": previous_constraint,
                    "to_constraint": new_constraint,
                    "registry_latest_version": registry_latest_version,
                })
            },
        )
        .collect();

    if manifest_updates.is_empty() {
        if !json {
            println!("All package constraints already allow registry latest versions");
        }
    } else if !json {
        for (name, previous_constraint, new_constraint, registry_latest_version) in
            &manifest_updates
        {
            println!(
                "Prayfile: {name} constraint {previous_constraint} -> {new_constraint} (registry latest {registry_latest_version})"
            );
        }
    }

    let mut options = update_resolve_options(package.as_deref());
    options.environment = project.environment;
    let candidate = pray_core::resolve::resolve_manifest_in_context(
        &manifest_path,
        &project.project_root,
        parse_manifest(&updated_text)?,
        &options,
    )?;
    write_update(
        candidate,
        package,
        json,
        manifest_constraint_updates,
        (updated_text != manifest_text).then_some(updated_text),
        dry_run,
    )
}
