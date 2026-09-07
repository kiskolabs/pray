use crate::apply_report::{
    build_materialization_preview, print_materialization_report, MaterializationMode,
};
use crate::lockfile_ops::build_lockfile;
use crate::project_paths::{
    lockfile_path, manifest_path, resolve_project, resolve_project_with_options,
};
use crate::update_report::{
    merge_selected_package_update, print_constraint_blocked_packages, print_update_json_report,
    print_update_summary,
};
use pray_core::lockfile::{read_lockfile, write_lockfile, write_lockfile_if_changed};
use pray_core::manifest::{parse_manifest, read_manifest_text};
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

#[path = "commands_update_latest.rs"]
mod latest;
use latest::update_latest_command;

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

    let options = update_resolve_options(package.as_deref());
    let project = resolve_project_with_options(&manifest_path, &options)?;
    write_update(
        project,
        package,
        json,
        manifest_constraint_updates,
        None,
        false,
    )
}

fn update_resolve_options(package: Option<&str>) -> ResolveOptions {
    let mut options = ResolveOptions {
        refresh_source_revisions: true,
        ignore_locked_versions: package.is_none(),
        ..ResolveOptions::default()
    };
    if let Some(package) = package {
        options.unlocked_packages.insert(package.to_owned());
    }
    options
}

fn write_update(
    project: pray_core::resolve::ResolvedProject,
    package: Option<String>,
    json: bool,
    manifest_constraint_updates: Vec<serde_json::Value>,
    manifest_update: Option<String>,
    dry_run: bool,
) -> PrayResult<()> {
    let previous_lockfile = read_lockfile(&lockfile_path()).ok();
    let rendered = render_project(&project)?;
    let laid_out = layout_rendered_targets(&project, &rendered)?;
    let updated_lockfile = build_lockfile(&project, &laid_out)?;
    let merged_lockfile = match (previous_lockfile.as_ref(), package.as_deref()) {
        (Some(previous), Some(name)) => {
            merge_selected_package_update(previous, &updated_lockfile, name)
        }
        _ => updated_lockfile,
    };
    let preview = build_materialization_preview(
        &project,
        &laid_out,
        &merged_lockfile,
        &lockfile_path(),
        previous_lockfile.as_ref(),
    )?;
    if dry_run {
        print_materialization_report(&preview, MaterializationMode::Plan);
        return Ok(());
    }
    write_rendered_targets_with_previous_lockfile(&project, &rendered, previous_lockfile.as_ref())?;
    if let Some(text) = manifest_update {
        fs::write(&project.manifest_path, text)?;
    }
    write_lockfile_if_changed(&lockfile_path(), &merged_lockfile)?;
    if json {
        print_update_json_report(
            &manifest_constraint_updates,
            Some(&preview),
            previous_lockfile.as_ref(),
            &merged_lockfile,
            package.as_deref(),
            &project,
        )?;
    } else {
        print_materialization_report(&preview, MaterializationMode::Install);
        let reported = print_update_summary(
            previous_lockfile.as_ref(),
            &merged_lockfile,
            package.as_deref(),
            &project,
            "Update summary",
        )?;
        print_constraint_blocked_packages(&project, "Update summary", !reported)?;
    }
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
    write_rendered_targets_with_previous_lockfile(&project, &rendered, Some(&previous_lockfile))?;
    write_lockfile(&lockfile_path(), &merged_lockfile)?;
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
