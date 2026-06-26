use crate::hashing::sha256_prefixed;
use crate::render::RenderedTarget;
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Lockfile {
    pub prayfile_lock: String,
    pub spec: String,
    pub generated_by: String,
    pub manifest_hash: String,
    pub source: Vec<LockSource>,
    pub package: Vec<LockedPackage>,
    pub target: Vec<LockedTarget>,
    pub managed_span: Vec<ManagedSpanRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockSource {
    pub name: String,
    pub kind: String,
    pub url: String,
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

pub fn build_lockfile(
    manifest_hash: String,
    manifest_sources: &[crate::manifest::ManifestSource],
    manifest_targets: &[crate::manifest::ManifestTarget],
    rendered: &[RenderedTarget],
    packages: &[crate::resolve::ResolvedPackage],
) -> Lockfile {
    Lockfile {
        prayfile_lock: "1".to_string(),
        spec: "0.1".to_string(),
        generated_by: "pray 0.1.0".to_string(),
        manifest_hash,
        source: manifest_sources
            .iter()
            .map(|source| LockSource {
                name: source.name.clone(),
                kind: source.kind.clone(),
                url: source.url.clone(),
            })
            .collect(),
        package: packages
            .iter()
            .map(|package| LockedPackage {
                name: package.declaration.name.clone(),
                version: package.spec.version.clone(),
                source: package.declaration.source.clone(),
                path: package.root.to_string_lossy().to_string(),
                tree_hash: package.tree_hash.clone(),
                artifact_hash: package.artifact_hash.clone(),
                artifact: package.artifact.clone(),
                exports: package.selected_exports.clone(),
                dependencies: package
                    .spec
                    .dependencies
                    .iter()
                    .map(|dependency| dependency.name.clone())
                    .collect(),
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
    }
    .canonicalized()
}
