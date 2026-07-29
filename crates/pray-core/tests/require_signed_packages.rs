use pray_core::client_trust::{
    enforce_require_signed_packages, save_policy, ClientTrustPolicy, ClientTrustRule,
};
use pray_core::registry::RegistryPackageVersion;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn require_signed_packages_fails_when_signature_missing() {
    let home = temporary_home("require-signed-packages");
    save_policy(
        &home,
        &ClientTrustPolicy {
            default: ClientTrustRule {
                require_signed_packages: true,
                ..ClientTrustRule::default()
            },
            rules: Vec::new(),
        },
    )
    .expect("save");
    std::env::set_var("PRAY_HOME", &home);

    let selected = RegistryPackageVersion {
        version: "1.0.0".to_string(),
        signature: None,
        ..RegistryPackageVersion::default()
    };
    let error = enforce_require_signed_packages("https://example.test", "sample/base", &selected)
        .expect_err("unsigned rejected");
    assert!(error.to_string().contains("requires signed packages"));

    let signed = RegistryPackageVersion {
        version: "1.0.0".to_string(),
        signature: Some("ed25519:abc".to_string()),
        ..RegistryPackageVersion::default()
    };
    enforce_require_signed_packages("https://example.test", "sample/base", &signed)
        .expect("signed accepted");

    std::env::remove_var("PRAY_HOME");
    let _ = fs::remove_dir_all(&home);
}

fn temporary_home(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "pray-{prefix}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("home");
    path
}
