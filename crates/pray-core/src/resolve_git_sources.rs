use crate::lockfile::Lockfile;
use crate::manifest::{ManifestPackage, ManifestSource};
use crate::registry::{resolve_local_registry_package_root, RegistryPackageResolution};
use crate::resolve_context::{PackageResolutionContext, ResolveOptions};
use crate::resolve_git::{
    ensure_git_repository, local_git_source_root, resolve_distribution_root,
};
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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

pub(crate) fn prepare_git_sources(
    project_root: &Path,
    sources: &[ManifestSource],
    lockfile: Option<&Lockfile>,
    options: &ResolveOptions,
) -> PrayResult<BTreeMap<String, GitSourceCheckout>> {
    let mut git_sources = BTreeMap::new();
    for source in sources {
        if source.kind != "git" {
            continue;
        }
        let clone_url = source.url.strip_prefix("git+").unwrap_or(&source.url);
        let pinned_revision = if options.refresh_source_revisions {
            None
        } else {
            pinned_revision_for_source(lockfile, source)
        };
        let refresh = options.refresh_source_revisions;
        if is_local_filesystem_source(clone_url) && local_git_repo_path(clone_url).is_none() {
            if let Some(source_root) = local_git_source_root(clone_url) {
                git_sources.insert(
                    source.name.clone(),
                    GitSourceCheckout {
                        cache_directory: source_root,
                        revision: String::new(),
                        subdir: source.subdir.clone(),
                    },
                );
            }
            continue;
        }
        let (cache_directory, revision) = ensure_git_repository(
            project_root,
            clone_url,
            refresh,
            pinned_revision.as_deref(),
            source.subdir.as_deref(),
        )?;
        git_sources.insert(
            source.name.clone(),
            GitSourceCheckout {
                cache_directory,
                revision,
                subdir: source.subdir.clone(),
            },
        );
    }
    Ok(git_sources)
}

pub(crate) fn is_local_filesystem_source(clone_url: &str) -> bool {
    clone_url.starts_with("file://") || Path::new(clone_url).is_absolute()
}

pub(crate) fn local_git_repo_path(clone_url: &str) -> Option<PathBuf> {
    let path = if let Some(path) = clone_url.strip_prefix("file://") {
        PathBuf::from(path)
    } else {
        PathBuf::from(clone_url)
    };
    if path.join(".git").is_dir() {
        Some(path)
    } else {
        None
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
    git_sources: &BTreeMap<String, GitSourceCheckout>,
    declaration: &ManifestPackage,
    context: &PackageResolutionContext,
) -> PrayResult<RegistryPackageResolution> {
    let clone_url = source_url.strip_prefix("git+").unwrap_or(source_url);
    if let Some(checkout) = git_sources.get(source_name) {
        let distribution_root =
            resolve_distribution_root(&checkout.cache_directory, checkout.subdir.as_deref())?;
        let source_key = if checkout.revision.is_empty() {
            clone_url.to_string()
        } else {
            format!("{}@{}", clone_url, checkout.revision)
        };
        return resolve_local_registry_package_root(
            project_root,
            &source_key,
            &distribution_root,
            declaration,
            context,
        );
    }
    if let Some(source_root) = local_git_source_root(clone_url) {
        return resolve_local_registry_package_root(
            project_root,
            clone_url,
            &source_root,
            declaration,
            context,
        );
    }
    Err(PrayError::Resolution(format!(
        "git source {source_name} was not prepared"
    )))
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
        if is_local_filesystem_source(clone_url) && local_git_repo_path(clone_url).is_none() {
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
