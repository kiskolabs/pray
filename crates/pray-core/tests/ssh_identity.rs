use pray_core::registry::{
    registry_artifact_signature, registry_package_signing_identity, RegistryPackageVersion,
};

#[test]
fn package_signature_uses_signer_fingerprint_when_present() {
    let version = RegistryPackageVersion {
        version: "1.0.0".to_string(),
        artifact: "v1/artifacts/sample/base/1.0.0/pkg.praypkg".to_string(),
        signer: Some("alice@example.com".to_string()),
        signer_fingerprint: Some("sha256:abc".to_string()),
        signature: None,
        ..RegistryPackageVersion::default()
    };
    let identity = registry_package_signing_identity(&version).expect("identity");
    assert_eq!(identity, "SHA256:ABC");
    let signature = registry_artifact_signature(b"artifact", "sha256:tree", &identity);
    assert!(!signature.is_empty());
}
