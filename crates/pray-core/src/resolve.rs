use crate::constraint::version_satisfies;
use crate::lockfile::Lockfile;
use crate::manifest::{Manifest, ManifestPackage, ManifestSource};
use crate::package_spec::{parse_package_spec, PackageSpec};
use crate::registry::{resolve_local_registry_package_root, resolve_registry_package_root};
use crate::resolve_context::{PackageResolutionContext, ResolveOptions};
use crate::resolve_exports::{
    build_skill_file_index, load_export_bodies, load_package_file_bytes, read_text, select_exports,
};
use crate::resolve_git_sources::{
    prepare_git_sources, prepare_pray_ssh_host_keys, resolve_git_package_root, GitSourceCheckout,
};

use crate::paths::find_prayspec_file;
pub use crate::resolve_git::{discover_distribution_root, git_source_cache_directory};
pub use crate::resolve_git_refresh::{
    annotate_failed_git_refresh, annotate_missing_git_catalog,
    resolution_may_benefit_from_git_source_refresh,
};
pub use crate::resolve_git_sources::refresh_git_sources;
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;
use std::fs;
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

#[path = "resolve_upstream.rs"]
mod resolve_upstream;
pub use resolve_upstream::apply_path_upstream_refreshes;

fn resolve_package(
    project_root: &Path,
    sources: &BTreeMap<String, ManifestSource>,
    git_sources: &BTreeMap<String, GitSourceCheckout>,
    user_config: &crate::config::PrayConfig,
    declaration: &ManifestPackage,
    lockfile: Option<&Lockfile>,
    options: &ResolveOptions,
) -> PrayResult<ResolvedPackage> {
    let PackageRootResolution {
        root,
        signer_fingerprint,
        registry_latest_version,
    } = resolve_package_root(
        project_root,
        sources,
        git_sources,
        user_config,
        declaration,
        lockfile,
        options,
    )?;
    let spec_path = find_prayspec_file(&root)?;
    let spec_text = fs::read_to_string(&spec_path)?;
    let spec = parse_package_spec(&spec_text)?.canonicalized();
    if spec.name != declaration.name {
        return Err(PrayError::Resolution(format!(
            "package path {:?} declares {:?}, expected {:?}",
            root, spec.name, declaration.name
        )));
    }
    if !version_satisfies(&spec.version, &declaration.constraint)? {
        return Err(PrayError::Resolution(format!(
            "package {} version {} does not satisfy constraint {}",
            declaration.name, spec.version, declaration.constraint
        )));
    }
    let selected_exports = select_exports(declaration, &spec)?;
    let file_bytes = load_package_file_bytes(&root, &spec)?;
    let tree_hash = PackageSpec::tree_hash_from_file_bytes(&file_bytes)?;
    let export_bodies = load_export_bodies(&file_bytes, &spec, &selected_exports)?;
    let skill_files = build_skill_file_index(&spec);
    let source_checksum = tree_hash.clone();
    let upstream_context = resolve_upstream::UpstreamResolutionContext::new(
        project_root,
        sources,
        git_sources,
        user_config,
        lockfile,
        options,
    );
    let upstream = resolve_upstream::lock_path_upstream(&upstream_context, declaration, &spec)?;
    Ok(ResolvedPackage {
        declaration: declaration.clone(),
        root,
        spec: spec.clone(),
        tree_hash: tree_hash.clone(),
        artifact_hash: tree_hash.clone(),
        artifact: format!(
            "path:{}",
            spec_path.parent().unwrap_or(&spec_path).to_string_lossy()
        ),
        selected_exports,
        source_checksum,
        export_bodies,
        skill_files,
        signer_fingerprint,
        registry_latest_version,
        explicit: false,
        upstream,
    })
}

#[derive(Debug, Clone)]
struct PackageRootResolution {
    root: PathBuf,
    signer_fingerprint: Option<String>,
    registry_latest_version: Option<String>,
}

