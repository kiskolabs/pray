use pray_core::registry::{
    registry_package_signing_identity, RegistryPackageMetadata, RegistryPackageVersion,
};
use pray_transport::{
    OriginInfo, PackageMetadata as TransportPackageMetadata, PackageVersion, PublisherInfo,
    SignatureInfo,
};

pub(crate) fn transport_package_metadata(
    metadata: &RegistryPackageMetadata,
) -> TransportPackageMetadata {
    let versions = metadata
        .versions
        .iter()
        .map(transport_package_version)
        .collect();
    TransportPackageMetadata {
        name: metadata.name.clone(),
        versions,
        updated_at: latest_publish_timestamp(metadata)
            .map(|timestamp| timestamp.to_string())
            .unwrap_or_else(|| "0".to_string()),
    }
}

pub(crate) fn transport_package_version(version: &RegistryPackageVersion) -> PackageVersion {
    let published_at = version
        .published_at
        .clone()
        .unwrap_or_else(|| "0".to_string());
    let publisher = match (
        version
            .signer_fingerprint
            .as_deref()
            .filter(|value| pray_core::ssh_identity::looks_like_ssh_fingerprint(value)),
        version.signer.as_deref(),
    ) {
        (Some(fingerprint), Some(label)) => Some(PublisherInfo {
            id: label.to_string(),
            key_fingerprint: pray_core::ssh_identity::normalize_identity(fingerprint),
        }),
        (_, Some(signer)) => Some(PublisherInfo {
            id: signer.to_string(),
            key_fingerprint: signer.to_string(),
        }),
        _ => None,
    };
    let signature = version.signature.as_ref().map(|signature| SignatureInfo {
        algorithm: "sha256".to_string(),
        signature: signature.clone(),
        public_key: registry_package_signing_identity(version).unwrap_or_default(),
    });
    let origin = version
        .published_at
        .as_ref()
        .map(|published_at| OriginInfo {
            server: "local".to_string(),
            first_seen: published_at.clone(),
        });
    PackageVersion {
        version: version.version.clone(),
        artifact: version.artifact.clone(),
        artifact_hash: version.artifact_hash.clone().unwrap_or_default(),
        tree_hash: version.tree_hash.clone().unwrap_or_default(),
        yanked: version.yanked,
        targets: version.targets.clone(),
        exports: version.exports.clone(),
        published_at,
        publisher,
        signature,
        origin,
        derived_metadata: version.derived_metadata.clone(),
    }
}

fn latest_publish_timestamp(metadata: &RegistryPackageMetadata) -> Option<u64> {
    metadata
        .versions
        .iter()
        .filter_map(|version| version.published_at.as_deref())
        .filter_map(|published_at| published_at.parse::<u64>().ok())
        .max()
}
