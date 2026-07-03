use crate::hashing::sha256_prefixed;
use crate::manifest::{Manifest, ManifestPackage, ManifestSource};
use crate::package_spec::{parse_package_spec, PackageSpec};
use crate::registry::{resolve_local_registry_package_root, resolve_registry_package_root};
use crate::{PrayError, PrayResult};
use semver::{Version, VersionReq};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ResolvedProject {
    pub manifest_path: PathBuf,
    pub project_root: PathBuf,
    pub manifest: Manifest,
    pub packages: Vec<ResolvedPackage>,
    pub local_files: Vec<ResolvedLocalFile>,
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
}

#[derive(Debug, Clone)]
pub struct ResolvedLocalFile {
    pub path: PathBuf,
    pub content: String,
    pub position: String,
    pub optional: bool,
}

impl ResolvedProject {
    pub fn lockfile_hash(&self) -> PrayResult<String> {
        self.manifest.manifest_hash()
    }
}

pub fn project_root_from_manifest(manifest_path: &Path) -> PathBuf {
    match manifest_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

pub fn resolve_project(manifest_path: &Path) -> PrayResult<ResolvedProject> {
    let project_root = project_root_from_manifest(manifest_path);
    let manifest_text = fs::read_to_string(manifest_path)?;
    let manifest = crate::manifest::parse_manifest(&manifest_text)?;
    let sources = source_map(&manifest.sources);
    let mut packages = Vec::new();
    let mut seen = BTreeSet::new();
    for declaration in &manifest.packages {
        let package = resolve_package(&project_root, &sources, declaration)?;
        if !seen.insert(package.declaration.name.clone()) {
            return Err(PrayError::Resolution(format!(
                "duplicate package declaration: {}",
                package.declaration.name
            )));
        }
        packages.push(package);
    }
    let mut local_files = Vec::new();
    for local in &manifest.local {
        local_files.push(resolve_local_file(&project_root, local)?);
    }
    Ok(ResolvedProject {
        manifest_path: manifest_path.to_path_buf(),
        project_root,
        manifest,
        packages,
        local_files,
    })
}

fn resolve_package(
    project_root: &Path,
    sources: &BTreeMap<String, ManifestSource>,
    declaration: &ManifestPackage,
) -> PrayResult<ResolvedPackage> {
    let root = resolve_package_root(project_root, sources, declaration)?;
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
    let tree_hash = spec.tree_hash_for_root(&root)?;
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
    })
}

fn resolve_package_root(
    project_root: &Path,
    sources: &BTreeMap<String, ManifestSource>,
    declaration: &ManifestPackage,
) -> PrayResult<PathBuf> {
    if let Some(path) = &declaration.path {
        return Ok(project_root.join(path));
    }
    if let Some(source_name) = &declaration.source {
        let source = sources
            .get(source_name)
            .ok_or_else(|| PrayError::Resolution(format!("unknown source: {source_name}")))?;
        if source.kind == "path" {
            let slug = declaration.name.replace('/', "-");
            return Ok(project_root.join(&source.url).join(slug));
        }
        if source.kind == "registry" || source.kind == "static index" {
            return resolve_registry_package_root(project_root, &source.url, declaration);
        }
        if source.kind == "git" {
            return resolve_git_package_root(project_root, &source.url, declaration);
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
    Ok(project_root.join(slug))
}

fn resolve_git_package_root(
    project_root: &Path,
    source_url: &str,
    declaration: &ManifestPackage,
) -> PrayResult<PathBuf> {
    let clone_url = source_url.strip_prefix("git+").unwrap_or(source_url);
    if let Some(source_root) = local_git_source_root(clone_url) {
        return resolve_local_registry_package_root(
            project_root,
            clone_url,
            &source_root,
            declaration,
        );
    }

    let git_cache_directory = project_root
        .join(".pray/cache/git")
        .join(cache_key(clone_url));
    if git_cache_directory.exists() {
        remove_path_if_exists(&git_cache_directory)?;
    }
    if let Some(parent) = git_cache_directory.parent() {
        fs::create_dir_all(parent)?;
    }
    run_git_success(
        project_root,
        &[
            "clone",
            clone_url,
            git_cache_directory.to_str().ok_or_else(|| {
                PrayError::Resolution(format!("invalid git cache path: {:?}", git_cache_directory))
            })?,
        ],
    )?;
    let source_key = format!("{}@clone", clone_url);
    resolve_local_registry_package_root(
        project_root,
        &source_key,
        &git_cache_directory,
        declaration,
    )
}

fn local_git_source_root(clone_url: &str) -> Option<PathBuf> {
    let path = if let Some(path) = clone_url.strip_prefix("file://") {
        PathBuf::from(path)
    } else {
        PathBuf::from(clone_url)
    };

    local_distribution_root(&path)
}

fn local_distribution_root(path: &Path) -> Option<PathBuf> {
    if is_local_distribution_root(path) {
        return Some(path.to_path_buf());
    }

    let prayers_root = path.join("prayers");
    if is_local_distribution_root(&prayers_root) {
        return Some(prayers_root);
    }

    if path.exists() {
        Some(path.to_path_buf())
    } else {
        None
    }
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
                content: String::new(),
                position: declaration.position.clone(),
                optional: true,
            });
        }
        return Err(PrayError::Resolution(missing_local_embed_guidance(&declaration.path)));
    }
    Ok(ResolvedLocalFile {
        content: read_text(&path)?,
        path,
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

fn version_satisfies(version: &str, constraint: &str) -> PrayResult<bool> {
    if constraint.trim().is_empty() || constraint.trim() == "*" {
        return Ok(true);
    }
    let version =
        Version::parse(version).map_err(|error| PrayError::Resolution(error.to_string()))?;
    let req = if constraint.trim_start().starts_with("~>") {
        VersionReq::parse(&ruby_pessimistic_to_semver(constraint)?)
            .map_err(|error| PrayError::Resolution(error.to_string()))?
    } else {
        VersionReq::parse(constraint.trim())
            .map_err(|error| PrayError::Resolution(error.to_string()))?
    };
    Ok(req.matches(&version))
}

fn ruby_pessimistic_to_semver(constraint: &str) -> PrayResult<String> {
    let text = constraint.trim().trim_start_matches("~>").trim();
    let parts: Vec<&str> = text.split('.').collect();
    if parts.is_empty() || parts.len() > 3 {
        return Err(PrayError::Resolution(format!(
            "unsupported Ruby pessimistic constraint: {constraint}"
        )));
    }
    let mut numbers = [0u64; 3];
    for (index, part) in parts.iter().enumerate() {
        numbers[index] = part
            .parse::<u64>()
            .map_err(|error| PrayError::Resolution(error.to_string()))?;
    }
    let lower = format!("{}.{}.{}", numbers[0], numbers[1], numbers[2]);
    let upper = match parts.len() {
        1 => format!("{}.0.0", numbers[0] + 1),
        2 => format!("{}.{}.0", numbers[0], numbers[1] + 1),
        _ => format!("{}.{}.0", numbers[0], numbers[1] + 1),
    };
    Ok(format!(">={}, <{}", lower, upper))
}

fn read_text(path: &Path) -> PrayResult<String> {
    let text = fs::read_to_string(path)?;
    Ok(crate::hashing::normalize_line_endings(&text))
}

#[cfg(test)]
mod tests {
    use super::project_root_from_manifest;
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
}
