use crate::project_paths::lockfile_path;
use pray_core::lockfile::read_lockfile;
use pray_core::manifest::Manifest;
use pray_core::package_upstream::upstream_merge_conflict_message;
use pray_core::resolve::resolve_upstream::{
    apply_path_upstream_latest_constraints, plan_path_upstream_conflicts,
    plan_path_upstream_latest_constraints, PathUpstreamLatestConstraint,
};
use pray_core::resolve::{apply_path_upstream_refreshes, ResolvedProject};
use pray_core::resolve_context::ResolveOptions;
use pray_core::{PrayError, PrayResult};
use std::path::Path;

pub(super) fn plan_latest_upstream_constraints(
    project: &ResolvedProject,
    selected: Option<&str>,
    options: &ResolveOptions,
) -> PrayResult<Vec<PathUpstreamLatestConstraint>> {
    let previous = read_lockfile(&lockfile_path()).ok();
    plan_path_upstream_latest_constraints(project, previous.as_ref(), selected, options)
}

pub(super) fn print_latest_upstream_constraints(plans: &[PathUpstreamLatestConstraint]) {
    for plan in plans {
        println!(
            "{} upstream {} -> {} (latest {})",
            plan.package_name, plan.current_constraint, plan.new_constraint, plan.latest_version
        );
    }
}

pub(super) fn latest_upstream_constraint_updates_json(
    plans: &[PathUpstreamLatestConstraint],
) -> Vec<serde_json::Value> {
    plans
        .iter()
        .map(|plan| {
            serde_json::json!({
                "name": plan.package_name,
                "from_constraint": plan.current_constraint,
                "to_constraint": plan.new_constraint,
                "latest_version": plan.latest_version,
            })
        })
        .collect()
}

pub(super) fn resolve_after_latest_upstream(
    manifest_path: &Path,
    project_root: &Path,
    manifest: Manifest,
    options: &ResolveOptions,
    selected: Option<&str>,
    plans: &[PathUpstreamLatestConstraint],
    dry_run: bool,
) -> PrayResult<ResolvedProject> {
    if !dry_run {
        apply_path_upstream_latest_constraints(plans)?;
    }
    let mut candidate = pray_core::resolve::resolve_manifest_in_context(
        manifest_path,
        project_root,
        manifest.clone(),
        options,
    )?;
    let previous = read_lockfile(&lockfile_path()).ok();
    if dry_run {
        report_path_upstream_would_conflicts(&candidate, previous.as_ref(), selected, options)?;
        return Ok(candidate);
    }
    if apply_path_upstream_refreshes(&candidate, previous.as_ref(), selected, options)? {
        candidate = pray_core::resolve::resolve_manifest_in_context(
            manifest_path,
            project_root,
            manifest,
            options,
        )?;
    }
    Ok(candidate)
}

pub(crate) fn report_path_upstream_would_conflicts(
    project: &ResolvedProject,
    previous: Option<&pray_core::lockfile::Lockfile>,
    selected: Option<&str>,
    options: &ResolveOptions,
) -> PrayResult<()> {
    let conflicts = plan_path_upstream_conflicts(project, previous, selected, options)?;
    if conflicts.is_empty() {
        return Ok(());
    }
    let mut messages = Vec::new();
    for conflict in &conflicts {
        messages.push(upstream_merge_conflict_message(
            &conflict.package_name,
            &conflict.upstream_name,
            &conflict.old_version,
            &conflict.new_version,
            &conflict.paths,
        ));
        println!(
            "would-conflict {}: {}",
            conflict.package_name,
            conflict.paths.join(", ")
        );
    }
    Err(PrayError::Resolution(messages.join("\n")))
}
