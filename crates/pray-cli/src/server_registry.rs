use pray_core::derived_metadata::derive_registry_derived_metadata_from_archive_bytes;
use pray_core::registry::{RegistryIndex, RegistryPackageMetadata};
use pray_core::{PrayError, PrayResult};
use pray_transport::PeerInfo;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn ensure_derived_metadata(
    root: &Path,
    metadata: &mut RegistryPackageMetadata,
) -> PrayResult<()> {
    for version in &mut metadata.versions {
        if version.derived_metadata.is_some() {
            continue;
        }
        let artifact_path = root.join(&version.artifact);
        let artifact_bytes = fs::read(&artifact_path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                PrayError::Resolution(format!(
                    "artifact not found for derived metadata: {}",
                    version.artifact
                ))
            } else {
                PrayError::from(error)
            }
        })?;
        version.derived_metadata = Some(derive_registry_derived_metadata_from_archive_bytes(
            &artifact_bytes,
        )?);
    }
    Ok(())
}

pub(crate) fn read_registry_package_metadata(
    root: &Path,
    package_name: &str,
) -> PrayResult<RegistryPackageMetadata> {
    let metadata_path = registry_metadata_path(root, package_name);
    let metadata_text = fs::read_to_string(&metadata_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PrayError::Resolution(format!("package metadata not found: {}", package_name))
        } else {
            PrayError::from(error)
        }
    })?;
    let metadata: RegistryPackageMetadata =
        serde_json::from_str(&metadata_text).map_err(|error| PrayError::Parse {
            kind: "registry metadata",
            message: error.to_string(),
        })?;
    if metadata.name != package_name {
        return Err(PrayError::Resolution(format!(
            "registry metadata name mismatch: expected {}, found {}",
            package_name, metadata.name
        )));
    }
    Ok(metadata)
}

pub(crate) fn write_registry_package_metadata(
    path: &Path,
    metadata: &RegistryPackageMetadata,
) -> PrayResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(metadata)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    )?;
    Ok(())
}

pub(crate) fn registry_metadata_path(root: &Path, package_name: &str) -> PathBuf {
    root.join("v1/packages")
        .join(package_name)
        .with_extension("json")
}

pub(crate) fn read_known_peers(root: &Path) -> PrayResult<Vec<PeerInfo>> {
    let path = root.join("v1/peers.json");
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let peers: Vec<PeerInfo> = serde_json::from_str(&text).map_err(|error| PrayError::Parse {
        kind: "peer list",
        message: error.to_string(),
    })?;
    for peer in &peers {
        if peer.name.trim().is_empty() || peer.url.trim().is_empty() {
            return Err(PrayError::Resolution(
                "peer list contains an entry with an empty name or url".to_string(),
            ));
        }
    }
    Ok(peers)
}

pub(crate) fn latest_publish_timestamp(metadata: &RegistryPackageMetadata) -> Option<u64> {
    metadata
        .versions
        .iter()
        .filter_map(|version| version.published_at.as_deref())
        .filter_map(|published_at| published_at.parse::<u64>().ok())
        .max()
}

pub(crate) fn merge_registry_package_metadata(
    root: &Path,
    incoming: RegistryPackageMetadata,
) -> PrayResult<RegistryPackageMetadata> {
    let mut current = read_or_create_registry_package_metadata(root, &incoming.name)?;
    for incoming_version in incoming.versions {
        match current
            .versions
            .iter()
            .position(|version| version.version == incoming_version.version)
        {
            Some(index) if current.versions[index].same_identity(&incoming_version) => {
                current.versions[index].merge_annotations_from(&incoming_version);
            }
            Some(_) => {
                return Err(PrayError::Resolution(format!(
                    "conflicting package version received for {} {}",
                    incoming.name, incoming_version.version
                )));
            }
            None => current.versions.push(incoming_version),
        }
    }
    Ok(current)
}

pub(crate) fn read_or_create_registry_package_metadata(
    root: &Path,
    package_name: &str,
) -> PrayResult<RegistryPackageMetadata> {
    match read_registry_package_metadata(root, package_name) {
        Ok(metadata) => Ok(metadata),
        Err(PrayError::Resolution(message))
            if message.starts_with("package metadata not found") =>
        {
            Ok(RegistryPackageMetadata {
                name: package_name.to_string(),
                versions: Vec::new(),
            })
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn update_registry_index_with_package(
    root: &Path,
    package_name: &str,
) -> PrayResult<()> {
    let mut index = read_or_create_registry_index(root)?;
    if index.spec.trim().is_empty() {
        index.spec = "prayfile-distribution-1".to_string();
    }
    if !index
        .packages
        .iter()
        .any(|existing| existing == package_name)
    {
        index.packages.push(package_name.to_string());
    }
    write_registry_index(root, &index)
}

fn read_or_create_registry_index(root: &Path) -> PrayResult<RegistryIndex> {
    let index_path = root.join("v1/index.json");
    match fs::read_to_string(&index_path) {
        Ok(index_text) => serde_json::from_str(&index_text).map_err(|error| PrayError::Parse {
            kind: "registry index",
            message: error.to_string(),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(RegistryIndex {
            spec: "prayfile-distribution-1".to_string(),
            packages: Vec::new(),
        }),
        Err(error) => Err(error.into()),
    }
}

fn write_registry_index(root: &Path, index: &RegistryIndex) -> PrayResult<()> {
    let path = root.join("v1/index.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(index)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    )?;
    Ok(())
}

pub(crate) fn read_registry_index(root: &Path) -> PrayResult<RegistryIndex> {
    let index_path = root.join("v1/index.json");
    let index_text = fs::read_to_string(&index_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PrayError::Resolution("missing registry index".to_string())
        } else {
            PrayError::from(error)
        }
    })?;
    serde_json::from_str(&index_text).map_err(|error| PrayError::Parse {
        kind: "registry index",
        message: error.to_string(),
    })
}
