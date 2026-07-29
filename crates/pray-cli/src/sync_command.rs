use crate::revision::{record_root_revision, RevisionAction};
use crate::sync_peers::{
    load_known_peer_records, map_transport_error, normalize_peer_info, upsert_known_peer,
    write_known_peer_records,
};
use crate::{
    load_registry_index, load_registry_package_metadata, registry_metadata_path,
    write_registry_index, write_registry_package_metadata,
};
use pray_core::derived_metadata::derive_registry_derived_metadata_from_archive_bytes;
use pray_core::hashing::sha256_prefixed;
use pray_core::package_integrity::verify_package_signature;
use pray_core::registry::{RegistryPackageMetadata, RegistryPackageVersion};
use pray_core::{PrayError, PrayResult};
use pray_transport::{
    ArtifactRef, PeerConfig, PeerInfo, SyncDirection, TransportAdapter, TransportRegistry,
    TrustLevel,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn sync_command(root: PathBuf, peers: Vec<String>) -> PrayResult<()> {
    let peer_sources = if peers.is_empty() {
        load_sync_peers(&root)?
            .into_iter()
            .map(|peer| peer.url)
            .collect()
    } else {
        peers
    };

    if peer_sources.is_empty() {
        return Err(PrayError::Unsupported(
            "no federation peers configured".to_string(),
        ));
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| {
            PrayError::Unsupported(format!("failed to start sync runtime: {error}"))
        })?;

    let summary = runtime.block_on(async { synchronize_registry(&root, peer_sources).await })?;
    record_root_revision(&root, RevisionAction::Sync)?;

    println!(
        "Synchronized {} package(s) from {} peer(s); learned {} peer(s)",
        summary.packages, summary.peers, summary.known_peers
    );
    Ok(())
}

pub(crate) struct SyncSummary {
    peers: usize,
    packages: usize,
    known_peers: usize,
}

pub(crate) async fn synchronize_registry(
    root: &Path,
    peer_sources: Vec<String>,
) -> PrayResult<SyncSummary> {
    let registry = TransportRegistry::new();
    let mut pending_peers: VecDeque<String> = peer_sources.into_iter().collect();
    let mut visited_peers = BTreeSet::new();
    let mut known_peers = load_known_peer_records(root)?;
    let mut package_versions_by_name: BTreeMap<String, BTreeMap<String, RegistryPackageVersion>> =
        BTreeMap::new();
    let mut peer_count = 0usize;

    for peer_source in pending_peers.clone() {
        upsert_known_peer(
            &mut known_peers,
            PeerInfo {
                name: peer_source.clone(),
                url: peer_source,
                public: false,
            },
        );
    }

    while let Some(peer_source) = pending_peers.pop_front() {
        if !visited_peers.insert(peer_source.clone()) {
            continue;
        }
        if visited_peers.len() > pray_core::resource_limits::MAX_FEDERATION_PEERS {
            return Err(PrayError::Resolution(format!(
                "federation sync exceeded {} peers",
                pray_core::resource_limits::MAX_FEDERATION_PEERS
            )));
        }
        peer_count += 1;
        let peer = federation_peer_config(&peer_source);
        let transport = registry.create(&peer).map_err(map_transport_error)?;
        let discovery = transport
            .fetch_discovery(&peer)
            .await
            .map_err(map_transport_error)?;
        if discovery.spec != "pray-federation-v1" {
            return Err(PrayError::Resolution(format!(
                "peer {peer_source} does not speak the pray federation protocol"
            )));
        }

        for discovered_peer in discovery.peers {
            let discovered_peer = normalize_peer_info(discovered_peer)?;
            if discovered_peer.url == peer_source {
                continue;
            }
            upsert_known_peer(&mut known_peers, discovered_peer.clone());
            if !visited_peers.contains(&discovered_peer.url)
                && !pending_peers
                    .iter()
                    .any(|queued| queued == &discovered_peer.url)
            {
                pending_peers.push_back(discovered_peer.url.clone());
            }
        }

        let index = transport
            .fetch_index(&peer, None)
            .await
            .map_err(map_transport_error)?;
        if index.spec != "prayfile-distribution-1" {
            return Err(PrayError::Resolution(format!(
                "peer {peer_source} returned unsupported registry index spec: {}",
                index.spec
            )));
        }

        for package_summary in index.packages {
            let metadata = transport
                .fetch_package(&peer, &package_summary.name)
                .await
                .map_err(map_transport_error)?;
            if metadata.name != package_summary.name {
                return Err(PrayError::Resolution(format!(
                    "peer {peer_source} returned mismatched package metadata for {}",
                    package_summary.name
                )));
            }
            sync_package_from_peer(
                root,
                &peer,
                transport.as_ref(),
                metadata,
                &mut package_versions_by_name,
            )
            .await?;
        }
    }

    write_known_peer_records(root, &known_peers)?;

    let mut local_index = load_registry_index(root)?;
    let mut package_names: BTreeSet<String> = local_index.packages.into_iter().collect();
    for (package_name, version_map) in &package_versions_by_name {
        write_synced_package_metadata(root, package_name, version_map)?;
        package_names.insert(package_name.clone());
    }
    local_index.packages = package_names.into_iter().collect();
    write_registry_index(root, &local_index)?;

    Ok(SyncSummary {
        peers: peer_count,
        packages: package_versions_by_name.len(),
        known_peers: known_peers.len(),
    })
}

pub(crate) async fn sync_package_from_peer(
    root: &Path,
    peer: &PeerConfig,
    transport: &dyn TransportAdapter,
    metadata: pray_transport::PackageMetadata,
    package_versions_by_name: &mut BTreeMap<String, BTreeMap<String, RegistryPackageVersion>>,
) -> PrayResult<()> {
    if !package_versions_by_name.contains_key(&metadata.name) {
        let existing_versions = load_local_package_versions(root, &metadata.name)?;
        package_versions_by_name.insert(metadata.name.clone(), existing_versions);
    }
    let package_versions = package_versions_by_name
        .get_mut(&metadata.name)
        .expect("package versions should be initialized");

    for version in metadata.versions {
        let mut local_version = sync_package_version_from_transport(&version)?;
        if let Some(existing_version) = package_versions.get(&local_version.version) {
            if existing_version.same_identity(&local_version) {
                let mut merged_version = existing_version.clone();
                merged_version.merge_annotations_from(&local_version);
                package_versions.insert(local_version.version.clone(), merged_version);
                continue;
            }
            return Err(PrayError::Integrity(format!(
                "conflicting metadata for package {} version {}",
                metadata.name, local_version.version
            )));
        }

        let artifact_hash = local_version.artifact_hash.as_ref().ok_or_else(|| {
            PrayError::Integrity(format!(
                "federation package {} {} is missing an artifact hash",
                metadata.name, local_version.version
            ))
        })?;
        let artifact = ArtifactRef {
            name: metadata.name.clone(),
            version: local_version.version.clone(),
            url: local_version.artifact.clone(),
            hash: artifact_hash.clone(),
        };
        let bytes = transport
            .fetch_artifact(peer, &artifact)
            .await
            .map_err(map_transport_error)?;
        let computed_hash = sha256_prefixed(&bytes);
        if &computed_hash != artifact_hash {
            return Err(PrayError::Integrity(format!(
                "artifact hash mismatch for {} {}",
                metadata.name, local_version.version
            )));
        }
        let tree_hash = local_version.tree_hash.as_ref().ok_or_else(|| {
            PrayError::Integrity(format!(
                "federation package {} {} is missing a tree hash",
                metadata.name, local_version.version
            ))
        })?;
        verify_package_signature(
            &metadata.name,
            &local_version.version,
            &local_version,
            &bytes,
            tree_hash,
        )?;

        if local_version.derived_metadata.is_none() {
            local_version.derived_metadata =
                Some(derive_registry_derived_metadata_from_archive_bytes(&bytes)?);
        }

        write_synced_artifact(root, &local_version.artifact, &bytes)?;
        package_versions.insert(local_version.version.clone(), local_version);
    }

    Ok(())
}

pub(crate) fn load_local_package_versions(
    root: &Path,
    package_name: &str,
) -> PrayResult<BTreeMap<String, RegistryPackageVersion>> {
    let metadata_path = registry_metadata_path(root, package_name);
    let metadata = load_registry_package_metadata(&metadata_path, package_name)?;
    Ok(metadata
        .versions
        .into_iter()
        .map(|version| (version.version.clone(), version))
        .collect())
}

pub(crate) fn write_synced_package_metadata(
    root: &Path,
    package_name: &str,
    versions: &BTreeMap<String, RegistryPackageVersion>,
) -> PrayResult<()> {
    let metadata = RegistryPackageMetadata {
        name: package_name.to_string(),
        versions: versions.values().cloned().collect(),
    };
    let metadata_path = registry_metadata_path(root, package_name);
    write_registry_package_metadata(&metadata_path, &metadata)
}

pub(crate) fn write_synced_artifact(
    root: &Path,
    artifact_path: &str,
    bytes: &[u8],
) -> PrayResult<()> {
    let relative = pray_core::paths::sanitize_relative_path(artifact_path)?;
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}

pub(crate) fn federation_peer_config(peer_source: &str) -> PeerConfig {
    let transport = if pray_core::ssh_client::is_pray_ssh_url(peer_source) {
        "ssh"
    } else {
        "http"
    };
    PeerConfig {
        name: peer_source.to_string(),
        transport: transport.to_string(),
        url: Some(peer_source.to_string()),
        trust: TrustLevel::Full,
        direction: SyncDirection::Pull,
        config: serde_json::json!({}),
    }
}

pub(crate) fn load_sync_peers(root: &Path) -> PrayResult<Vec<PeerInfo>> {
    let path = root.join("v1/peers.json");
    let text = fs::read_to_string(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PrayError::Unsupported("no federation peers configured".to_string())
        } else {
            PrayError::from(error)
        }
    })?;
    let peers: Vec<PeerInfo> = serde_json::from_str(&text).map_err(|error| PrayError::Parse {
        kind: "peer list",
        message: error.to_string(),
    })?;
    let mut normalized_peers = Vec::new();
    for peer in peers {
        normalized_peers.push(normalize_peer_info(peer)?);
    }
    Ok(normalized_peers)
}

