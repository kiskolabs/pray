use crate::manifest::{Manifest, ManifestPackage, ManifestSource};
use crate::package_spec::PackageSpec;
use crate::resolve_context::ResolveOptions;
use crate::resolve_git_sources::{prepare_git_sources, prepare_pray_ssh_host_keys};

pub use crate::resolve_git::{
    discover_distribution_root, git_source_cache_directory, global_git_cache_ready,
};
pub use crate::resolve_git_legacy::quarantine_legacy_git_checkouts;
pub use crate::resolve_git_lookup::git_source_cached_repository;
pub use crate::resolve_git_paths::git_source_cache_directory_with_subdir;
pub use crate::resolve_git_refresh::{
    annotate_failed_git_refresh, annotate_missing_git_catalog,
    resolution_may_benefit_from_git_source_refresh,
};
pub use crate::resolve_git_sources::refresh_git_sources;
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ResolvedProject {
    pub manifest_path: PathBuf,
    pub project_root: PathBuf,
    pub manifest: Manifest,
    pub manifest_hash: String,
    pub packages: Vec<ResolvedPackage>,
    pub local_files: Vec<ResolvedLocalFile>,
    pub source_revisions: BTreeMap<String, String>,
    pub source_host_keys: BTreeMap<String, String>,
    pub environment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub declaration: ManifestPackage,
    pub root: PathBuf,
    pub spec: PackageSpec,
    pub tree_hash: String,
    pub artifact_hash: String,
    pub artifact: String,
    pub selected_exports: Vec<String>,
    pub source_checksum: String,
    pub export_bodies: BTreeMap<String, String>,
    pub skill_files: BTreeMap<String, Vec<String>>,
    pub signer_fingerprint: Option<String>,
    /// Highest non-yanked version in registry metadata when the package came from a registry source.
    pub registry_latest_version: Option<String>,
    /// True when the package was declared in Prayfile; false for transitive dependencies.
    pub explicit: bool,
    pub upstream: Option<crate::package_upstream::LockedUpstream>,
}

#[derive(Debug, Clone)]
pub struct ResolvedLocalFile {
    pub path: PathBuf,
    pub manifest_path: String,
    pub content: String,
    pub source_checksum: String,
    pub position: String,
    pub optional: bool,
}

impl ResolvedProject {
    pub fn lockfile_hash(&self) -> PrayResult<String> {
        Ok(self.manifest_hash.clone())
    }
}

pub fn project_root_from_manifest(manifest_path: &Path) -> PathBuf {
    match manifest_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

fn canonical_project_root(manifest_path: &Path) -> PrayResult<PathBuf> {
    let root = project_root_from_manifest(manifest_path);
    if root.is_absolute() {
        return Ok(root);
    }
    let cwd = std::env::current_dir().map_err(|error| {
        PrayError::Resolution(format!("failed to resolve project root from cwd: {error}"))
    })?;
    Ok(cwd.join(root))
}

pub fn resolve_project(manifest_path: &Path) -> PrayResult<ResolvedProject> {
    resolve_project_with_options(manifest_path, &ResolveOptions::default())
}

pub fn resolve_project_with_git_refresh_fallback(
    manifest_path: &Path,
    options: &ResolveOptions,
    allow_git_refresh_fallback: bool,
) -> PrayResult<ResolvedProject> {
    match resolve_project_with_options(manifest_path, options) {
        Ok(project) => Ok(project),
        Err(PrayError::Resolution(message))
            if allow_git_refresh_fallback
                && !options.offline
                && !options.refresh_source_revisions
                && resolution_may_benefit_from_git_source_refresh(&message) =>
        {
            let refreshed_options = ResolveOptions {
                refresh_source_revisions: true,
                ..options.clone()
            };
            match resolve_project_with_options(manifest_path, &refreshed_options) {
                Ok(project) => Ok(project),
                Err(error) => {
                    let lockfile_path =
                        project_root_from_manifest(manifest_path).join("Prayfile.lock");
                    Err(annotate_failed_git_refresh(&lockfile_path, error))
                }
            }
        }
        Err(error) => Err(error),
    }
}

pub fn resolve_project_with_options(
    manifest_path: &Path,
    options: &ResolveOptions,
) -> PrayResult<ResolvedProject> {
    let project_root = canonical_project_root(manifest_path)?;
    resolve_project_in_context(manifest_path, &project_root, options)
}

#[path = "resolve_project.rs"]
mod project;
pub use project::{resolve_manifest_in_context, resolve_project_in_context};

#[path = "resolve_implied_source.rs"]
mod implied_source;
pub(crate) use implied_source::implied_source_name;

#[path = "resolve_package_root.rs"]
mod package_root;
pub(crate) use package_root::{resolve_package_root, PackageRootResolution};

#[path = "resolve_upstream.rs"]
pub mod resolve_upstream;
pub use crate::resolve_git_materialize_confirm::confirm_git_source_revisions_materializable;
pub use resolve_upstream::{
    apply_path_upstream_refreshes, path_fork_drift_lines, plan_path_upstream_conflicts,
    PathUpstreamConflict,
};

#[path = "resolve_one.rs"]
mod one;

#[path = "resolve_local.rs"]
mod local;
pub use local::missing_local_embed_guidance;
pub(crate) use local::resolve_local_file;

fn source_map(sources: &[ManifestSource]) -> BTreeMap<String, ManifestSource> {
    sources
        .iter()
        .map(|source| (source.name.clone(), source.clone()))
        .collect()
}

#[cfg(test)]
#[path = "resolve_unit.rs"]
mod tests;
