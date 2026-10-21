use super::super::{source_map, ResolvedPackage, ResolvedProject};
use super::input_output::{
    content_file_bytes, local_content_for_refresh, should_materialize_empty_fork,
};
use super::lock::ensure_locked_upstream_matches;
use super::resolution_context::UpstreamResolutionContext;
use crate::config::load_user_config;
use crate::lockfile::Lockfile;
use crate::package_upstream::{try_merge_content_files, LockedUpstream};
use crate::resolve_context::ResolveOptions;
use crate::resolve_git_sources::prepare_git_sources;
use crate::PrayResult;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathUpstreamConflict {
    pub package_name: String,
    pub upstream_name: String,
    pub old_version: String,
    pub new_version: String,
    pub paths: Vec<String>,
}

pub fn plan_path_upstream_conflicts(
    project: &ResolvedProject,
    previous: Option<&Lockfile>,
    selected: Option<&str>,
    options: &ResolveOptions,
) -> PrayResult<Vec<PathUpstreamConflict>> {
    let user_config = load_user_config()?;
    let git_sources = prepare_git_sources(
        &project.project_root,
        &project.manifest.sources,
        previous,
        options,
    )?;
    let sources = source_map(&project.manifest.sources);
    let mut conflicts = Vec::new();
    for package in &project.packages {
        if selected.is_some_and(|name| name != package.declaration.name) {
            continue;
        }
        if let Some(conflict) = plan_one_path_upstream_conflict(
            project,
            package,
            previous,
            &sources,
            &git_sources,
            &user_config,
            options,
        )? {
            conflicts.push(conflict);
        }
    }
    Ok(conflicts)
}

fn plan_one_path_upstream_conflict(
    project: &ResolvedProject,
    package: &ResolvedPackage,
    previous: Option<&Lockfile>,
    sources: &BTreeMap<String, crate::manifest::ManifestSource>,
    git_sources: &crate::resolve_git_sources::GitSourceSet,
    user_config: &crate::config::PrayConfig,
    options: &ResolveOptions,
) -> PrayResult<Option<PathUpstreamConflict>> {
    let Some(new_upstream) = &package.upstream else {
        return Ok(None);
    };
    if package.declaration.path.is_none() {
        return Ok(None);
    }
    let local_content = local_content_for_refresh(&package.root, &package.spec)?;
    let old_upstream = previous.and_then(|lockfile| {
        lockfile
            .package
            .iter()
            .find(|entry| entry.name == package.declaration.name)
            .and_then(|entry| entry.upstream.clone())
    });
    if should_materialize_empty_fork(&local_content, &package.spec, old_upstream.is_some()) {
        return Ok(None);
    }
    let Some(old_upstream) = old_upstream else {
        return Ok(None);
    };
    if old_upstream.version == new_upstream.version
        && old_upstream.tree_hash == new_upstream.tree_hash
    {
        return Ok(None);
    }
    let resolution_context = UpstreamResolutionContext::new(
        &project.project_root,
        sources,
        git_sources,
        user_config,
        previous,
        options,
    );
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
    match try_merge_content_files(&old_content, &new_content, &local_content) {
        Ok(_) => Ok(None),
        Err(paths) => Ok(Some(PathUpstreamConflict {
            package_name: package.declaration.name.clone(),
            upstream_name: old_upstream.name.clone(),
            old_version: old_upstream.version.clone(),
            new_version: new_upstream.version.clone(),
            paths,
        })),
    }
}
