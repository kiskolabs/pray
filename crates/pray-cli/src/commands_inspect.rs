use crate::apply_report::{
    build_materialization_preview, print_materialization_report, MaterializationMode,
};
use crate::cache_clean::clean_unused_registry_cache;
use crate::commands_materialize::resolve_project_for_materialization;
use crate::commands_update::{
    constraint_preview_options, preview_remote_updates, remote_preview_options,
};
use crate::lockfile_ops::build_lockfile;
use crate::materialize::remove_path_if_exists;
use crate::project_paths::{
    load_manifest, locked_package, lockfile_path, manifest_path, resolve_project,
    resolve_project_with_options, workspace_root,
};
use crate::update_report::{print_constraint_blocked_packages, print_update_summary};
use pray_core::lockfile::read_lockfile;
use pray_core::registry::version_is_greater_than;
use pray_core::render::{layout_rendered_targets, render_project};
use pray_core::resolve_context::ResolveOptions;
use pray_core::{PrayError, PrayResult};

pub(crate) fn manifest_command() -> PrayResult<()> {
    let manifest = load_manifest()?;
    let json = serde_json::to_string_pretty(&manifest.canonicalized())
        .map_err(|error| PrayError::Manifest(error.to_string()))?;
    println!("{json}");
    Ok(())
}

pub(crate) fn plan_command(remote: bool) -> PrayResult<()> {
    let options = if remote {
        remote_preview_options()
    } else {
        ResolveOptions::default()
    };
    let project = if remote {
        resolve_project_with_options(&manifest_path(), &options)?
    } else {
        resolve_project_for_materialization(&options, false, false)?
    };
    let rendered = render_project(&project)?;
    let laid_out = layout_rendered_targets(&project, &rendered)?;
    let lockfile = build_lockfile(&project, &laid_out)?;
    let previous_lockfile = read_lockfile(&lockfile_path()).ok();
    let preview = build_materialization_preview(
        &project,
        &laid_out,
        &lockfile,
        &lockfile_path(),
        previous_lockfile.as_ref(),
    )?;
    print_materialization_report(&preview, MaterializationMode::Plan);
    Ok(())
}

pub(crate) fn clean_command(unused: bool) -> PrayResult<()> {
    let project_root = workspace_root();
    if unused {
        return clean_unused_registry_cache(&project_root);
    }
    remove_path_if_exists(&project_root.join(".pray/cache"))?;
    remove_path_if_exists(&project_root.join(".pray/vendor"))?;
    remove_path_if_exists(&project_root.join(".pray/state.json"))?;
    Ok(())
}

pub(crate) fn tree_command() -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let package_map: std::collections::BTreeMap<String, &pray_core::resolve::ResolvedPackage> =
        project
            .packages
            .iter()
            .map(|package| (package.declaration.name.clone(), package))
            .collect();
    let mut lines = vec!["Dependency tree".to_string()];
    for package in &project.packages {
        let mut ancestry = std::collections::BTreeSet::new();
        render_tree_node(package, &package_map, 0, &mut ancestry, &mut lines);
    }
    println!("{}", lines.join("\n"));
    Ok(())
}

pub(crate) fn list_command() -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let mut lines = vec!["Package list".to_string()];
    for package in &project.packages {
        lines.push(format!(
            "{} {} source={} exports={}",
            package.declaration.name,
            package.spec.version,
            package_source_summary(package),
            format_list(&package.selected_exports)
        ));
    }
    println!("{}", lines.join("\n"));
    Ok(())
}

pub(crate) fn outdated_command(remote: bool) -> PrayResult<()> {
    if remote {
        return preview_remote_updates(None, false);
    }

    let previous_lockfile = read_lockfile(&lockfile_path()).ok();
    let project = resolve_project_with_options(&manifest_path(), &constraint_preview_options())?;
    let rendered = render_project(&project)?;
    let laid_out = layout_rendered_targets(&project, &rendered)?;
    let latest_lockfile = build_lockfile(&project, &laid_out)?;
    let mut reported = print_update_summary(
        previous_lockfile.as_ref(),
        &latest_lockfile,
        None,
        &project,
        "Outdated packages",
    )?;
    reported |= print_constraint_blocked_packages(&project, "Outdated packages", !reported)?;
    if !reported {
        println!("Outdated packages");
        println!("All packages up to date");
    }
    Ok(())
}

