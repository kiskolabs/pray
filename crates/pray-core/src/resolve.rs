use crate::client_trust::{effective_trust_home, gate_git_source};
use crate::constraint::version_satisfies;
use crate::hashing::{normalize_line_endings, sha256_prefixed};
use crate::lockfile::Lockfile;
use crate::manifest::{Manifest, ManifestPackage, ManifestSource};
use crate::package_spec::{parse_package_spec, PackageSpec};
use crate::registry::{resolve_local_registry_package_root, resolve_registry_package_root};
use crate::resolve_context::{PackageResolutionContext, ResolveOptions};
use crate::{PrayError, PrayResult};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

pub fn resolve_project_with_options(
    manifest_path: &Path,
    options: &ResolveOptions,
) -> PrayResult<ResolvedProject> {
    let user_config = crate::config::load_user_config()?;
    let project_root = canonical_project_root(manifest_path)?;
    let lockfile_path = project_root.join("Prayfile.lock");
    let lockfile_hints = crate::lockfile::read_lockfile(&lockfile_path).ok();
    let manifest_text = crate::manifest::read_manifest_text(manifest_path)?;
    let manifest = crate::manifest::parse_manifest(&manifest_text)?;
    let manifest_hash = manifest.manifest_hash()?;
    let sources = source_map(&manifest.sources);
    let git_sources = prepare_git_sources(
        &project_root,
        &manifest.sources,
        lockfile_hints.as_ref(),
        options,
    )?;
    let source_host_keys = prepare_pray_ssh_host_keys(&manifest.sources)?;
    let mut packages = Vec::new();
    let mut seen = BTreeSet::new();
    let mut resolution_errors = Vec::new();
    for declaration in &manifest.packages {
        match resolve_package(
            &project_root,
            &sources,
            &git_sources,
            &user_config,
            declaration,
            lockfile_hints.as_ref(),
            options,
        ) {
            Ok(package) => {
                if !seen.insert(package.declaration.name.clone()) {
                    return Err(PrayError::Resolution(format!(
                        "duplicate package declaration: {}",
                        package.declaration.name
                    )));
                }
                packages.push(package);
            }
            Err(error) => resolution_errors.push(format!(
                "{}: {error}",
                declaration.name
            )),
        }
    }
    if !resolution_errors.is_empty() {
        return Err(PrayError::Resolution(resolution_errors.join("\n")));
    }
    let mut local_files = Vec::new();
    let mut local_errors = Vec::new();
    for local in &manifest.local {
        match resolve_local_file(&project_root, local) {
            Ok(resolved) => local_files.push(resolved),
            Err(error) => local_errors.push(format!("local {}: {error}", local.path)),
        }
    }
    if !local_errors.is_empty() {
        return Err(PrayError::Resolution(local_errors.join("\n")));
    }
    Ok(ResolvedProject {
        manifest_path: manifest_path.to_path_buf(),
        project_root,
        manifest,
        manifest_hash,
        packages,
        local_files,
        source_revisions: git_sources
            .into_iter()
            .filter_map(|(name, checkout)| {
                if checkout.revision.is_empty() {
                    None
                } else {
                    Some((name, checkout.revision))
                }
            })
            .collect(),
        source_host_keys,
    })
}

#[derive(Debug, Clone)]
struct GitSourceCheckout {
    cache_directory: PathBuf,
    revision: String,
    subdir: Option<String>,
}

