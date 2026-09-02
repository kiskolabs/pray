use crate::hashing::sha256_prefixed;
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

pub use crate::manifest_format::{format_package_declaration, replace_package_declaration};
pub use crate::manifest_parse::parse_manifest;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub prayfile_version: String,
    pub sources: Vec<ManifestSource>,
    pub targets: Vec<ManifestTarget>,
    pub packages: Vec<ManifestPackage>,
    pub local: Vec<ManifestLocal>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub symbols: BTreeMap<String, String>,
    pub render: RenderPolicy,
    /// Deprecated Prayfile keywords encountered while parsing (`target`, `output`, `agent`, `skills`).
    #[serde(default, skip)]
    pub deprecated_keywords: Vec<String>,
}

impl Manifest {
    pub fn note_deprecated_keyword(&mut self, keyword: &str) {
        if !crate::deprecation::is_deprecated_prayfile_keyword(keyword) {
            return;
        }
        if self
            .deprecated_keywords
            .iter()
            .any(|existing| existing == keyword)
        {
            return;
        }
        self.deprecated_keywords.push(keyword.to_string());
    }

    pub fn deprecation_warnings(&self) -> Vec<String> {
        crate::deprecation::deprecation_warnings_for(&self.deprecated_keywords)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestSource {
    pub name: String,
    pub kind: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subdir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DestinationMode {
    #[default]
    Legacy,
    Compose,
    Tree,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DestinationEntry {
    Package { name: String },
    Local { path: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportRole {
    Fragment,
    Folder,
    File,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestTarget {
    pub name: String,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    pub max_bytes: Option<u64>,
    #[serde(default)]
    pub mode: DestinationMode,
    #[serde(default)]
    pub scoped: bool,
    #[serde(default)]
    pub entries: Vec<DestinationEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestPackage {
    pub name: String,
    #[serde(default = "default_constraint")]
    pub constraint: String,
    pub source: Option<String>,
    #[serde(default)]
    pub exports: Vec<String>,
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub optional: bool,
    pub path: Option<String>,
    pub git: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
    pub tarball: Option<String>,
    pub oci: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<ExportRole>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bound: bool,
}

fn default_constraint() -> String {
    "*".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestLocal {
    pub path: String,
    #[serde(default = "default_local_position")]
    pub position: String,
    #[serde(default)]
    pub optional: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bound: bool,
}

fn default_local_position() -> String {
    "after".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenderPolicy {
    pub mode: String,
    pub conflict: String,
    pub churn: String,
    pub header: bool,
}

impl Default for RenderPolicy {
    fn default() -> Self {
        Self {
            mode: "managed".to_string(),
            conflict: "fail".to_string(),
            churn: "minimal".to_string(),
            header: true,
        }
    }
}

impl Manifest {
    pub fn canonicalized(&self) -> Self {
        let mut manifest = self.clone();
        manifest
            .sources
            .sort_by(|left, right| left.name.cmp(&right.name));
        manifest
            .targets
            .sort_by(|left, right| left.name.cmp(&right.name));
        manifest.packages.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then(left.source.cmp(&right.source))
                .then(left.constraint.cmp(&right.constraint))
        });
        manifest
            .local
            .sort_by(|left, right| left.path.cmp(&right.path));
        manifest
    }

    pub fn manifest_hash(&self) -> PrayResult<String> {
        let canonical = self.canonicalized();
        let bytes = serde_json::to_vec(&canonical)
            .map_err(|error| PrayError::Manifest(error.to_string()))?;
        Ok(sha256_prefixed(&bytes))
    }
}

pub fn read_manifest_text(manifest_path: &Path) -> PrayResult<String> {
    fs::read_to_string(manifest_path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            let manifest_label = manifest_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Prayfile");
            PrayError::Manifest(format!(
                "missing {manifest_label}; run pray init to create one"
            ))
        } else {
            PrayError::Io(error)
        }
    })
}
