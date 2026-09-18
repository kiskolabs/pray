use pray_core::derived_metadata::derive_registry_derived_metadata_from_archive_bytes;
use pray_core::hashing::sha256_prefixed;
use pray_core::package_integrity::package_signature_for_publish;
use pray_core::registry::RegistryPackageVersion;
use pray_core::resolve::ResolvedPackage;
use pray_core::ssh_identity::signing_identity;
use pray_core::PrayResult;

pub(crate) fn published_registry_package_version(
    package: &ResolvedPackage,
    signer: &str,
    signer_fingerprint: Option<&str>,
    published_at: u64,
    signing_key: Option<&ed25519_dalek::SigningKey>,
    artifact_path: &str,
    archive_bytes: &[u8],
) -> PrayResult<RegistryPackageVersion> {
    let artifact_hash = sha256_prefixed(archive_bytes);
    let signature_material = package_signature_for_publish(
        signing_key,
        archive_bytes,
        &artifact_hash,
        &package.tree_hash,
        &signing_identity(signer, signer_fingerprint),
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
        published_at: Some(published_at),
        signature: Some(signature_material.signature),
        derived_metadata: Some(derive_registry_derived_metadata_from_archive_bytes(
            archive_bytes,
        )?),
    })
}