pub(crate) fn explain_command(package_name: String) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let lockfile = read_lockfile(&lockfile_path()).ok();
    let package = project
        .packages
        .iter()
        .find(|package| package.declaration.name == package_name)
        .ok_or_else(|| PrayError::Resolution(format!("package {package_name} not found")))?;
    let lockfile_package = lockfile
        .as_ref()
        .and_then(|lockfile| locked_package(lockfile, package));

    let mut lines = vec!["Package explanation".to_string()];
    lines.push(format!("name: {}", package.declaration.name));
    lines.push(format!("constraint: {}", package.declaration.constraint));
    lines.push(format!("resolved version: {}", package.spec.version));
    if let Some(registry_latest_version) = &package.registry_latest_version {
        lines.push(format!("registry latest: {registry_latest_version}"));
        if version_is_greater_than(registry_latest_version, &package.spec.version)? {
            lines.push(format!(
                "constraint blocks upgrade: {} allows up to {}, registry has {registry_latest_version}",
                package.declaration.constraint, package.spec.version
            ));
        }
    }
    lines.push(format!("source: {}", package_source_summary(package)));
    lines.push(format!(
        "exports: {}",
        format_list(&package.selected_exports)
    ));
    lines.push(format!(
        "dependencies: {}",
        format_list(
            &package
                .spec
                .dependencies
                .iter()
                .map(|dependency| dependency.name.clone())
                .collect::<Vec<_>>()
        )
    ));
    lines.push(format!("tree hash: {}", package.tree_hash));
    lines.push(format!("artifact hash: {}", package.artifact_hash));

    match lockfile_package {
        Some(record) => {
            lines.push(format!("lockfile version: {}", record.version));
            lines.push(format!("lockfile path: {}", record.path));
            lines.push(format!(
                "lockfile exports: {}",
                format_list(&record.exports)
            ));
        }
        None => lines.push("lockfile record: missing".to_string()),
    }

    println!("{}", lines.join("\n"));
    Ok(())
}

fn render_tree_node(
    package: &pray_core::resolve::ResolvedPackage,
    package_map: &std::collections::BTreeMap<String, &pray_core::resolve::ResolvedPackage>,
    depth: usize,
    ancestry: &mut std::collections::BTreeSet<String>,
    lines: &mut Vec<String>,
) {
    let indent = "  ".repeat(depth);
    lines.push(format!(
        "{indent}{} {}",
        package.declaration.name, package.spec.version
    ));
    if !ancestry.insert(package.declaration.name.clone()) {
        return;
    }

    for dependency in &package.spec.dependencies {
        if let Some(resolved) = package_map.get(&dependency.name) {
            if ancestry.contains(&resolved.declaration.name) {
                lines.push(format!(
                    "{}  {} {} (cycle)",
                    indent, resolved.declaration.name, resolved.spec.version
                ));
            } else {
                render_tree_node(resolved, package_map, depth + 1, ancestry, lines);
            }
        } else {
            lines.push(format!(
                "{}  {} {} (unresolved)",
                indent, dependency.name, dependency.constraint
            ));
        }
    }

    ancestry.remove(&package.declaration.name);
}

fn package_source_summary(package: &pray_core::resolve::ResolvedPackage) -> String {
    package
        .declaration
        .path
        .as_ref()
        .map(|path| format!("path:{path}"))
        .or_else(|| {
            package
                .declaration
                .source
                .as_ref()
                .map(|source| format!("source:{source}"))
        })
        .unwrap_or_else(|| format!("root:{}", package.root.display()))
}

fn format_list(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}
