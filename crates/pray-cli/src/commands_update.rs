use crate::commands_materialize::install_command;
use crate::lockfile_ops::build_lockfile;
use crate::project_paths::{
    lockfile_path, manifest_path, resolve_project, resolve_project_with_options,
};
use crate::update_report::{
    merge_selected_package_update, print_constraint_blocked_packages, print_update_json_report,
    print_update_summary,
};
use pray_core::constraint::{latest_constraint_for_package, version_satisfies};
use pray_core::lockfile::{read_lockfile, write_lockfile};
use pray_core::manifest::{parse_manifest, read_manifest_text, replace_package_declaration};
use pray_core::render::{
    layout_rendered_targets, render_project, write_rendered_targets_with_previous_lockfile,
};
use pray_core::resolve_context::ResolveOptions;
use pray_core::{PrayError, PrayResult};
use std::fs;

pub(crate) fn update_command(
    package: Option<String>,
    major: bool,
    latest: bool,
    dry_run: bool,
    json: bool,
) -> PrayResult<()> {
    if major && latest {
        return Err(PrayError::Unsupported(
            "use either --major or --latest, not both".to_string(),
        ));
    }
    if major {
        if package.is_none() {
            return Err(PrayError::Unsupported(
                "major updates require a package name".to_string(),
            ));
        }
        if dry_run {
            return Err(PrayError::Unsupported(
                "major updates are not supported with --dry-run".to_string(),
            ));
        }
        return update_latest_command(package, json, false);
    }
    if latest {
        if dry_run && json {
            return Err(PrayError::Unsupported(
                "--json is not supported with --dry-run".to_string(),
            ));
        }
        return update_latest_command(package, json, dry_run);
    }

    if dry_run {
        return preview_remote_updates(package.as_deref(), json);
    }

    update_command_with_manifest_constraints(package, json, Vec::new())
}

fn update_latest_command(package: Option<String>, json: bool, dry_run: bool) -> PrayResult<()> {
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

    let mut updated_text = manifest_text;
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
        if json {
            let current_lockfile = read_lockfile(&lockfile_path()).unwrap_or_default();
            print_update_json_report(
                &manifest_constraint_updates,
                None,
                None,
                &current_lockfile,
                package.as_deref(),
                &project,
            )?;
            return Ok(());
        }
        println!("All package constraints already allow registry latest versions");
    } else if !json {
        for (name, previous_constraint, new_constraint, registry_latest_version) in
            &manifest_updates
        {
            println!(
                "Prayfile: {name} constraint {previous_constraint} -> {new_constraint} (registry latest {registry_latest_version})"
            );
        }
    }

    if !manifest_updates.is_empty() {
        parse_manifest(&updated_text)?;
    }
    if dry_run {
        return Ok(());
    }
    if !manifest_updates.is_empty() {
        fs::write(&manifest_path, updated_text)?;
    }

    update_command_with_manifest_constraints(package, json, manifest_constraint_updates)
}

fn update_command_with_manifest_constraints(
    package: Option<String>,
    json: bool,
    manifest_constraint_updates: Vec<serde_json::Value>,
) -> PrayResult<()> {
    let manifest_path = manifest_path();
    let manifest_text = read_manifest_text(&manifest_path)?;
    if let Some(package_name) = &package {
        let manifest = parse_manifest(&manifest_text)?;
        if !manifest
            .packages
            .iter()
            .any(|declaration| declaration.name == *package_name)
        {
            return Err(PrayError::Manifest(format!(
                "package {package_name} not found"
            )));
        }
    }

    let previous_lockfile = read_lockfile(&lockfile_path()).ok();
    let mut resolve_options = ResolveOptions {
        refresh_source_revisions: true,
        ..ResolveOptions::default()
    };
    if let Some(package_name) = &package {
        resolve_options
            .unlocked_packages
            .insert(package_name.clone());
    } else {
        resolve_options.ignore_locked_versions = true;
    }
    let install_preview = install_command(false, false, resolve_options.clone(), json)?;
    let updated_lockfile = read_lockfile(&lockfile_path())?;
    let refreshed_project = resolve_project_with_options(&manifest_path, &resolve_options)?;
    let merged_lockfile = if let (Some(previous_lockfile), Some(package_name)) =
        (previous_lockfile.as_ref(), package.as_deref())
    {
        merge_selected_package_update(previous_lockfile, &updated_lockfile, package_name)
    } else {
        updated_lockfile
    };
    if package.is_some() {
        write_lockfile(&lockfile_path(), &merged_lockfile)?;
    }
    if json {
        print_update_json_report(
            &manifest_constraint_updates,
            install_preview.as_ref(),
            previous_lockfile.as_ref(),
            &merged_lockfile,
            package.as_deref(),
            &refreshed_project,
        )?;
        return Ok(());
    }
    let update_reported = print_update_summary(
        previous_lockfile.as_ref(),
        &merged_lockfile,
        package.as_deref(),
        &refreshed_project,
        "Update summary",
    )?;
    let _ =
        print_constraint_blocked_packages(&refreshed_project, "Update summary", !update_reported)?;
    Ok(())
}

pub(crate) fn unlock_command(package: String) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    if !project
        .manifest
        .packages
        .iter()
        .any(|declaration| declaration.name == package)
    {
        return Err(PrayError::Manifest(format!("package {package} not found")));
    }
    let previous_lockfile = read_lockfile(&lockfile_path())?;
    let mut options = ResolveOptions::default();
    options.unlocked_packages.insert(package.clone());
    let project = resolve_project_with_options(&manifest_path(), &options)?;
    let rendered = render_project(&project)?;
    let laid_out = layout_rendered_targets(&project, &rendered)?;
    let updated_lockfile = build_lockfile(&project, &laid_out)?;
    let merged_lockfile =
        merge_selected_package_update(&previous_lockfile, &updated_lockfile, &package);
    write_lockfile(&lockfile_path(), &merged_lockfile)?;
    write_rendered_targets_with_previous_lockfile(&project, &rendered, Some(&previous_lockfile))?;
    println!("Unlocked {package}");
    Ok(())
}

pub(crate) fn constraint_preview_options() -> ResolveOptions {
    ResolveOptions {
        refresh_source_revisions: true,
        ignore_locked_versions: true,
        ..ResolveOptions::default()
    }
}

pub(crate) fn remote_preview_options() -> ResolveOptions {
    constraint_preview_options()
}

pub(crate) fn preview_remote_updates(selected_package: Option<&str>, json: bool) -> PrayResult<()> {
    if json {
        return Err(PrayError::Unsupported(
            "--json is not supported with --dry-run".to_string(),
        ));
    }
    let previous_lockfile = read_lockfile(&lockfile_path()).ok();
    let project = resolve_project_with_options(&manifest_path(), &remote_preview_options())?;
    let rendered = render_project(&project)?;
    let laid_out = layout_rendered_targets(&project, &rendered)?;
    let updated_lockfile = build_lockfile(&project, &laid_out)?;
    if print_update_summary(
        previous_lockfile.as_ref(),
        &updated_lockfile,
        selected_package,
        &project,
        "Remote update preview",
    )? {
        print_constraint_blocked_packages(&project, "Remote update preview", false)?;
        return Ok(());
    }
    if print_constraint_blocked_packages(&project, "Outdated packages", true)? {
        return Ok(());
    }
    println!("Outdated packages");
    println!("All packages up to date");
    Ok(())
}
