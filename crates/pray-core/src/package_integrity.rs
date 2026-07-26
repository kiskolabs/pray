use crate::hashing::sha256_prefixed;
use crate::registry::RegistryPackageVersion;
use crate::{PrayError, PrayResult};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use std::fs;
use std::path::Path;

pub const ED25519_SIGNATURE_PREFIX: &str = "ed25519:";
pub const SIGNING_KEY_ENV: &str = "PRAY_SIGNING_KEY";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSignatureMaterial {
    pub signature: String,
    pub signer_public_key: Option<String>,
}

pub fn require_remote_integrity_fields(
    package_name: &str,
    version: &str,
    selected: &RegistryPackageVersion,
) -> PrayResult<()> {
    if selected.artifact_hash.as_deref().unwrap_or("").is_empty() {
        return Err(PrayError::Integrity(format!(
            "package {package_name} {version} is missing artifact_hash"
        )));
    }
    if selected.tree_hash.as_deref().unwrap_or("").is_empty() {
        return Err(PrayError::Integrity(format!(
            "package {package_name} {version} is missing tree_hash"
        )));
    }
    Ok(())
}

pub fn artifact_content_digest(artifact_bytes: &[u8], tree_hash: &str, signer: &str) -> String {
    let mut payload = Vec::with_capacity(artifact_bytes.len() + tree_hash.len() + signer.len() + 2);
    payload.extend_from_slice(artifact_bytes);
    payload.push(0);
    payload.extend_from_slice(tree_hash.as_bytes());
    payload.push(0);
    payload.extend_from_slice(signer.as_bytes());
    sha256_prefixed(&payload)
}

pub fn package_hash_signing_payload(artifact_hash: &str, tree_hash: &str) -> Vec<u8> {
    let mut payload = Vec::with_capacity(artifact_hash.len() + tree_hash.len() + 1);
    payload.extend_from_slice(artifact_hash.as_bytes());
    payload.push(0);
    payload.extend_from_slice(tree_hash.as_bytes());
    payload
}

pub fn sign_package_hashes(
    signing_key: &SigningKey,
    artifact_hash: &str,
    tree_hash: &str,
) -> String {
    let payload = package_hash_signing_payload(artifact_hash, tree_hash);
    let signature = signing_key.sign(&payload);
    format!(
        "{ED25519_SIGNATURE_PREFIX}{}",
        STANDARD.encode(signature.to_bytes())
    )
}

pub fn load_ed25519_signing_key(path: &Path) -> PrayResult<SigningKey> {
    let private_key_bytes = fs::read(path).map_err(|error| {
        PrayError::Unsupported(format!(
            "failed to read signing key {}: {error}",
            path.display()
        ))
    })?;
    let seed: [u8; 32] = private_key_bytes.as_slice().try_into().map_err(|_| {
        PrayError::Unsupported(format!(
            "signing key {} must be 32 raw ed25519 seed bytes",
            path.display()
        ))
    })?;
    Ok(SigningKey::from_bytes(&seed))
}

pub fn resolve_publish_signing_key(explicit: Option<&Path>) -> PrayResult<Option<SigningKey>> {
    if let Some(path) = explicit {
        return Ok(Some(load_ed25519_signing_key(path)?));
    }
    match std::env::var(SIGNING_KEY_ENV) {
        Ok(path) if !path.trim().is_empty() => {
            Ok(Some(load_ed25519_signing_key(Path::new(path.trim()))?))
        }
        _ => Ok(None),
    }
}

pub fn package_signature_for_publish(
    signing_key: Option<&SigningKey>,
    artifact_bytes: &[u8],
    artifact_hash: &str,
    tree_hash: &str,
    signing_identity: &str,
) -> PackageSignatureMaterial {
    if let Some(key) = signing_key {
        return PackageSignatureMaterial {
            signature: sign_package_hashes(key, artifact_hash, tree_hash),
            signer_public_key: Some(ssh_public_key_from_verifying_key(&key.verifying_key())),
        };
    }
    PackageSignatureMaterial {
        signature: artifact_content_digest(artifact_bytes, tree_hash, signing_identity),
        signer_public_key: None,
    }
}

