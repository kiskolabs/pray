use super::*;

fn locked_upstream(tree_hash: &str) -> LockedUpstream {
    LockedUpstream {
        name: "sample/base".to_string(),
        version: "1.4.3".to_string(),
        source: Some("sample".to_string()),
        tree_hash: tree_hash.to_string(),
        artifact_hash: "sha256:artifact".to_string(),
    }
}

#[test]
fn locked_upstream_rejects_republished_content() {
    let locked = locked_upstream("sha256:old");
    let resolved = locked_upstream("sha256:changed");
    let error = ensure_locked_upstream_matches(&locked, &resolved)
        .expect_err("changed locked content should fail");
    assert!(error
        .to_string()
        .contains("locked upstream tree hash mismatch"));
}

#[test]
fn failed_refresh_restores_deleted_content() {
    let root = std::env::temp_dir().join(format!(
        "pray-upstream-rollback-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("temporary root");
    std::fs::write(root.join("removed.md"), "old\n").expect("old content");
    let old = BTreeMap::from([("removed.md".to_string(), b"old\n".to_vec())]);
    let merged = BTreeMap::new();

    let result: PrayResult<()> = crate::transaction::run(&root, || {
        write_content_files(&root, &old, &merged)?;
        Err(PrayError::Resolution("later failure".to_string()))
    });

    assert!(result.is_err());
    assert_eq!(
        std::fs::read(root.join("removed.md")).expect("restored content"),
        b"old\n"
    );
    std::fs::remove_dir_all(root).expect("remove temporary root");
}

#[test]
fn refresh_rejects_content_paths_outside_the_package() {
    let workspace = std::env::temp_dir().join(format!(
        "pray-upstream-path-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let root = workspace.join("package");
    std::fs::create_dir_all(&root).expect("temporary root");
    std::fs::write(workspace.join("victim.md"), "keep\n").expect("victim");
    let old = BTreeMap::from([("../victim.md".to_string(), b"keep\n".to_vec())]);

    let error = write_content_files(&root, &old, &BTreeMap::new())
        .expect_err("escaping content path should fail");

    assert!(error.to_string().contains("escapes package root"));
    assert_eq!(
        std::fs::read(workspace.join("victim.md")).expect("victim remains"),
        b"keep\n"
    );
    std::fs::remove_dir_all(workspace).expect("remove temporary root");
}

#[test]
fn refresh_rejects_content_files_over_the_package_limit() {
    let root = std::env::temp_dir().join(format!(
        "pray-upstream-size-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("temporary root");
    let file = std::fs::File::create(root.join("large.md")).expect("large file");
    file.set_len(crate::resource_limits::MAX_ARCHIVE_ENTRY_BYTES + 1)
        .expect("sparse file");
    let spec = crate::package_spec::PackageSpec {
        files: vec!["large.md".to_string()],
        ..crate::package_spec::PackageSpec::default()
    };

    let error = content_file_bytes(&root, &spec).expect_err("oversized content should fail");

    assert!(error.to_string().contains("package file exceeds"));
    std::fs::remove_dir_all(root).expect("remove temporary root");
}

#[test]
fn refresh_rejects_too_many_content_files() {
    let spec = crate::package_spec::PackageSpec {
        files: (0..=crate::resource_limits::MAX_ARCHIVE_ENTRIES)
            .map(|index| format!("{index}.md"))
            .collect(),
        ..crate::package_spec::PackageSpec::default()
    };

    let error = content_file_bytes(std::path::Path::new("missing"), &spec)
        .expect_err("too many content files should fail");

    assert!(error.to_string().contains("package content exceeds"));
}