fn prepare_pray_ssh_host_keys(
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

fn prepare_git_sources(
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

fn is_local_filesystem_source(clone_url: &str) -> bool {
    clone_url.starts_with("file://") || Path::new(clone_url).is_absolute()
}

fn local_git_repo_path(clone_url: &str) -> Option<PathBuf> {
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

fn pinned_revision_for_source(
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
    source
        .rev
        .clone()
        .or_else(|| source.tag.clone())
}

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
    if let Some(source_name) = &declaration.source {
        let source = sources
            .get(source_name)
            .ok_or_else(|| PrayError::Resolution(format!("unknown source: {source_name}")))?;
        let context =
            PackageResolutionContext::from_lockfile(lockfile, &declaration.name, options);
        if let Some(local_path) = user_config.local.source.get(source_name) {
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
            let resolved = resolve_registry_package_root(
                project_root,
                &source.url,
                declaration,
                &context,
            )?;
            return Ok(PackageRootResolution {
                root: resolved.root,
                signer_fingerprint: resolved.signer_fingerprint,
                registry_latest_version: resolved.registry_latest_version,
            });
        }
        if source.kind == "git" {
            return resolve_git_package_root(
                project_root,
                source_name,
                &source.url,
                git_sources,
                declaration,
                &context,
            );
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

fn resolve_git_package_root(
    project_root: &Path,
    source_name: &str,
    source_url: &str,
    git_sources: &BTreeMap<String, GitSourceCheckout>,
    declaration: &ManifestPackage,
    context: &PackageResolutionContext,
) -> PrayResult<PackageRootResolution> {
    let clone_url = source_url.strip_prefix("git+").unwrap_or(source_url);
    if let Some(checkout) = git_sources.get(source_name) {
        let distribution_root =
            resolve_distribution_root(&checkout.cache_directory, checkout.subdir.as_deref())?;
        let source_key = if checkout.revision.is_empty() {
            clone_url.to_string()
        } else {
            format!("{}@{}", clone_url, checkout.revision)
        };
        let resolved = resolve_local_registry_package_root(
            project_root,
            &source_key,
            &distribution_root,
            declaration,
            context,
        )?;
        return Ok(PackageRootResolution {
            root: resolved.root,
            signer_fingerprint: resolved.signer_fingerprint,
            registry_latest_version: resolved.registry_latest_version,
        });
    }
    if let Some(source_root) = local_git_source_root(clone_url) {
        let resolved = resolve_local_registry_package_root(
            project_root,
            clone_url,
            &source_root,
            declaration,
            context,
        )?;
        return Ok(PackageRootResolution {
            root: resolved.root,
            signer_fingerprint: resolved.signer_fingerprint,
            registry_latest_version: resolved.registry_latest_version,
        });
    }
    Err(PrayError::Resolution(format!(
        "git source {source_name} was not prepared"
    )))
}

pub fn refresh_git_sources(manifest_path: &Path) -> PrayResult<()> {
    let project_root = canonical_project_root(manifest_path)?;
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

fn ensure_git_repository(
    project_root: &Path,
    clone_url: &str,
    refresh: bool,
    pinned_revision: Option<&str>,
    sparse_subdir: Option<&str>,
) -> PrayResult<(PathBuf, String)> {
    let git_cache_directory = project_root
        .join(".pray/cache/git")
        .join(cache_key(clone_url));

    if git_cache_directory.join(".git").is_dir() {
        if refresh {
            refresh_global_git_cache(clone_url)?;
        }
        if let Some(revision) = pinned_revision {
            checkout_git_revision(&git_cache_directory, clone_url, revision, refresh)?;
        } else if refresh {
            refresh_git_worktree(&git_cache_directory, clone_url)?;
        }
        if let Some(subdir) = sparse_subdir {
            apply_sparse_checkout(&git_cache_directory, subdir)?;
        }
        let revision = git_head_revision(&git_cache_directory)?;
        return finalize_git_repository(clone_url, &git_cache_directory, revision);
    }

    if git_cache_directory.exists() {
        remove_path_if_exists(&git_cache_directory)?;
    }
    if let Some(parent) = git_cache_directory.parent() {
        fs::create_dir_all(parent)?;
    }
    let destination = git_cache_directory.to_str().ok_or_else(|| {
        PrayError::Resolution(format!("invalid git cache path: {:?}", git_cache_directory))
    })?;
    if seed_git_cache_from_global(clone_url, destination, project_root)? {
        ensure_git_remote_origin(&git_cache_directory, clone_url)?;
    } else {
        run_git_success(
            project_root,
            &["clone", "--depth", "1", clone_url, destination],
        )?;
        let _ = mirror_git_cache_to_global(clone_url, &git_cache_directory);
    }
    if let Some(revision) = pinned_revision {
        checkout_git_revision(&git_cache_directory, clone_url, revision, true)?;
    }
    if let Some(subdir) = sparse_subdir {
        apply_sparse_checkout(&git_cache_directory, subdir)?;
    }
    let revision = git_head_revision(&git_cache_directory)?;
    finalize_git_repository(clone_url, &git_cache_directory, revision)
}

fn global_cache_root() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("PRAY_CACHE") {
        return Some(PathBuf::from(path));
    }
    if let Ok(home) = std::env::var("PRAY_HOME") {
        return Some(PathBuf::from(home).join("cache"));
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache").join("pray"))
}

fn global_git_cache_directory(clone_url: &str) -> Option<PathBuf> {
    global_cache_root().map(|root| root.join("git").join(cache_key(clone_url)))
}

fn global_git_cache_ready(global_cache: &Path) -> bool {
    global_cache.join(".git").is_dir() || global_cache.join("HEAD").is_file()
}

fn seed_git_cache_from_global(
    clone_url: &str,
    destination: &str,
    working_directory: &Path,
) -> PrayResult<bool> {
    let Some(global_cache) = global_git_cache_directory(clone_url) else {
        return Ok(false);
    };
    if !global_git_cache_ready(&global_cache) {
        return Ok(false);
    }
    let global_path = global_cache.to_str().ok_or_else(|| {
        PrayError::Resolution(format!("invalid global git cache path: {:?}", global_cache))
    })?;
    run_git_success(
        working_directory,
        &["clone", "--depth", "1", "--quiet", global_path, destination],
    )?;
    Ok(true)
}

fn mirror_git_cache_to_global(clone_url: &str, project_cache: &Path) -> PrayResult<()> {
    let Some(global_cache) = global_git_cache_directory(clone_url) else {
        return Ok(());
    };
    if global_git_cache_ready(&global_cache) {
        return Ok(());
    }
    let cache_parent = project_cache.parent().ok_or_else(|| {
        PrayError::Resolution(format!(
            "invalid project git cache path: {:?}",
            project_cache
        ))
    })?;
    let cache_name = project_cache
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            PrayError::Resolution(format!(
                "invalid project git cache path: {:?}",
                project_cache
            ))
        })?;
    if let Some(parent) = global_cache.parent() {
        fs::create_dir_all(parent)?;
    }
    let destination = global_cache.to_str().ok_or_else(|| {
        PrayError::Resolution(format!("invalid global git cache path: {:?}", global_cache))
    })?;
    if global_cache.exists() {
        remove_path_if_exists(&global_cache)?;
    }
    run_git_success(
        cache_parent,
        &["clone", "--bare", "--quiet", cache_name, destination],
    )?;
    Ok(())
}

fn apply_sparse_checkout(repository: &Path, subdir: &str) -> PrayResult<()> {
    run_git_success(repository, &["sparse-checkout", "init", "--cone"])?;
    run_git_success(repository, &["sparse-checkout", "set", subdir])?;
    Ok(())
}

fn resolve_distribution_root(repo_root: &Path, subdir: Option<&str>) -> PrayResult<PathBuf> {
    if let Some(subdir) = subdir {
        let path = repo_root.join(subdir);
        if is_local_distribution_root(&path) {
            return Ok(path);
        }
        return Err(PrayError::Resolution(format!(
            "no pray distribution root at subdir {:?} in git source {:?}",
            path, repo_root
        )));
    }
    require_distribution_root(repo_root)
}

fn finalize_git_repository(
    clone_url: &str,
    git_cache_directory: &Path,
    revision: String,
) -> PrayResult<(PathBuf, String)> {
    gate_git_source(&effective_trust_home()?, clone_url, git_cache_directory)?;
    if crate::client_trust::env_truthy("PRAY_TRUST_IMPORT") {
        let global_scope = crate::client_trust::env_truthy("PRAY_TRUST_GLOBAL");
        crate::client_trust::prompt_import_signing_keys_for_source(
            &effective_trust_home()?,
            clone_url,
            git_cache_directory,
            global_scope,
        )?;
    }
    Ok((git_cache_directory.to_path_buf(), revision))
}

pub fn git_source_cache_directory(project_root: &Path, clone_url: &str) -> PathBuf {
    project_root
        .join(".pray/cache/git")
        .join(cache_key(clone_url))
}

fn ensure_git_remote_origin(repository: &Path, clone_url: &str) -> PrayResult<()> {
    if run_git_success(repository, &["remote", "get-url", "origin"]).is_ok() {
        run_git_success(repository, &["remote", "set-url", "origin", clone_url])?;
    } else {
        run_git_success(repository, &["remote", "add", "origin", clone_url])?;
    }
    Ok(())
}

fn refresh_global_git_cache(clone_url: &str) -> PrayResult<()> {
    let Some(global_cache) = global_git_cache_directory(clone_url) else {
        return Ok(());
    };
    if !global_git_cache_ready(&global_cache) {
        return Ok(());
    }
    ensure_git_remote_origin(&global_cache, clone_url)?;
    run_git_success(&global_cache, &["fetch", "--depth", "1", "origin"])?;
    Ok(())
}

fn refresh_git_worktree(repository: &Path, clone_url: &str) -> PrayResult<()> {
    ensure_git_remote_origin(repository, clone_url)?;
    run_git_success(repository, &["fetch", "--depth", "1", "origin"])?;
    run_git_success(repository, &["reset", "--hard", "FETCH_HEAD"])?;
    Ok(())
}

fn checkout_git_revision(
    repository: &Path,
    clone_url: &str,
    revision: &str,
    allow_fetch: bool,
) -> PrayResult<()> {
    if git_object_exists(repository, revision) {
        run_git_success(repository, &["reset", "--hard", revision])?;
        return Ok(());
    }
    if !allow_fetch {
        return Err(PrayError::Resolution(format!(
            "git source {:?} is locked to revision {revision}, but that commit is not available locally; rerun pray install without --locked to refresh the cache",
            repository
        )));
    }
    ensure_git_remote_origin(repository, clone_url)?;
    run_git_success(repository, &["fetch", "--depth", "1", "origin", revision])?;
    if git_object_exists(repository, revision) {
        run_git_success(repository, &["reset", "--hard", revision])?;
        return Ok(());
    }
    run_git_success(repository, &["fetch", "origin", revision])?;
    run_git_success(repository, &["reset", "--hard", revision])?;
    Ok(())
}

fn git_object_exists(repository: &Path, object: &str) -> bool {
    run_git_success(repository, &["cat-file", "-e", object]).is_ok()
}

fn git_head_revision(repository: &Path) -> PrayResult<String> {
    let output = run_git_command(repository, &["rev-parse", "HEAD"])?;
    if !output.status.success() {
        return Err(command_error("git rev-parse HEAD", output));
    }
    let revision = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if revision.is_empty() {
        return Err(PrayError::Resolution(
            "git repository has no HEAD revision".to_string(),
        ));
    }
    Ok(revision)
}

fn require_distribution_root(repo_root: &Path) -> PrayResult<PathBuf> {
    discover_distribution_root(repo_root).ok_or_else(|| {
        PrayError::Resolution(format!(
            "no pray distribution root in git source {:?}. \
             Expected v1/packages at the repository root or under prayers/. \
             Publish with `pray publish --root ./prayers` or point the source at a distribution repository.",
            repo_root
        ))
    })
}

fn local_git_source_root(clone_url: &str) -> Option<PathBuf> {
    let path = if let Some(path) = clone_url.strip_prefix("file://") {
        PathBuf::from(path)
    } else {
        PathBuf::from(clone_url)
    };

    if !path.exists() {
        return None;
    }
    discover_distribution_root(&path)
}

fn discover_distribution_root(path: &Path) -> Option<PathBuf> {
    if is_local_distribution_root(path) {
        return Some(path.to_path_buf());
    }

    let prayers_root = path.join("prayers");
    if is_local_distribution_root(&prayers_root) {
        return Some(prayers_root);
    }

    None
}

fn is_local_distribution_root(path: &Path) -> bool {
    path.join("v1/packages").is_dir()
}

fn cache_key(text: &str) -> String {
    sha256_prefixed(text.as_bytes())
        .trim_start_matches("sha256:")
        .chars()
        .take(16)
        .collect()
}

fn run_git_success(root: &Path, arguments: &[&str]) -> PrayResult<()> {
    let output = run_git_command(root, arguments)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(command_error("git", output))
    }
}

fn run_git_command(root: &Path, arguments: &[&str]) -> PrayResult<std::process::Output> {
    Command::new(git_program())
        .current_dir(root)
        .args(arguments)
        .output()
        .map_err(|error| PrayError::Unsupported(format!("failed to run `git`: {error}")))
}

fn git_program() -> String {
    [
        "/usr/bin/git",
        "/opt/homebrew/bin/git",
        "/usr/local/bin/git",
        "git",
    ]
    .into_iter()
    .find(|candidate| Path::new(candidate).exists() || *candidate == "git")
    .unwrap_or("git")
    .to_string()
}

fn command_error(program: &str, output: std::process::Output) -> PrayError {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let mut message = format!("{program} failed with status {}", output.status);
    if !stderr.is_empty() {
        message.push_str(&format!(": {stderr}"));
    } else if !stdout.is_empty() {
        message.push_str(&format!(": {stdout}"));
    }
    PrayError::Resolution(message)
}

fn remove_path_if_exists(path: &Path) -> PrayResult<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => {
            fs::remove_dir_all(path)?;
            Ok(())
        }
        Ok(_) => {
            fs::remove_file(path)?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
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

fn find_prayspec_file(root: &Path) -> PrayResult<PathBuf> {
    let mut prayspec_files = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("prayspec") {
            prayspec_files.push(path);
        }
    }
    match prayspec_files.len() {
        1 => Ok(prayspec_files.remove(0)),
        0 => Err(PrayError::Resolution(format!(
            "no prayspec file found in {:?}",
            root
        ))),
        _ => Err(PrayError::Resolution(format!(
            "multiple prayspec files found in {:?}",
            root
        ))),
    }
}

fn source_map(sources: &[ManifestSource]) -> BTreeMap<String, ManifestSource> {
    sources
        .iter()
        .map(|source| (source.name.clone(), source.clone()))
        .collect()
}

fn select_exports(declaration: &ManifestPackage, spec: &PackageSpec) -> PrayResult<Vec<String>> {
    if declaration.exports.is_empty() {
        return Ok(spec.exports.keys().cloned().collect());
    }
    for export in &declaration.exports {
        if !spec.exports.contains_key(export) {
            return Err(PrayError::Resolution(format!(
                "package {} does not export {}",
                declaration.name, export
            )));
        }
    }
    Ok(declaration.exports.clone())
}

fn read_text(path: &Path) -> PrayResult<String> {
    let text = fs::read_to_string(path)?;
    Ok(normalize_line_endings(&text))
}

fn load_package_file_bytes(
    root: &Path,
    spec: &PackageSpec,
) -> PrayResult<BTreeMap<String, Vec<u8>>> {
    let mut file_bytes = BTreeMap::new();
    for file in &spec.files {
        let path = root.join(file);
        if !path.exists() {
            return Err(PrayError::Integrity(format!(
                "package file missing: {}",
                file
            )));
        }
        if path.is_dir() {
            return Err(PrayError::Integrity(format!(
                "package file is a directory: {}",
                file
            )));
        }
        file_bytes.insert(file.clone(), fs::read(&path)?);
    }
    Ok(file_bytes)
}

fn load_export_bodies(
    file_bytes: &BTreeMap<String, Vec<u8>>,
    spec: &PackageSpec,
    selected_exports: &[String],
) -> PrayResult<BTreeMap<String, String>> {
    let mut export_bodies = BTreeMap::new();
    for export_name in selected_exports {
        let entry = spec.exports.get(export_name).ok_or_else(|| {
            PrayError::Resolution(format!(
                "package {} is missing export {}",
                spec.name, export_name
            ))
        })?;
        if entry.kind != "fragment" {
            continue;
        }
        let bytes = file_bytes.get(&entry.path).ok_or_else(|| {
            PrayError::Integrity(format!(
                "package file missing for export {}: {}",
                export_name, entry.path
            ))
        })?;
        let text = std::str::from_utf8(bytes).map_err(|error| {
            PrayError::Integrity(format!(
                "package file is not valid utf-8 for export {}: {}",
                export_name, error
            ))
        })?;
        export_bodies.insert(export_name.clone(), normalize_line_endings(text));
    }
    Ok(export_bodies)
}

fn build_skill_file_index(spec: &PackageSpec) -> BTreeMap<String, Vec<String>> {
    let mut index = BTreeMap::new();
    for (export_name, export) in &spec.exports {
        if !matches!(export.kind.as_str(), "folder" | "skill") {
            continue;
        }
        let folder_prefix = export.path.trim_end_matches('/');
        let files = indexed_files_under_prefix(&spec.files, folder_prefix);
        if !files.is_empty() {
            index.insert(export_name.clone(), files);
        }
    }
    for (skill_name, skill) in &spec.skills {
        if index.contains_key(skill_name) {
            continue;
        }
        let skill_prefix = skill.path.trim_end_matches('/');
        let files = indexed_files_under_prefix(&spec.files, skill_prefix);
        if !files.is_empty() {
            index.insert(skill_name.clone(), files);
        }
    }
    index
}

fn indexed_files_under_prefix(files: &[String], prefix: &str) -> Vec<String> {
    let mut indexed = Vec::new();
    for file in files {
        if let Some(relative) = skill_relative_file(file, prefix) {
            indexed.push(relative);
        }
    }
    indexed
}

fn skill_relative_file(file: &str, skill_prefix: &str) -> Option<String> {
    let relative = file.strip_prefix(skill_prefix)?.trim_start_matches('/');
    if relative.is_empty() || file == skill_prefix {
        None
    } else {
        Some(relative.to_string())
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
