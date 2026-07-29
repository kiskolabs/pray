use ed25519_dalek::SigningKey;
use pray_core::hashing::sha256_prefixed;
use pray_core::package_integrity::{
    artifact_content_digest, package_signature_for_publish, require_remote_integrity_fields,
    sign_package_hashes, ssh_public_key_from_verifying_key, verify_package_signature,
    ED25519_SIGNATURE_PREFIX,
};
use pray_core::registry::RegistryPackageVersion;

#[test]
fn ed25519_package_hash_signatures_round_trip() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let artifact_hash = "sha256:artifact";
    let tree_hash = "sha256:tree";
    let signature = sign_package_hashes(&signing_key, artifact_hash, tree_hash);
    let public_key = ssh_public_key_from_verifying_key(&signing_key.verifying_key());
    let selected = RegistryPackageVersion {
        version: "1.0.0".to_string(),
        artifact: "v1/artifacts/pkg.praypkg".to_string(),
        artifact_hash: Some(artifact_hash.to_string()),
        tree_hash: Some(tree_hash.to_string()),
        signature: Some(signature),
        signer_public_key: Some(public_key),
        ..RegistryPackageVersion::default()
    };
    verify_package_signature("sample/base", "1.0.0", &selected, b"unused", tree_hash)
        .expect("ed25519 signature should verify");
}

#[test]
fn require_remote_integrity_fields_fails_when_hashes_missing() {
    let selected = RegistryPackageVersion {
        version: "1.0.0".to_string(),
        artifact: "v1/artifacts/pkg.praypkg".to_string(),
        ..RegistryPackageVersion::default()
    };
    let error = require_remote_integrity_fields("sample/base", "1.0.0", &selected)
        .expect_err("missing hashes");
    assert!(error.to_string().contains("artifact_hash"));
}

#[test]
fn package_signature_for_publish_prefers_ed25519_when_key_present() {
    let signing_key = SigningKey::from_bytes(&[9u8; 32]);
    let artifact_bytes = b"archive";
    let artifact_hash = sha256_prefixed(artifact_bytes);
    let tree_hash = "sha256:tree";
    let material = package_signature_for_publish(
        Some(&signing_key),
        artifact_bytes,
        &artifact_hash,
        tree_hash,
        "legacy-signer",
    );
    assert!(material.signature.starts_with(ED25519_SIGNATURE_PREFIX));
    assert_eq!(
        material.signer_public_key.as_deref(),
        Some(ssh_public_key_from_verifying_key(&signing_key.verifying_key()).as_str())
    );
    let selected = RegistryPackageVersion {
        version: "1.0.0".to_string(),
        artifact: "v1/artifacts/pkg.praypkg".to_string(),
        artifact_hash: Some(artifact_hash),
        tree_hash: Some(tree_hash.to_string()),
        signature: Some(material.signature),
        signer_public_key: material.signer_public_key,
        ..RegistryPackageVersion::default()
    };
    verify_package_signature("sample/base", "1.0.0", &selected, artifact_bytes, tree_hash)
        .expect("ed25519 publish signature should verify");
}

#[test]
fn package_signature_for_publish_falls_back_to_content_digest() {
    let artifact_bytes = b"archive";
    let artifact_hash = sha256_prefixed(artifact_bytes);
    let tree_hash = "sha256:tree";
    let material = package_signature_for_publish(
        None,
        artifact_bytes,
        &artifact_hash,
        tree_hash,
        "legacy-signer",
    );
    assert!(!material.signature.starts_with(ED25519_SIGNATURE_PREFIX));
    assert_eq!(material.signer_public_key, None);
    assert_eq!(
        material.signature,
        artifact_content_digest(artifact_bytes, tree_hash, "legacy-signer")
    );
}

#[test]
fn ed25519_signature_rejects_wrong_signer_key() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let other_key = SigningKey::from_bytes(&[8u8; 32]);
    let artifact_hash = "sha256:artifact";
    let tree_hash = "sha256:tree";
    let signature = sign_package_hashes(&signing_key, artifact_hash, tree_hash);
    let selected = RegistryPackageVersion {
        version: "1.0.0".to_string(),
        artifact: "v1/artifacts/pkg.praypkg".to_string(),
        artifact_hash: Some(artifact_hash.to_string()),
        tree_hash: Some(tree_hash.to_string()),
        signature: Some(signature),
        signer_public_key: Some(ssh_public_key_from_verifying_key(
            &other_key.verifying_key(),
        )),
        ..RegistryPackageVersion::default()
    };
    let error = verify_package_signature("sample/base", "1.0.0", &selected, b"unused", tree_hash)
        .expect_err("wrong key");
    assert!(error.to_string().contains("signature mismatch"));
}

#[test]
fn ed25519_signature_rejects_replay_against_different_hashes() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let artifact_hash = "sha256:artifact";
    let tree_hash = "sha256:tree";
    let signature = sign_package_hashes(&signing_key, artifact_hash, tree_hash);
    let selected = RegistryPackageVersion {
        version: "1.0.0".to_string(),
        artifact: "v1/artifacts/pkg.praypkg".to_string(),
        artifact_hash: Some("sha256:other-artifact".to_string()),
        tree_hash: Some(tree_hash.to_string()),
        signature: Some(signature),
        signer_public_key: Some(ssh_public_key_from_verifying_key(
            &signing_key.verifying_key(),
        )),
        ..RegistryPackageVersion::default()
    };
    let error = verify_package_signature("sample/base", "1.0.0", &selected, b"unused", tree_hash)
        .expect_err("replay");
    assert!(error.to_string().contains("signature mismatch"));
}

#[test]
fn content_digest_signature_rejects_tampered_artifact() {
    let artifact_bytes = b"archive";
    let tree_hash = "sha256:tree";
    let signature = artifact_content_digest(artifact_bytes, tree_hash, "legacy-signer");
    let selected = RegistryPackageVersion {
        version: "1.0.0".to_string(),
        artifact: "v1/artifacts/pkg.praypkg".to_string(),
        artifact_hash: Some(sha256_prefixed(artifact_bytes)),
        tree_hash: Some(tree_hash.to_string()),
        signature: Some(signature),
        signer: Some("legacy-signer".to_string()),
        ..RegistryPackageVersion::default()
    };
    let error = verify_package_signature("sample/base", "1.0.0", &selected, b"tampered", tree_hash)
        .expect_err("tampered");
    assert!(error.to_string().contains("signature mismatch"));
}