fn resolve_package_root(
    project_root: &Path,
    sources: &BTreeMap<String, ManifestSource>,
    git_sources: &BTreeMap<String, GitSourceCheckout>,
    user_config: &crate::config::PrayConfig,
    declaration: &ManifestPackage,
    lockfile: Option<&Lockfile>,
    options: &ResolveOptions,
) -> PrayResult<PackageRootResolution> {
    if let Some(local_path) = user_config.local.package.get(&declaration.name) {
        return Ok(PackageRootResolution {
            root: project_root.join(local_path),
            signer_fingerprint: None,
            registry_latest_version: None,
        });
    }
    if let Some(path) = &declaration.path {
        return Ok(PackageRootResolution {
            root: project_root.join(path),
            signer_fingerprint: None,
            registry_latest_version: None,
        });
    }
    let source_name = implied_source_name(declaration, sources)?;
    if let Some(source_name) = source_name {
        let source = sources
            .get(&source_name)
            .ok_or_else(|| PrayError::Resolution(format!("unknown source: {source_name}")))?;
        let context = PackageResolutionContext::from_lockfile(lockfile, &declaration.name, options);
        if let Some(local_path) = user_config.local.source.get(&source_name) {
            let source_root = project_root.join(local_path);
            let resolved = resolve_local_registry_package_root(
                project_root,
                &format!("local:{source_name}"),
                &source_root,
                declaration,
                &context,
            )?;
            return Ok(PackageRootResolution {
                root: resolved.root,
                signer_fingerprint: resolved.signer_fingerprint,
                registry_latest_version: resolved.registry_latest_version,
            });
        }
        if source.kind == "path" {
            let slug = declaration.name.replace('/', "-");
            return Ok(PackageRootResolution {
                root: project_root.join(&source.url).join(slug),
                signer_fingerprint: None,
                registry_latest_version: None,
            });
        }
        if source.kind == "registry" || source.kind == "static index" || source.kind == "pray_ssh" {
            let resolved =
                resolve_registry_package_root(project_root, &source.url, declaration, &context)?;
            return Ok(PackageRootResolution {
                root: resolved.root,
                signer_fingerprint: resolved.signer_fingerprint,
                registry_latest_version: resolved.registry_latest_version,
            });
        }
        if source.kind == "git" {
            let resolved = resolve_git_package_root(
                project_root,
                &source_name,
                &source.url,
                git_sources,
                declaration,
                &context,
            )?;
            return Ok(PackageRootResolution {
                root: resolved.root,
                signer_fingerprint: resolved.signer_fingerprint,
                registry_latest_version: resolved.registry_latest_version,
            });
        }
        return Err(PrayError::Unsupported(format!(
            "source kind {} not implemented yet",
            source.kind
        )));
    }
    if declaration.git.is_some() || declaration.tarball.is_some() || declaration.oci.is_some() {
        return Err(PrayError::Unsupported(
            "remote sources are not implemented yet".to_string(),
        ));
    }
    let slug = declaration.name.replace('/', "-");
    Ok(PackageRootResolution {
        root: project_root.join(slug),
        signer_fingerprint: None,
        registry_latest_version: None,
    })
}

pub fn missing_local_embed_guidance(path: impl AsRef<str>) -> String {
    let path = path.as_ref();
    format!(
        "Prayfile lists `local \"{path}\"` but the file does not exist. \
         Create the file or remove the entry from Prayfile, then run `pray install`."
    )
}

fn resolve_local_file(
    project_root: &Path,
    declaration: &crate::manifest::ManifestLocal,
) -> PrayResult<ResolvedLocalFile> {
    let path = project_root.join(&declaration.path);
    if !path.exists() {
        if declaration.optional {
            return Ok(ResolvedLocalFile {
                path,
                manifest_path: declaration.path.clone(),
                content: String::new(),
                position: declaration.position.clone(),
                optional: true,
            });
        }
        return Err(PrayError::Resolution(missing_local_embed_guidance(
            &declaration.path,
        )));
    }
    Ok(ResolvedLocalFile {
        content: read_text(&path)?,
        path,
        manifest_path: declaration.path.clone(),
        position: declaration.position.clone(),
        optional: declaration.optional,
    })
}

fn source_map(sources: &[ManifestSource]) -> BTreeMap<String, ManifestSource> {
    sources
        .iter()
        .map(|source| (source.name.clone(), source.clone()))
        .collect()
}

fn package_namespace(name: &str) -> Option<&str> {
    name.split_once('/').map(|(namespace, _)| namespace)
}

fn implied_source_name(
    declaration: &ManifestPackage,
    sources: &BTreeMap<String, ManifestSource>,
) -> PrayResult<Option<String>> {
    if let Some(name) = &declaration.source {
        return Ok(Some(name.clone()));
    }
    if let Some(namespace) = package_namespace(&declaration.name) {
        if sources.contains_key(namespace) {
            return Ok(Some(namespace.to_string()));
        }
    }
    match sources.len() {
        0 => Ok(None),
        1 => Ok(sources.keys().next().cloned()),
        _ => Err(PrayError::Resolution(format!(
            "package {} requires source: when multiple sources are declared and the package namespace does not match a source",
            declaration.name
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{discover_distribution_root, project_root_from_manifest};
    use std::fs;
    use std::path::Path;

    #[test]
    fn project_root_from_manifest_uses_cwd_for_bare_filename() {
        let root = project_root_from_manifest(Path::new("Prayfile"));
        assert_eq!(root, Path::new("."));
    }

    #[test]
    fn project_root_from_manifest_uses_parent_directory() {
        let root = project_root_from_manifest(Path::new("examples/simple-project/Prayfile"));
        assert_eq!(root, Path::new("examples/simple-project"));
    }

    #[test]
    fn discover_distribution_root_finds_root_and_prayers_subdirectory() {
        let workspace =
            std::env::temp_dir().join(format!("pray-discover-distribution-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        let repo_root = workspace.join("repo");
        let prayers_root = repo_root.join("prayers");
        fs::create_dir_all(prayers_root.join("v1/packages")).expect("prayers distribution");
        fs::create_dir_all(repo_root.join("v1/packages")).expect("root distribution");

        assert_eq!(
            discover_distribution_root(&repo_root),
            Some(repo_root.clone())
        );

        fs::remove_dir_all(repo_root.join("v1")).expect("remove root distribution");
        assert_eq!(discover_distribution_root(&repo_root), Some(prayers_root));
        let _ = fs::remove_dir_all(&workspace);
    }

    #[test]
    fn discover_distribution_root_returns_none_without_registry_layout() {
        let workspace =
            std::env::temp_dir().join(format!("pray-discover-missing-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        let repo_root = workspace.join("repo");
        fs::create_dir_all(&repo_root).expect("repo root");
        assert_eq!(discover_distribution_root(&repo_root), None);
        let _ = fs::remove_dir_all(&workspace);
    }
}
