use crate::lockfile::Lockfile;
use crate::manifest::{ManifestPackage, ManifestSource};
use crate::registry::{resolve_local_registry_package_root, RegistryPackageResolution};
use crate::resolve_context::PackageResolutionContext;
use crate::resolve_git::resolve_distribution_root;
use crate::resolve_git_ensure::ensure_git_repository;
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) use crate::resolve_git_source_set::{prepare_git_sources, GitSourceSet};

#[derive(Debug, Clone)]
pub(crate) struct GitSourceCheckout {
    pub(crate) cache_directory: PathBuf,
    pub(crate) revision: String,
    pub(crate) subdir: Option<String>,
}

pub(crate) fn prepare_pray_ssh_host_keys(
    sources: &[ManifestSource],
) -> PrayResult<BTreeMap<String, String>> {
    use crate::client_trust::{effective_trust_home, gate_pray_ssh_host};
    use crate::ssh_client::parse_pray_ssh_url;

    let home = effective_trust_home()?;
    let mut host_keys = BTreeMap::new();
    for source in sources {
        if source.kind != "pray_ssh" {
            continue;
        }
        let target = parse_pray_ssh_url(&source.url)?;
        let fingerprint = gate_pray_ssh_host(&home, &source.url, &target.host, target.port)?;
        if !fingerprint.is_empty() {
            host_keys.insert(source.name.clone(), fingerprint);
        }
    }
    Ok(host_keys)
}

pub(crate) fn is_local_filesystem_source(clone_url: &str) -> bool {
    clone_url.starts_with("file://") || Path::new(clone_url).is_absolute()
}

pub(crate) fn local_git_repo_path(project_root: &Path, clone_url: &str) -> Option<PathBuf> {
    let path = clone_url_filesystem_path(project_root, clone_url);
    if path.join(".git").is_dir() {
        Some(path)
    } else {
        None
    }
}

pub(crate) fn local_git_source_root(project_root: &Path, clone_url: &str) -> Option<PathBuf> {
    let path = clone_url_filesystem_path(project_root, clone_url);
    if !path.exists() {
        return None;
    }
    crate::resolve_git::discover_distribution_root(&path)
}

fn clone_url_filesystem_path(project_root: &Path, clone_url: &str) -> PathBuf {
    let path = clone_url
        .strip_prefix("file://")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(clone_url));
    if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    }
}

pub(crate) fn pinned_revision_for_source(
    lockfile: Option<&Lockfile>,
    source: &ManifestSource,
) -> Option<String> {
    if let Some(revision) = lockfile
        .and_then(|lockfile| {
            lockfile
                .source
                .iter()
                .find(|entry| entry.name == source.name && entry.kind == "git")
        })
        .and_then(|entry| entry.revision.clone())
    {
        return Some(revision);
    }
    if source.kind != "git" {
        return None;
    }
    source.rev.clone().or_else(|| source.tag.clone())
}

pub(crate) fn resolve_git_package_root(
    project_root: &Path,
    source_name: &str,
    source_url: &str,
    git_sources: &GitSourceSet,
    declaration: &ManifestPackage,
    context: &PackageResolutionContext,
) -> PrayResult<RegistryPackageResolution> {
    let clone_url = source_url.strip_prefix("git+").unwrap_or(source_url);
    match git_sources.ensure(source_name) {
        Ok(checkout) => {
            let distribution_root =
                resolve_distribution_root(&checkout.cache_directory, checkout.subdir.as_deref())?;
            let source_key = if checkout.revision.is_empty() {
                clone_url.to_string()
            } else {
                format!("{}@{}", clone_url, checkout.revision)
            };
            resolve_local_registry_package_root(
                project_root,
                &source_key,
                &distribution_root,
                declaration,
                context,
            )
            .map_err(|error| {
                crate::resolve_git_refresh::annotate_missing_git_catalog(
                    error,
                    &declaration.name,
                    source_name,
                    &checkout.revision,
                )
            })
        }
        Err(error) => {
            if let Some(source_root) = local_git_source_root(project_root, clone_url) {
                resolve_local_registry_package_root(
                    project_root,
                    clone_url,
                    &source_root,
                    declaration,
                    context,
                )
            } else {
                Err(error)
            }
        }
    }
}

pub fn refresh_git_sources(manifest_path: &Path) -> PrayResult<()> {
    let project_root = project_root_for_manifest(manifest_path)?;
    let manifest_text = crate::manifest::read_manifest_text(manifest_path)?;
    let manifest = crate::manifest::parse_manifest(&manifest_text)?;
    for source in &manifest.sources {
        if source.kind != "git" {
            continue;
        }
        let clone_url = source.url.strip_prefix("git+").unwrap_or(&source.url);
        if is_local_filesystem_source(clone_url)
            && local_git_repo_path(&project_root, clone_url).is_none()
        {
            continue;
        }
        let _ = ensure_git_repository(
            &project_root,
            clone_url,
            true,
            None,
            source.subdir.as_deref(),
        )?;
    }
    Ok(())
}

fn project_root_for_manifest(manifest_path: &Path) -> PrayResult<PathBuf> {
    let root = match manifest_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    };
    if root.is_absolute() {
        return Ok(root);
    }
    let cwd = std::env::current_dir().map_err(|error| {
        PrayError::Resolution(format!("failed to resolve project root from cwd: {error}"))
    })?;
    Ok(cwd.join(root))
}
