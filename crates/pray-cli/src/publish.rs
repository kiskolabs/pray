use crate::revision::{record_root_revision, RevisionAction};
use crate::sync_peers::map_transport_error;
use crate::transport_metadata::transport_package_metadata;
use crate::materialize::build_package_archive_bytes;
use crate::{
    current_signer, current_signer_fingerprint, current_timestamp, load_registry_index,
    load_registry_package_metadata, manifest_path, registry_artifact_path, registry_metadata_path,
    torrent_manifest_bytes, torrent_manifest_path, write_output_bytes, write_registry_index,
    write_registry_package_metadata, write_torrent_manifest,
};
use pray_core::derived_metadata::derive_registry_derived_metadata_from_archive_bytes;
use pray_core::hashing::sha256_prefixed;
use pray_core::package_integrity::{package_signature_for_publish, resolve_publish_signing_key};
use pray_core::registry::{
    upload_registry_artifact, RegistryPackageMetadata, RegistryPackageVersion,
};
use pray_core::resolve::{resolve_project, ResolvedProject};
use pray_core::ssh_identity::signing_identity;
use pray_core::{PrayError, PrayResult};
use pray_transport::{
    PeerConfig, SyncDirection, TransportRegistry,
    TrustLevel,
};
use std::path::{Path, PathBuf};

