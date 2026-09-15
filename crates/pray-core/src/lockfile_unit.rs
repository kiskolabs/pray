use super::{
    build_lockfile, lockfile_hash, lockfiles_equivalent, normalize_lockfile_artifact,
    parse_lockfile, relative_lockfile_path, serialize_lockfile, LockSource, LockedPackage,
    Lockfile,
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
fn build_lockfile_records_package_upstream() {
    let lockfile = Lockfile {
        package: vec![LockedPackage {
            name: "fork/base".to_string(),
            version: "1.0.0".to_string(),
            source: None,
            path: "./packages/base".to_string(),
            tree_hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            artifact_hash:
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_string(),
            artifact: "path:./packages/base".to_string(),
            exports: Vec::new(),
            dependencies: Vec::new(),
            signer_fingerprint: None,
            upstream: Some(crate::package_upstream::LockedUpstream {
                name: "sample/base".to_string(),
                version: "1.4.3".to_string(),
                source: Some("sample".to_string()),
                tree_hash:
                    "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                        .to_string(),
                artifact_hash:
                    "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                        .to_string(),
            }),
        }],
        ..Lockfile::default()
    };
    let serialized = lockfile.serialized().expect("serialize lockfile");
    assert!(serialized.contains("sample/base"));
    assert!(serialized.contains("1.4.3"));
}

#[test]
fn parse_lockfile_round_trips_serialized_bytes() {
    let lockfile = Lockfile {
        prayfile_lock: "1".to_string(),
        spec: "0.1".to_string(),
        generated_by: "pray 1.15.0".to_string(),
        manifest_hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        ..Lockfile::default()
    };
    let serialized = serialize_lockfile(&lockfile).expect("serialize");
    let parsed = parse_lockfile(&serialized).expect("parse");
    assert!(lockfiles_equivalent(&lockfile.canonicalized(), &parsed));
    let digest = lockfile_hash(&parsed).expect("hash");
    assert!(digest.starts_with("sha256:"));
}

#[test]
fn parse_lockfile_rejects_invalid_toml() {
    let error = parse_lockfile("not = [lockfile").expect_err("invalid");
    assert!(error.to_string().contains("lockfile parse error"));
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
            upstream: None,
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
