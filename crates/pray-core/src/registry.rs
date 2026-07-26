use crate::constraint::version_satisfies;
use crate::derived_metadata::RegistryDerivedMetadata;
use crate::manifest::ManifestPackage;
use crate::package_integrity::{artifact_content_digest, require_remote_integrity_fields};
use crate::paths::remove_path_if_exists;
use crate::registry_http::{http_get, http_post, http_put, join_url};
use crate::registry_ssh::{
    resolve_ssh_registry_package_root, submit_confession_ssh, upload_registry_artifact_ssh,
};
use crate::registry_torrent::{fetch_torrent_artifact, fetch_torrent_manifest};
use crate::resolve_context::PackageResolutionContext;
use crate::{PrayError, PrayResult};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RegistryPackageResolution {
    pub root: PathBuf,
    pub signer_fingerprint: Option<String>,
    /// Highest non-yanked version published in registry metadata, regardless of Prayfile constraint.
    pub registry_latest_version: Option<String>,
}

pub fn lockfile_signer_fingerprint(version: &RegistryPackageVersion) -> Option<String> {
    version
        .signer_fingerprint
        .as_deref()
        .filter(|value| crate::ssh_identity::looks_like_ssh_fingerprint(value))
        .map(crate::ssh_identity::normalize_identity)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryIndex {
    pub spec: String,
    #[serde(default)]
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryPackageMetadata {
    pub name: String,
    #[serde(default)]
    pub versions: Vec<RegistryPackageVersion>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryPackageVersion {
    pub version: String,
    pub artifact: String,
    #[serde(default)]
    pub artifact_hash: Option<String>,
    #[serde(default)]
    pub tree_hash: Option<String>,
    #[serde(default)]
    pub yanked: bool,
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default)]
    pub exports: Vec<String>,
    #[serde(default)]
    pub signer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_public_key: Option<String>,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived_metadata: Option<RegistryDerivedMetadata>,
}

impl RegistryPackageVersion {
    pub fn same_identity(&self, other: &Self) -> bool {
        self.version == other.version
            && self.artifact == other.artifact
            && self.artifact_hash == other.artifact_hash
            && self.tree_hash == other.tree_hash
            && self.yanked == other.yanked
            && self.targets == other.targets
            && self.exports == other.exports
            && self.signer == other.signer
            && self.signer_fingerprint == other.signer_fingerprint
            && self.signer_public_key == other.signer_public_key
            && self.published_at == other.published_at
            && self.signature == other.signature
    }

    pub fn merge_annotations_from(&mut self, other: &Self) {
        if self.derived_metadata.is_none() {
            self.derived_metadata = other.derived_metadata.clone();
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfessionSubmission {
    pub package: String,
    pub version: String,
    pub status: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub lockfile: Option<String>,
    #[serde(default)]
    pub distribution_point: Option<String>,
    #[serde(default)]
    pub signer: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub signature: Option<String>,
}

pub fn resolve_registry_package_root(
    project_root: &Path,
    source_url: &str,
    declaration: &ManifestPackage,
    context: &PackageResolutionContext,
) -> PrayResult<RegistryPackageResolution> {
    if crate::ssh_client::is_pray_ssh_url(source_url) {
        return resolve_ssh_registry_package_root(project_root, source_url, declaration, context);
    }

    let metadata = fetch_registry_package_metadata(source_url, &declaration.name)?;
    let registry_latest_version = registry_latest_version_label(&metadata);
    let selected = select_package_version(
        &metadata,
        &declaration.constraint,
        context.preferred_version.as_deref(),
    )?;
    let signer_fingerprint = lockfile_signer_fingerprint(&selected);
    require_remote_integrity_fields(&declaration.name, &selected.version, &selected)?;
    if let Some(vendored_root) = crate::registry_cache::try_vendored_package_root(
        project_root,
        &declaration.name,
        &selected,
    )? {
        return Ok(RegistryPackageResolution {
            root: vendored_root,
            signer_fingerprint,
            registry_latest_version,
        });
    }
    let cache_directory = crate::registry_cache::registry_cache_directory(
        project_root,
        source_url,
        &declaration.name,
        &selected.version,
    );

    if let Some(mut cached) = crate::registry_cache::try_reuse_cached_registry_package(
        &cache_directory,
        &selected,
        signer_fingerprint.clone(),
    )? {
        cached.registry_latest_version = registry_latest_version.clone();
        return Ok(cached);
    }
    if context.offline {
        return Err(crate::registry_cache::offline_package_error(
            &declaration.name,
            &selected.version,
        ));
    }

    if cache_directory.exists() {
        remove_path_if_exists(&cache_directory)?;
    }
    fs::create_dir_all(&cache_directory)?;

    let artifact_url = join_url(source_url, &selected.artifact);
    let torrent_manifest = fetch_torrent_manifest(source_url, &selected.artifact)?;
    let artifact_bytes = if let Some(manifest) = torrent_manifest {
        fetch_torrent_artifact(source_url, &selected.artifact, &manifest)?
    } else {
        http_get(&artifact_url)?
    };
    crate::registry_cache::validate_and_unpack_registry_package(
        &cache_directory,
        declaration,
        &selected,
        &artifact_bytes,
    )?;

    Ok(RegistryPackageResolution {
        root: cache_directory,
        signer_fingerprint,
        registry_latest_version,
    })
}

pub fn resolve_local_registry_package_root(
    project_root: &Path,
    source_key: &str,
    source_root: &Path,
    declaration: &ManifestPackage,
    context: &PackageResolutionContext,
) -> PrayResult<RegistryPackageResolution> {
    let metadata_path = source_root.join(format!("v1/packages/{}.json", declaration.name));
    let metadata_text = fs::read_to_string(&metadata_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PrayError::Resolution(format!(
                "package {} not found in distribution {:?}. \
                 Missing {}. Check the package name, version constraint `{}`, and that the source publishes registry metadata.",
                declaration.name,
                source_root,
                metadata_path.display(),
                declaration.constraint
            ))
        } else {
            PrayError::Resolution(format!(
                "failed to read package metadata {}: {error}",
                metadata_path.display()
            ))
        }
    })?;
    let metadata: RegistryPackageMetadata =
        serde_json::from_str(&metadata_text).map_err(|error| PrayError::Parse {
            kind: "registry metadata",
            message: error.to_string(),
        })?;
    let registry_latest_version = registry_latest_version_label(&metadata);
    let selected = select_package_version(
        &metadata,
        &declaration.constraint,
        context.preferred_version.as_deref(),
    )?;
    let signer_fingerprint = lockfile_signer_fingerprint(&selected);
    require_remote_integrity_fields(&declaration.name, &selected.version, &selected)?;
    if let Some(vendored_root) = crate::registry_cache::try_vendored_package_root(
        project_root,
        &declaration.name,
        &selected,
    )? {
        return Ok(RegistryPackageResolution {
            root: vendored_root,
            signer_fingerprint,
            registry_latest_version,
        });
    }
    let cache_identifier = format!(
        "{}:{}:{}:{}",
        source_key,
        declaration.name,
        selected.version,
        selected
            .artifact_hash
            .as_deref()
            .unwrap_or("no-artifact-hash")
    );
    let cache_directory = crate::registry_cache::registry_cache_directory(
        project_root,
        &cache_identifier,
        &declaration.name,
        &selected.version,
    );

    if let Some(mut cached) = crate::registry_cache::try_reuse_cached_registry_package(
        &cache_directory,
        &selected,
        signer_fingerprint.clone(),
    )? {
        cached.registry_latest_version = registry_latest_version.clone();
        return Ok(cached);
    }
    if context.offline {
        return Err(crate::registry_cache::offline_package_error(
            &declaration.name,
            &selected.version,
        ));
    }

    if cache_directory.exists() {
        remove_path_if_exists(&cache_directory)?;
    }
    fs::create_dir_all(&cache_directory)?;

    let artifact_bytes =
        crate::registry_cache::read_local_registry_artifact_bytes(source_root, &selected.artifact)?;
    crate::registry_cache::validate_and_unpack_registry_package(
        &cache_directory,
        declaration,
        &selected,
        &artifact_bytes,
    )?;

    Ok(RegistryPackageResolution {
        root: cache_directory,
        signer_fingerprint,
        registry_latest_version,
    })
}

pub fn registry_package_signing_identity(version: &RegistryPackageVersion) -> Option<String> {
    crate::ssh_identity::package_signing_identity(
        version.signer.as_deref(),
        version.signer_fingerprint.as_deref(),
    )
}

pub fn registry_artifact_signature(artifact_bytes: &[u8], tree_hash: &str, signer: &str) -> String {
    artifact_content_digest(artifact_bytes, tree_hash, signer)
}

pub fn submit_confession(source_url: &str, confession: &ConfessionSubmission) -> PrayResult<()> {
    if crate::ssh_client::is_pray_ssh_url(source_url) {
        return submit_confession_ssh(source_url, confession);
    }
    let endpoint = join_url(source_url, "v1/confessions");
    let payload =
        serde_json::to_vec(confession).map_err(|error| PrayError::Manifest(error.to_string()))?;
    let response = http_post(&endpoint, "application/json", &payload)?;
    if response.status / 100 != 2 {
        return Err(PrayError::Resolution(format!(
            "confession submission failed with HTTP {}",
            response.status
        )));
    }
    Ok(())
}

pub fn upload_registry_artifact(
    source_url: &str,
    artifact_path: &str,
    bytes: &[u8],
) -> PrayResult<()> {
    if crate::ssh_client::is_pray_ssh_url(source_url) {
        return upload_registry_artifact_ssh(source_url, artifact_path, bytes);
    }
    let endpoint = join_url(source_url, artifact_path);
    let response = http_put(&endpoint, "application/octet-stream", bytes)?;
    if response.status / 100 != 2 {
        return Err(PrayError::Resolution(format!(
            "artifact upload failed with HTTP {}",
            response.status
        )));
    }
    Ok(())
}

fn fetch_registry_package_metadata(
    source_url: &str,
    package_name: &str,
) -> PrayResult<RegistryPackageMetadata> {
    let url = join_url(source_url, &format!("v1/packages/{}.json", package_name));
    let response = http_get(&url)?;
    serde_json::from_slice(&response).map_err(|error| PrayError::Parse {
        kind: "registry metadata",
        message: error.to_string(),
    })
}

pub fn highest_registry_version(
    metadata: &RegistryPackageMetadata,
) -> PrayResult<Option<RegistryPackageVersion>> {
    let mut selected: Option<RegistryPackageVersion> = None;
    for version in &metadata.versions {
        if version.yanked {
            continue;
        }
        match &selected {
            Some(existing) if compare_versions(&version.version, &existing.version)? <= 0 => {}
            _ => selected = Some(version.clone()),
        }
    }
    Ok(selected)
}

pub fn registry_latest_version_label(metadata: &RegistryPackageMetadata) -> Option<String> {
    highest_registry_version(metadata)
        .ok()
        .flatten()
        .map(|version| version.version)
}

pub fn version_is_greater_than(left: &str, right: &str) -> PrayResult<bool> {
    Ok(compare_versions(left, right)? > 0)
}

pub(crate) fn select_package_version(
    metadata: &RegistryPackageMetadata,
    constraint: &str,
    preferred_version: Option<&str>,
) -> PrayResult<RegistryPackageVersion> {
    if let Some(preferred_version) = preferred_version {
        if let Some(version) = metadata
            .versions
            .iter()
            .find(|version| version.version == preferred_version && !version.yanked)
        {
            if version_satisfies(&version.version, constraint)? {
                return Ok(version.clone());
            }
            // Prayfile constraint changed; fall through to the highest satisfying version.
        }
    }
    let mut selected: Option<RegistryPackageVersion> = None;
    for version in &metadata.versions {
        if version.yanked {
            continue;
        }
        if !version_satisfies(&version.version, constraint)? {
            continue;
        }
        match &selected {
            Some(existing) if compare_versions(&version.version, &existing.version)? <= 0 => {}
            _ => selected = Some(version.clone()),
        }
    }
    selected.ok_or_else(|| {
        PrayError::Resolution(format!(
            "no registry version for {} satisfies {}",
            metadata.name, constraint
        ))
    })
}

#[doc(hidden)]
pub fn select_package_version_for_test(
    metadata: &RegistryPackageMetadata,
    constraint: &str,
    preferred_version: Option<&str>,
) -> PrayResult<RegistryPackageVersion> {
    select_package_version(metadata, constraint, preferred_version)
}

fn compare_versions(left: &str, right: &str) -> PrayResult<i32> {
    let left = Version::parse(left).map_err(|error| PrayError::Resolution(error.to_string()))?;
    let right = Version::parse(right).map_err(|error| PrayError::Resolution(error.to_string()))?;
    Ok(match left.cmp(&right) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    })
}

pub fn fetch_optional_distribution_bytes(
    source_url: &str,
    relative_path: &str,
) -> PrayResult<Option<Vec<u8>>> {
    if !source_url.starts_with("http://") && !source_url.starts_with("https://") {
        return Err(PrayError::Unsupported(format!(
            "distribution fetch requires http or https source, got {source_url}"
        )));
    }
    let url = join_url(source_url, relative_path);
    match http_get(&url) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(PrayError::Resolution(message)) if message.contains("404") => Ok(None),
        Err(error) => Err(error),
    }
}
