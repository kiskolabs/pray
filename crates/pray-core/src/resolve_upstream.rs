use super::{source_map, ResolvedPackage, ResolvedProject};
use crate::config::load_user_config;
use crate::lockfile::Lockfile;
use crate::package_spec_render::{fork_spec_after_refresh, render_package_spec};
use crate::package_upstream::{
    is_clean_replica, try_merge_content_files, upstream_merge_conflict_message, LockedUpstream,
};
use crate::paths::find_prayspec_file;
use crate::resolve_context::ResolveOptions;
use crate::resolve_git_sources::prepare_git_sources;
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;
use std::path::Path;

#[path = "resolve_upstream_drift.rs"]
mod drift;
#[path = "resolve_upstream_io.rs"]
mod input_output;
#[path = "resolve_upstream_latest.rs"]
mod latest;
#[path = "resolve_upstream_lock.rs"]
mod lock;
#[path = "resolve_upstream_context.rs"]
mod resolution_context;

pub use drift::path_fork_drift_lines;
use input_output::{content_file_bytes, local_content_for_refresh, write_content_files};
pub use latest::{
    apply_path_upstream_latest_constraints, plan_path_upstream_latest_constraints,
    PathUpstreamLatestConstraint,
};
use lock::ensure_locked_upstream_matches;
pub(super) use lock::lock_path_upstream;
pub(super) use resolution_context::UpstreamResolutionContext;

pub fn apply_path_upstream_refreshes(
    project: &ResolvedProject,
    previous: Option<&Lockfile>,
    selected: Option<&str>,
    options: &ResolveOptions,
) -> PrayResult<bool> {
    let user_config = load_user_config()?;
    let git_sources = prepare_git_sources(
        &project.project_root,
        &project.manifest.sources,
        previous,
        options,
    )?;
    let sources = source_map(&project.manifest.sources);
    let mut changed = false;
    for package in &project.packages {
        if selected.is_some_and(|name| name != package.declaration.name) {
            continue;
        }
        if apply_one_path_upstream(
            project,
            package,
            previous,
            &sources,
            &git_sources,
            &user_config,
            options,
        )? {
            changed = true;
        }
    }
    Ok(changed)
}

fn apply_one_path_upstream(
    project: &ResolvedProject,
    package: &ResolvedPackage,
    previous: Option<&Lockfile>,
    sources: &BTreeMap<String, crate::manifest::ManifestSource>,
    git_sources: &BTreeMap<String, crate::resolve_git_sources::GitSourceCheckout>,
    user_config: &crate::config::PrayConfig,
    options: &ResolveOptions,
) -> PrayResult<bool> {
    let Some(new_upstream) = &package.upstream else {
        return Ok(false);
    };
    if package.declaration.path.is_none() {
        return Ok(false);
    }
    let local_content = local_content_for_refresh(&package.root, &package.spec)?;
    let resolution_context = UpstreamResolutionContext::new(
        &project.project_root,
        sources,
        git_sources,
        user_config,
        previous,
        options,
    );
    if local_content.is_empty() {
        return materialize_empty_path_fork(package, new_upstream, &resolution_context);
    }
    let Some(old_upstream) = previous.and_then(|lockfile| {
        lockfile
            .package
            .iter()
            .find(|entry| entry.name == package.declaration.name)
            .and_then(|entry| entry.upstream.clone())
    }) else {
        return Ok(false);
    };
    if old_upstream.version == new_upstream.version
        && old_upstream.tree_hash == new_upstream.tree_hash
    {
        return Ok(false);
    }
    let old_package = resolution_context.resolve(
        &old_upstream.name,
        &format!("= {}", old_upstream.version),
        old_upstream.source.as_deref(),
    )?;
    let resolved_old = LockedUpstream {
        name: old_upstream.name.clone(),
        version: old_package.spec.version.clone(),
        source: old_upstream.source.clone(),
        tree_hash: old_package.tree_hash.clone(),
        artifact_hash: old_package.artifact_hash.clone(),
    };
    ensure_locked_upstream_matches(&old_upstream, &resolved_old)?;
    let new_package = resolution_context.resolve(
        &new_upstream.name,
        &format!("= {}", new_upstream.version),
        new_upstream.source.as_deref(),
    )?;
    let old_content = content_file_bytes(&old_package.root, &old_package.spec)?;
    let new_content = content_file_bytes(&new_package.root, &new_package.spec)?;
    let merged = match try_merge_content_files(&old_content, &new_content, &local_content) {
        Ok(merged) => merged,
        Err(paths) => {
            return Err(PrayError::Resolution(upstream_merge_conflict_message(
                &package.declaration.name,
                &old_upstream.name,
                &old_upstream.version,
                &new_upstream.version,
                &paths,
            )));
        }
    };
    let clean = is_clean_replica(&old_content, &local_content);
    write_content_files(&package.root, &old_content, &merged)?;
    rewrite_fork_spec(
        &package.root,
        &package.spec,
        &new_package.spec,
        clean,
        merged.keys().cloned().collect(),
    )?;
    Ok(true)
}

fn materialize_empty_path_fork(
    package: &ResolvedPackage,
    new_upstream: &LockedUpstream,
    resolution_context: &UpstreamResolutionContext<'_>,
) -> PrayResult<bool> {
    let new_package = resolution_context.resolve(
        &new_upstream.name,
        &format!("= {}", new_upstream.version),
        new_upstream.source.as_deref(),
    )?;
    let new_content = content_file_bytes(&new_package.root, &new_package.spec)?;
    write_content_files(&package.root, &BTreeMap::new(), &new_content)?;
    rewrite_fork_spec(
        &package.root,
        &package.spec,
        &new_package.spec,
        true,
        new_content.keys().cloned().collect(),
    )?;
    Ok(true)
}

fn rewrite_fork_spec(
    package_root: &Path,
    local: &crate::package_spec::PackageSpec,
    new_upstream: &crate::package_spec::PackageSpec,
    clean: bool,
    merged_paths: Vec<String>,
) -> PrayResult<()> {
    let spec_path = find_prayspec_file(package_root)?;
    let updated = fork_spec_after_refresh(local, new_upstream, clean, &merged_paths);
    crate::transaction::write_file(&spec_path, render_package_spec(&updated))?;
    Ok(())
}

#[cfg(test)]
#[path = "resolve_upstream_tests.rs"]
mod tests;