pub fn verifying_key_bytes_from_ssh_public_key(public_key: &str) -> PrayResult<[u8; 32]> {
    let mut fields = public_key.split_whitespace();
    let algorithm = fields.next().ok_or_else(|| {
        PrayError::Unsupported("public key must include an algorithm".to_string())
    })?;
    if algorithm != "ssh-ed25519" {
        return Err(PrayError::Unsupported(format!(
            "unsupported public key algorithm: {algorithm}"
        )));
    }
    let key_value = fields.next().ok_or_else(|| {
        PrayError::Unsupported("public key must include key bytes".to_string())
    })?;
    let blob = STANDARD
        .decode(key_value.as_bytes())
        .map_err(|error| PrayError::Parse {
            kind: "public key",
            message: error.to_string(),
        })?;
    let mut cursor = blob.as_slice();
    let blob_algorithm = read_ssh_string(&mut cursor)?;
    if blob_algorithm != b"ssh-ed25519" {
        return Err(PrayError::Parse {
            kind: "public key",
            message: "ed25519 public key blob must start with ssh-ed25519".to_string(),
        });
    }
    let key_bytes = read_ssh_string(&mut cursor)?;
    key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| PrayError::Parse {
            kind: "public key",
            message: "ed25519 public key must be 32 bytes".to_string(),
        })
}

pub fn ssh_public_key_from_verifying_key(verifying_key: &VerifyingKey) -> String {
    let mut blob = Vec::new();
    write_ssh_string(&mut blob, b"ssh-ed25519");
    write_ssh_string(&mut blob, verifying_key.as_bytes());
    format!("ssh-ed25519 {}", STANDARD.encode(blob))
}

pub fn verify_package_signature(
    package_name: &str,
    version: &str,
    selected: &RegistryPackageVersion,
    artifact_bytes: &[u8],
    tree_hash: &str,
) -> PrayResult<()> {
    let Some(signature) = selected.signature.as_deref() else {
        return Ok(());
    };
    if let Some(encoded) = signature.strip_prefix(ED25519_SIGNATURE_PREFIX) {
        let public_key = selected.signer_public_key.as_deref().ok_or_else(|| {
            PrayError::Integrity(format!(
                "package {package_name} {version} ed25519 signature missing signer_public_key"
            ))
        })?;
        let artifact_hash = selected.artifact_hash.as_deref().ok_or_else(|| {
            PrayError::Integrity(format!(
                "package {package_name} {version} is missing artifact_hash"
            ))
        })?;
        let key_bytes = verifying_key_bytes_from_ssh_public_key(public_key)?;
        let verifying_key =
            VerifyingKey::from_bytes(&key_bytes).map_err(|error| PrayError::Parse {
                kind: "public key",
                message: error.to_string(),
            })?;
        let signature_bytes =
            STANDARD
                .decode(encoded.as_bytes())
                .map_err(|error| PrayError::Parse {
                    kind: "signature",
                    message: error.to_string(),
                })?;
        let signature = Signature::from_slice(&signature_bytes)
            .map_err(|error| PrayError::Integrity(error.to_string()))?;
        let payload = package_hash_signing_payload(artifact_hash, tree_hash);
        verifying_key
            .verify(&payload, &signature)
            .map_err(|_| {
                PrayError::Integrity(format!(
                    "package signature mismatch for {package_name} {version}"
                ))
            })?;
        return Ok(());
    }

    let signing_identity = crate::ssh_identity::package_signing_identity(
        selected.signer.as_deref(),
        selected.signer_fingerprint.as_deref(),
    )
    .ok_or_else(|| {
        PrayError::Integrity(format!(
            "package signature missing signer for {package_name} {version}"
        ))
    })?;
    let expected = artifact_content_digest(artifact_bytes, tree_hash, &signing_identity);
    if expected != signature {
        return Err(PrayError::Integrity(format!(
            "package signature mismatch for {package_name} {version}"
        )));
    }
    Ok(())
}

fn read_ssh_string(cursor: &mut &[u8]) -> PrayResult<Vec<u8>> {
    if cursor.len() < 4 {
        return Err(PrayError::Resolution(
            "truncated ssh public key blob".to_string(),
        ));
    }
    let length = u32::from_be_bytes([cursor[0], cursor[1], cursor[2], cursor[3]]) as usize;
    *cursor = &cursor[4..];
    if cursor.len() < length {
        return Err(PrayError::Resolution(
            "truncated ssh public key blob".to_string(),
        ));
    }
    let (value, rest) = cursor.split_at(length);
    *cursor = rest;
    Ok(value.to_vec())
}

fn write_ssh_string(buffer: &mut Vec<u8>, bytes: &[u8]) {
    buffer.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buffer.extend_from_slice(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
