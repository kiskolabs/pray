use pray_core::lockfile::{LockedPackage, Lockfile};
use pray_core::registry::{
    highest_registry_version, select_package_version_for_test, RegistryPackageMetadata,
    RegistryPackageVersion,
};
use pray_core::resolve_context::{PackageResolutionContext, ResolveOptions};
use std::collections::BTreeSet;

#[test]
fn preferred_version_is_used_when_it_satisfies_constraint() {
    let metadata = RegistryPackageMetadata {
        name: "sample/base".to_string(),
        versions: vec![
            RegistryPackageVersion {
                version: "1.4.3".to_string(),
                artifact: "a".to_string(),
                ..RegistryPackageVersion::default()
            },
            RegistryPackageVersion {
                version: "2.0.0".to_string(),
                artifact: "b".to_string(),
                ..RegistryPackageVersion::default()
            },
        ],
    };
    let selected = select_package_version_for_test(&metadata, "~> 1.4", Some("1.4.3"))
        .expect("select version");
    assert_eq!(selected.version, "1.4.3");
}

#[test]
fn preferred_version_falls_back_when_constraint_no_longer_matches_lock() {
    let metadata = RegistryPackageMetadata {
        name: "sample/base".to_string(),
        versions: vec![
            RegistryPackageVersion {
                version: "1.4.3".to_string(),
                artifact: "a".to_string(),
                ..RegistryPackageVersion::default()
            },
            RegistryPackageVersion {
                version: "2.0.0".to_string(),
                artifact: "b".to_string(),
                ..RegistryPackageVersion::default()
            },
        ],
    };
    let selected = select_package_version_for_test(&metadata, "~> 2.0", Some("1.4.3"))
        .expect("select version");
    assert_eq!(selected.version, "2.0.0");
}

#[test]
fn highest_registry_version_reports_latest_even_when_constraint_caps_lower() {
    let metadata = RegistryPackageMetadata {
        name: "amkisko/working-rules".to_string(),
        versions: vec![
            RegistryPackageVersion {
                version: "1.0.0".to_string(),
                artifact: "a".to_string(),
                ..RegistryPackageVersion::default()
            },
            RegistryPackageVersion {
                version: "2.0.0".to_string(),
                artifact: "b".to_string(),
                ..RegistryPackageVersion::default()
            },
        ],
    };
    let selected = select_package_version_for_test(&metadata, "~> 1.0", None).expect("select");
    assert_eq!(selected.version, "1.0.0");
    let latest = highest_registry_version(&metadata)
        .expect("latest")
        .expect("version");
    assert_eq!(latest.version, "2.0.0");
}

#[test]
fn unlocked_packages_skip_locked_version_hints() {
    let mut unlocked = BTreeSet::new();
    unlocked.insert("sample/base".to_string());
    let lockfile = Lockfile {
        package: vec![LockedPackage {
            name: "sample/base".to_string(),
            version: "1.4.3".to_string(),
            source: None,
            path: ".".to_string(),
            tree_hash: "sha256:abc".to_string(),
            artifact_hash: "sha256:abc".to_string(),
            artifact: "path:.".to_string(),
            exports: vec![],
            dependencies: vec![],
            signer_fingerprint: None,
        }],
        ..Lockfile::default()
    };
    let mut options = ResolveOptions::default();
    options.unlocked_packages = unlocked;
    let context = PackageResolutionContext::from_lockfile(Some(&lockfile), "sample/base", &options);
    assert!(context.preferred_version.is_none());
}
