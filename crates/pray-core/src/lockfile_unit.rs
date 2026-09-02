use super::{
    build_lockfile, lockfiles_equivalent, normalize_lockfile_artifact, relative_lockfile_path,
    LockSource, LockedPackage, Lockfile,
};
use std::collections::BTreeMap;
use std::path::Path;

#[test]
fn build_lockfile_records_git_source_revision() {
    let mut source_revisions = BTreeMap::new();
    source_revisions.insert(
        "dist".to_string(),
        "abc123def4567890abc123def4567890abc123de".to_string(),
    );
    let lockfile = build_lockfile(
        "sha256:manifest".to_string(),
        None,
        Path::new("."),
        &[crate::manifest::ManifestSource {
            name: "dist".to_string(),
            kind: "git".to_string(),
            url: "git+https://example.com/dist.git".to_string(),
            subdir: None,
            rev: None,
            tag: None,
        }],
        &[],
        &[],
        &[],
        &source_revisions,
        &BTreeMap::new(),
    );
    assert_eq!(
        lockfile.source,
        vec![LockSource {
            name: "dist".to_string(),
            kind: "git".to_string(),
            url: "git+https://example.com/dist.git".to_string(),
            revision: Some("abc123def4567890abc123def4567890abc123de".to_string()),
            host_key_fingerprint: None,
        }]
    );
    let serialized = lockfile.serialized().expect("serialize lockfile");
    assert!(serialized.contains("revision ="));
}

#[test]
fn lockfiles_equivalent_ignores_field_order() {
    let left = Lockfile {
        manifest_hash: "sha256:manifest".to_string(),
        package: vec![LockedPackage {
            name: "alpha".to_string(),
            version: "1.0.0".to_string(),
            source: None,
            path: "packages/alpha".to_string(),
            tree_hash: "sha256:tree".to_string(),
            artifact_hash: "sha256:artifact".to_string(),
            artifact: "alpha-1.0.0.praypkg".to_string(),
            exports: vec!["SKILL.md".to_string()],
            dependencies: Vec::new(),
            signer_fingerprint: None,
        }],
        ..Lockfile::default()
    };
    let mut right = left.clone();
    right.package.reverse();
    assert!(lockfiles_equivalent(&left.canonicalized(), &right));
}

#[test]
fn relative_lockfile_path_strips_absolute_project_prefix() {
    let project_root = Path::new("/tmp/project");
    let package_root = Path::new("/tmp/project/./packages/base");
    assert_eq!(
        relative_lockfile_path(project_root, package_root),
        "./packages/base"
    );
}

#[test]
fn normalize_lockfile_artifact_relativizes_path_artifacts() {
    let project_root = Path::new("/tmp/project");
    let package_root = Path::new("/tmp/project/packages/base");
    assert_eq!(
        normalize_lockfile_artifact(
            project_root,
            "path:/tmp/project/./packages/base",
            package_root,
        ),
        "path:./packages/base"
    );
    assert_eq!(
        normalize_lockfile_artifact(
            project_root,
            "v1/artifacts/sample/base/1.0.0/package.praypkg",
            package_root,
        ),
        "v1/artifacts/sample/base/1.0.0/package.praypkg"
    );
}
