use pray_core::lockfile::{LockedPackage, Lockfile};
use pray_core::registry::{
    select_package_version_for_test, RegistryPackageMetadata, RegistryPackageVersion,
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
