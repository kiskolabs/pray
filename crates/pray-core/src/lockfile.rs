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

#[cfg(test)]
mod tests {
    use super::{build_lockfile, lockfiles_equivalent, LockSource, LockedPackage, Lockfile};
    use std::collections::BTreeMap;

    #[test]
    fn build_lockfile_records_git_source_revision() {
        let mut source_revisions = BTreeMap::new();
        source_revisions.insert(
            "dist".to_string(),
            "abc123def4567890abc123def4567890abc123de".to_string(),
        );
        let lockfile = build_lockfile(
            "sha256:manifest".to_string(),
            &[crate::manifest::ManifestSource {
                name: "dist".to_string(),
                kind: "git".to_string(),
                url: "git+https://example.com/dist.git".to_string(),
                subdir: None,
            }],
            &[],
            &[],
            &[],
            &source_revisions,
            &BTreeMap::new(),
        );
        assert_eq!(
            lockfile.source,
            vec![LockSource {
                name: "dist".to_string(),
                kind: "git".to_string(),
                url: "git+https://example.com/dist.git".to_string(),
                revision: Some("abc123def4567890abc123def4567890abc123de".to_string()),
                host_key_fingerprint: None,
            }]
        );
        let serialized = lockfile.serialized().expect("serialize lockfile");
        assert!(serialized.contains("revision ="));
    }

    #[test]
    fn lockfiles_equivalent_ignores_field_order() {
        let mut left = Lockfile::default();
        left.manifest_hash = "sha256:manifest".to_string();
        left.package.push(LockedPackage {
            name: "alpha".to_string(),
            version: "1.0.0".to_string(),
            source: None,
            path: "packages/alpha".to_string(),
            tree_hash: "sha256:tree".to_string(),
            artifact_hash: "sha256:artifact".to_string(),
            artifact: "alpha-1.0.0.praypkg".to_string(),
            exports: vec!["SKILL.md".to_string()],
            dependencies: Vec::new(),
            signer_fingerprint: None,
        });
        let mut right = left.clone();
        right.package.reverse();
        assert!(lockfiles_equivalent(&left.canonicalized(), &right));
    }
}

pub fn build_lockfile(
    manifest_hash: String,
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
        generated_by: "pray 0.1.0".to_string(),
        manifest_hash,
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
    }
    .canonicalized()
}