pub(crate) fn publish_command(
    roots: Vec<PathBuf>,
    servers: Vec<String>,
    signing_key_path: Option<PathBuf>,
) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let signer = current_signer()?;
    let signer_fingerprint = current_signer_fingerprint();
    let published_at = current_timestamp()?;
    let signing_key = resolve_publish_signing_key(signing_key_path.as_deref())?;
    let runtime = if servers.is_empty() {
        None
    } else {
        Some(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| {
                    PrayError::Unsupported(format!("failed to start publish runtime: {error}"))
                })?,
        )
    };

    for root in roots {
        publish_to_root(
            &project,
            &signer,
            signer_fingerprint.as_deref(),
            &published_at,
            signing_key.as_ref(),
            &root,
        )?;
        record_root_revision(&root, RevisionAction::Publish)?;
    }
    if let Some(runtime) = &runtime {
        for server_url in servers {
            publish_to_server(
                &project,
                &signer,
                signer_fingerprint.as_deref(),
                &published_at,
                signing_key.as_ref(),
                &server_url,
                runtime,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn publish_to_root(
    project: &ResolvedProject,
    signer: &str,
    signer_fingerprint: Option<&str>,
    published_at: &str,
    signing_key: Option<&ed25519_dalek::SigningKey>,
    root: &Path,
) -> PrayResult<()> {
    let mut registry_index = load_registry_index(root)?;
    let mut package_names = registry_index
        .packages
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();

    for package in &project.packages {
        let archive_bytes = build_package_archive_bytes(package)?;
        let artifact_path =
            registry_artifact_path(&package.declaration.name, &package.spec.version);
        let artifact_output_path = root.join(&artifact_path);
        write_output_bytes(&artifact_output_path, &archive_bytes)?;
        write_torrent_manifest(root, package, &artifact_path, &archive_bytes)?;

        let metadata_path = registry_metadata_path(root, &package.declaration.name);
        let mut metadata =
            load_registry_package_metadata(&metadata_path, &package.declaration.name)?;
        let version_entry = published_registry_package_version(
            package,
            signer,
            signer_fingerprint,
            published_at,
            signing_key,
            &artifact_path,
            &archive_bytes,
        )?;
        metadata
            .versions
            .retain(|entry| entry.version != version_entry.version);
        metadata.versions.push(version_entry);
        write_registry_package_metadata(&metadata_path, &metadata)?;
        package_names.insert(package.declaration.name.clone());
    }

    registry_index.packages = package_names.into_iter().collect();
    write_registry_index(root, &registry_index)?;
    Ok(())
}

pub(crate) fn published_registry_package_version(
    package: &pray_core::resolve::ResolvedPackage,
    signer: &str,
    signer_fingerprint: Option<&str>,
    published_at: &str,
    signing_key: Option<&ed25519_dalek::SigningKey>,
    artifact_path: &str,
    archive_bytes: &[u8],
) -> PrayResult<RegistryPackageVersion> {
    let signing_identity = signing_identity(signer, signer_fingerprint);
    let artifact_hash = sha256_prefixed(archive_bytes);
    let signature_material = package_signature_for_publish(
        signing_key,
        archive_bytes,
        &artifact_hash,
        &package.tree_hash,
        &signing_identity,
    );
    Ok(RegistryPackageVersion {
        version: package.spec.version.clone(),
        artifact: artifact_path.to_string(),
        artifact_hash: Some(artifact_hash),
        tree_hash: Some(package.tree_hash.clone()),
        yanked: false,
        targets: package.spec.targets.clone(),
        exports: package.spec.exports.keys().cloned().collect(),
        signer: Some(signer.to_string()),
        signer_fingerprint: signer_fingerprint.map(str::to_string),
        signer_public_key: signature_material.signer_public_key,
        published_at: Some(published_at.to_string()),
        signature: Some(signature_material.signature),
        derived_metadata: Some(derive_registry_derived_metadata_from_archive_bytes(
            archive_bytes,
        )?),
    })
}

pub(crate) fn publish_to_server(
    project: &ResolvedProject,
    signer: &str,
    signer_fingerprint: Option<&str>,
    published_at: &str,
    signing_key: Option<&ed25519_dalek::SigningKey>,
    server_url: &str,
    runtime: &tokio::runtime::Runtime,
) -> PrayResult<()> {
    if pray_core::ssh_client::is_pray_ssh_url(server_url) {
        return publish_to_ssh_server(
            project,
            signer,
            signer_fingerprint,
            published_at,
            signing_key,
            server_url,
        );
    }

    let peer = PeerConfig {
        name: server_url.to_string(),
        transport: "http".to_string(),
        url: Some(server_url.to_string()),
        trust: TrustLevel::Full,
        direction: SyncDirection::Push,
        config: serde_json::json!({}),
    };
    let registry = TransportRegistry::new();
    let transport = registry.create(&peer).map_err(map_transport_error)?;

    for package in &project.packages {
        let archive_bytes = build_package_archive_bytes(package)?;
        let artifact_path =
            registry_artifact_path(&package.declaration.name, &package.spec.version);
        upload_registry_artifact(server_url, &artifact_path, &archive_bytes)?;
        upload_registry_artifact(
            server_url,
            &torrent_manifest_path(&artifact_path),
            &torrent_manifest_bytes(package, &artifact_path, &archive_bytes)?,
        )?;

        let metadata = RegistryPackageMetadata {
            name: package.declaration.name.clone(),
            versions: vec![published_registry_package_version(
                package,
                signer,
                signer_fingerprint,
                published_at,
                signing_key,
                &artifact_path,
                &archive_bytes,
            )?],
        };
        let transport_metadata = transport_package_metadata(&metadata);
        runtime
            .block_on(transport.push_package(&peer, &transport_metadata))
            .map_err(map_transport_error)?;
    }

    Ok(())
}

pub(crate) fn publish_to_ssh_server(
    project: &ResolvedProject,
    signer: &str,
    signer_fingerprint: Option<&str>,
    published_at: &str,
    signing_key: Option<&ed25519_dalek::SigningKey>,
    server_url: &str,
) -> PrayResult<()> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use pray_core::ssh_client::with_pray_ssh_session;
    use serde_json::json;

    with_pray_ssh_session(server_url, |session| {
        for package in &project.packages {
            let archive_bytes = build_package_archive_bytes(package)?;
            let artifact_path =
                registry_artifact_path(&package.declaration.name, &package.spec.version);
            session.call_json(
                "artifact.put",
                json!({
                    "path": artifact_path,
                    "body": STANDARD.encode(&archive_bytes),
                }),
            )?;
            let torrent_path = torrent_manifest_path(&artifact_path);
            session.call_json(
                "artifact.put",
                json!({
                    "path": torrent_path,
                    "body": STANDARD.encode(&torrent_manifest_bytes(
                        package,
                        &artifact_path,
                        &archive_bytes,
                    )?),
                }),
            )?;

            let metadata = RegistryPackageMetadata {
                name: package.declaration.name.clone(),
                versions: vec![published_registry_package_version(
                    package,
                    signer,
                    signer_fingerprint,
                    published_at,
                    signing_key,
                    &artifact_path,
                    &archive_bytes,
                )?],
            };
            let transport_metadata = transport_package_metadata(&metadata);
            session.call_json(
                "sync.push",
                json!({
                    "metadata": transport_metadata,
                }),
            )?;
        }
        Ok(())
    })
}
