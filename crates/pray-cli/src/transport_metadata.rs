use pray_core::registry::{
    registry_package_signing_identity, RegistryPackageMetadata, RegistryPackageVersion,
};
use pray_core::{PrayError, PrayResult};
use pray_transport::{
    OriginInfo, PackageMetadata as TransportPackageMetadata, PackageVersion, PublisherInfo,
    SignatureInfo,
};
use std::collections::BTreeSet;

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
    let signature = version.signature.as_ref().map(|signature| {
        if signature.starts_with(pray_core::package_integrity::ED25519_SIGNATURE_PREFIX) {
            SignatureInfo {
                algorithm: "ed25519".to_string(),
                signature: signature.clone(),
                public_key: version.signer_public_key.clone().unwrap_or_default(),
            }
        } else {
            SignatureInfo {
                algorithm: "sha256".to_string(),
                signature: signature.clone(),
                public_key: registry_package_signing_identity(version).unwrap_or_default(),
            }
        }
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

pub(crate) fn registry_package_metadata_from_transport(
    metadata: &TransportPackageMetadata,
) -> PrayResult<RegistryPackageMetadata> {
    if metadata.name.trim().is_empty() {
        return Err(PrayError::Resolution(
            "federation package metadata missing package name".to_string(),
        ));
    }

    let mut seen_versions = BTreeSet::new();
    let mut versions = Vec::new();
    for version in &metadata.versions {
        let registry_version = registry_package_version_from_transport(version)?;
        if !seen_versions.insert(registry_version.version.clone()) {
            return Err(PrayError::Resolution(format!(
                "duplicate package version in federation payload: {} {}",
                metadata.name, registry_version.version
            )));
        }
        versions.push(registry_version);
    }

    Ok(RegistryPackageMetadata {
        name: metadata.name.clone(),
        versions,
    })
}

pub(crate) fn registry_package_version_from_transport(
    version: &PackageVersion,
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

    let signer = version
        .publisher
        .as_ref()
        .and_then(|publisher| {
            if publisher.id.trim().is_empty() {
                None
            } else {
                Some(publisher.id.clone())
            }
        })
        .or_else(|| {
            version
                .signature
                .as_ref()
                .map(|signature| signature.public_key.clone())
        })
        .filter(|signer| !signer.trim().is_empty());
    let signer_fingerprint = version
        .publisher
        .as_ref()
        .map(|publisher| publisher.key_fingerprint.clone())
        .filter(|fingerprint| !fingerprint.trim().is_empty());
    let signature = version
        .signature
        .as_ref()
        .map(|signature| signature.signature.clone())
        .filter(|signature| !signature.trim().is_empty());
    let published_at = if version.published_at.trim().is_empty() {
        None
    } else {
        Some(version.published_at.clone())
    };

    Ok(RegistryPackageVersion {
        version: version.version.clone(),
        artifact: version.artifact.clone(),
        artifact_hash: empty_string_to_none(&version.artifact_hash),
        tree_hash: empty_string_to_none(&version.tree_hash),
        yanked: version.yanked,
        targets: version.targets.clone(),
        exports: version.exports.clone(),
        signer,
        signer_fingerprint,
        signer_public_key: version
            .signature
            .as_ref()
            .map(|value| value.public_key.clone())
            .and_then(|value| empty_string_to_none(&value)),
        published_at,
        signature,
        derived_metadata: version.derived_metadata.clone(),
    })
}

fn empty_string_to_none(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}