pub(crate) fn sync_package_version_from_transport(
    version: &pray_transport::PackageVersion,
) -> PrayResult<RegistryPackageVersion> {
    if version.version.trim().is_empty() {
        return Err(PrayError::Resolution(
            "federation package version missing version string".to_string(),
        ));
    }
    if version.artifact.trim().is_empty() {
        return Err(PrayError::Resolution(format!(
            "federation package version {} missing artifact path",
            version.version
        )));
    }
    if version.artifact_hash.trim().is_empty() {
        return Err(PrayError::Integrity(format!(
            "federation package version {} missing artifact hash",
            version.version
        )));
    }

    let signer = version
        .publisher
        .as_ref()
        .map(|publisher| publisher.id.clone())
        .filter(|signer| !signer.trim().is_empty())
        .or_else(|| {
            version
                .signature
                .as_ref()
                .map(|signature| signature.public_key.clone())
                .filter(|signer| !signer.trim().is_empty())
        });

    let signer_fingerprint = version
        .publisher
        .as_ref()
        .map(|publisher| publisher.key_fingerprint.clone())
        .filter(|fingerprint| !fingerprint.trim().is_empty());

    Ok(RegistryPackageVersion {
        version: version.version.clone(),
        artifact: version.artifact.clone(),
        artifact_hash: Some(version.artifact_hash.clone()),
        tree_hash: if version.tree_hash.trim().is_empty() {
            None
        } else {
            Some(version.tree_hash.clone())
        },
        yanked: version.yanked,
        targets: version.targets.clone(),
        exports: version.exports.clone(),
        signer,
        signer_fingerprint,
        signer_public_key: version
            .signature
            .as_ref()
            .map(|signature| signature.public_key.clone())
            .filter(|value| !value.trim().is_empty()),
        published_at: Some(version.published_at.clone()),
        signature: version
            .signature
            .as_ref()
            .map(|signature| signature.signature.clone()),
        derived_metadata: version.derived_metadata.clone(),
    })
}
