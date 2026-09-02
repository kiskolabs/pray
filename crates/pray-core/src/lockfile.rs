use crate::hashing::sha256_prefixed;
use crate::render::RenderedTarget;
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Lockfile {
    pub prayfile_lock: String,
    pub spec: String,
    pub generated_by: String,
    pub manifest_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    pub source: Vec<LockSource>,
    pub package: Vec<LockedPackage>,
    pub target: Vec<LockedTarget>,
    pub managed_span: Vec<ManagedSpanRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provisioned: Vec<ProvisionedFileRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockSource {
    pub name: String,
    pub kind: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_key_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub path: String,
    pub tree_hash: String,
    pub artifact_hash: String,
    pub artifact: String,
    pub exports: Vec<String>,
    pub dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockedTarget {
    pub name: String,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManagedSpanRecord {
    pub id: String,
    pub target: String,
    pub open_line: usize,
    pub close_line: usize,
    pub ideal_checksum: String,
    pub package: String,
    pub export: String,
    pub source_checksum: String,
    pub silenced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvisionedFileRecord {
    pub path: String,
    pub content_hash: String,
    pub package: String,
    pub export: String,
}

impl Lockfile {
    pub fn canonicalized(&self) -> Self {
        let mut lockfile = self.clone();
        lockfile
            .source
            .sort_by(|left, right| left.name.cmp(&right.name));
        lockfile.package.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then(left.source.cmp(&right.source))
                .then(left.version.cmp(&right.version))
        });
        lockfile
            .target
            .sort_by(|left, right| left.name.cmp(&right.name));
        lockfile.managed_span.sort_by(|left, right| {
            left.target
                .cmp(&right.target)
                .then(left.open_line.cmp(&right.open_line))
                .then(left.id.cmp(&right.id))
        });
        lockfile.provisioned.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then(left.package.cmp(&right.package))
        });
        lockfile
    }

    pub fn serialized(&self) -> PrayResult<String> {
        let bytes = toml::to_string_pretty(&self.canonicalized())
            .map_err(|error| PrayError::Manifest(error.to_string()))?;
        Ok(bytes)
    }

    pub fn file_hash(&self) -> PrayResult<String> {
        let text = self.serialized()?;
        Ok(sha256_prefixed(text.as_bytes()))
    }

    pub fn equivalent_to(&self, other: &Self) -> bool {
        self == &other.canonicalized()
    }
}

pub fn lockfiles_equivalent(canonical: &Lockfile, other: &Lockfile) -> bool {
    canonical.equivalent_to(other)
}

pub fn write_lockfile_if_changed(path: &Path, lockfile: &Lockfile) -> PrayResult<()> {
    let serialized = lockfile.serialized()?;
    if path.exists() {
        if let Ok(existing) = fs::read(path) {
            if existing == serialized.as_bytes() {
                return Ok(());
            }
        }
    }
    fs::write(path, serialized)?;
    Ok(())
}

pub fn write_lockfile(path: &Path, lockfile: &Lockfile) -> PrayResult<()> {
    let serialized = lockfile.serialized()?;
    fs::write(path, serialized)?;
    Ok(())
}

pub fn read_lockfile(path: &Path) -> PrayResult<Lockfile> {
    let text = fs::read_to_string(path)?;
    let lockfile = toml::from_str(&text).map_err(|error| PrayError::Parse {
        kind: "lockfile",
        message: error.to_string(),
    })?;
    Ok(lockfile)
}

pub fn relative_lockfile_path(project_root: &Path, path: &Path) -> String {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    };
    let normalized_root = lexical_normalize_path(project_root);
    let normalized_absolute = lexical_normalize_path(&absolute);
    let relative = normalized_absolute
        .strip_prefix(&normalized_root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| {
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                lexical_normalize_path(path)
            }
        });
    format_relative_lockfile_path(&relative)
}

fn format_relative_lockfile_path(relative: &Path) -> String {
    let text = relative.to_string_lossy().replace('\\', "/");
    if text == "." || text.starts_with("./") {
        text
    } else {
        format!("./{text}")
    }
}

fn lexical_normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(std::path::MAIN_SEPARATOR_STR),
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = normalized.pop();
            }
            Component::Normal(segment) => normalized.push(segment),
        }
    }
    normalized
}

fn normalize_lockfile_artifact(project_root: &Path, artifact: &str, package_root: &Path) -> String {
    if let Some(path_text) = artifact.strip_prefix("path:") {
        let path = Path::new(path_text);
        let relative = if path.is_absolute() {
            relative_lockfile_path(project_root, path)
        } else {
            relative_lockfile_path(project_root, package_root)
        };
        format!("path:{relative}")
    } else {
        artifact.to_string()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_lockfile(
    manifest_hash: String,
    environment: Option<String>,
    project_root: &Path,
    manifest_sources: &[crate::manifest::ManifestSource],
    manifest_targets: &[crate::manifest::ManifestTarget],
    rendered: &[RenderedTarget],
    packages: &[crate::resolve::ResolvedPackage],
    source_revisions: &std::collections::BTreeMap<String, String>,
    source_host_keys: &std::collections::BTreeMap<String, String>,
) -> Lockfile {
    Lockfile {
        prayfile_lock: "1".to_string(),
        spec: "0.1".to_string(),
        generated_by: format!("pray {}", env!("CARGO_PKG_VERSION")),
        manifest_hash,
        environment,
        source: manifest_sources
            .iter()
            .map(|source| LockSource {
                name: source.name.clone(),
                kind: source.kind.clone(),
                url: source.url.clone(),
                revision: source_revisions.get(&source.name).cloned(),
                host_key_fingerprint: source_host_keys.get(&source.name).cloned(),
            })
            .collect(),
        package: packages
            .iter()
            .map(|package| LockedPackage {
                name: package.declaration.name.clone(),
                version: package.spec.version.clone(),
                source: package.declaration.source.clone(),
                path: relative_lockfile_path(project_root, &package.root),
                tree_hash: package.tree_hash.clone(),
                artifact_hash: package.artifact_hash.clone(),
                artifact: normalize_lockfile_artifact(
                    project_root,
                    &package.artifact,
                    &package.root,
                ),
                exports: package.selected_exports.clone(),
                dependencies: package
                    .spec
                    .dependencies
                    .iter()
                    .map(|dependency| dependency.name.clone())
                    .collect(),
                signer_fingerprint: package.signer_fingerprint.clone(),
            })
            .collect(),
        target: manifest_targets
            .iter()
            .map(|target| LockedTarget {
                name: target.name.clone(),
                outputs: target.outputs.clone(),
            })
            .collect(),
        managed_span: rendered
            .iter()
            .flat_map(|target| target.managed_spans.iter().cloned())
            .collect(),
        provisioned: Vec::new(),
    }
    .canonicalized()
}

#[cfg(test)]
#[path = "lockfile_unit.rs"]
mod lockfile_unit;
