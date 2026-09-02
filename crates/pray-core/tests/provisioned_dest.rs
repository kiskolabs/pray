use pray_core::lockfile::{build_lockfile, ProvisionedFileRecord};
use pray_core::paths::ProjectRelativePath;
use pray_core::render::{
    planned_provisioned_files, provisioned_lock_records, write_rendered_targets,
    write_rendered_targets_with_previous_lockfile,
};
use pray_core::resolve::resolve_project_in_context;
use pray_core::resolve_context::ResolveOptions;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{stamp}"))
}

fn write_file_package(root: &Path, body: &str) {
    let package_root = root.join("packages/shell");
    fs::create_dir_all(package_root.join("exports")).expect("dirs");
    fs::write(
        package_root.join("shell.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/shell"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["exports/zshrc"]
  spec.exports = {
    "zshrc" => { type: "file", path: "exports/zshrc" }
  }
end
"#,
    )
    .expect("prayspec");
    fs::write(package_root.join("exports/zshrc"), body).expect("export");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
pray "sample/shell", "~> 1.0", path: "packages/shell", file: ".zshrc"
"#,
    )
    .expect("prayfile");
}

fn resolve_root(root: &Path) -> pray_core::resolve::ResolvedProject {
    resolve_project_in_context(&root.join("Prayfile"), root, &ResolveOptions::default())
        .expect("resolve")
}

#[test]
fn destination_path_rejects_leading_tilde() {
    let path = ProjectRelativePath::parse("~fixtures/shell").expect("literal tilde path");
    assert_eq!(path.as_path(), Path::new("~fixtures/shell"));
    let error = pray_core::paths::validate_destination_path("~/.zshrc").expect_err("tilde");
    assert!(error.to_string().contains("repository-relative"));
}

#[test]
fn exclusive_file_writes_when_dest_is_missing() {
    let root = unique_temp_dir("pray-provisioned-missing");
    write_file_package(&root, "alias ll=ls\n");
    let project = resolve_root(&root);
    write_rendered_targets(&project, &[]).expect("write");
    assert_eq!(
        fs::read_to_string(root.join(".zshrc")).expect("dest"),
        "alias ll=ls\n"
    );
}

#[test]
fn exclusive_file_adopts_dest_when_bytes_already_match() {
    let root = unique_temp_dir("pray-provisioned-adopt");
    write_file_package(&root, "alias ll=ls\n");
    fs::write(root.join(".zshrc"), "alias ll=ls\n").expect("existing");
    let project = resolve_root(&root);
    write_rendered_targets(&project, &[]).expect("adopt");
    assert_eq!(
        fs::read_to_string(root.join(".zshrc")).expect("dest"),
        "alias ll=ls\n"
    );
}

#[test]
fn exclusive_file_refuses_to_clobber_unmanaged_bytes() {
    let root = unique_temp_dir("pray-provisioned-clobber");
    write_file_package(&root, "alias ll=ls\n");
    fs::write(root.join(".zshrc"), "keep me\n").expect("existing");
    let project = resolve_root(&root);
    let error = write_rendered_targets(&project, &[]).expect_err("clobber");
    assert!(error.to_string().contains(".zshrc"));
    assert_eq!(
        fs::read_to_string(root.join(".zshrc")).expect("kept"),
        "keep me\n"
    );
}

#[test]
fn exclusive_file_refuses_symlink_destination() {
    let root = unique_temp_dir("pray-provisioned-symlink");
    write_file_package(&root, "alias ll=ls\n");
    let target = root.join("real-zshrc");
    fs::write(&target, "keep link target\n").expect("target");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, root.join(".zshrc")).expect("symlink");
    #[cfg(not(unix))]
    {
        let _ = (root, target);
        return;
    }
    let project = resolve_root(&root);
    let error = write_rendered_targets(&project, &[]).expect_err("symlink");
    assert!(error.to_string().contains("symbolic link"));
    assert_eq!(
        fs::read_to_string(target).expect("untouched"),
        "keep link target\n"
    );
}

#[test]
fn exclusive_file_refuses_symlinked_parent_directory() {
    let base = unique_temp_dir("pray-provisioned-parent-symlink");
    let root = base.join("project");
    let outside = base.join("outside");
    fs::create_dir_all(&root).expect("project");
    fs::create_dir_all(&outside).expect("outside");
    write_file_package(&root, "alias ll=ls\n");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
pray "sample/shell", "~> 1.0", path: "packages/shell", file: "linked/zshrc"
"#,
    )
    .expect("prayfile");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, root.join("linked")).expect("parent symlink");
    #[cfg(not(unix))]
    return;

    let project = resolve_root(&root);
    let error = write_rendered_targets(&project, &[]).expect_err("parent symlink");
    assert!(error.to_string().contains("symbolic link"));
    assert!(!outside.join("zshrc").exists());
}

#[test]
fn exclusive_file_updates_when_previous_lock_hash_matches() {
    let root = unique_temp_dir("pray-provisioned-update");
    write_file_package(&root, "alias ll=ls\n");
    let project = resolve_root(&root);
    write_rendered_targets(&project, &[]).expect("first write");
    let records = provisioned_lock_records(&project).expect("records");
    let lockfile = lockfile_with_provisioned(&project, records);

    fs::write(root.join("packages/shell/exports/zshrc"), "alias la=ls\n").expect("new export");
    let updated = resolve_root(&root);
    write_rendered_targets_with_previous_lockfile(&updated, &[], Some(&lockfile))
        .expect("managed update");
    assert_eq!(
        fs::read_to_string(root.join(".zshrc")).expect("dest"),
        "alias la=ls\n"
    );
}

#[test]
fn exclusive_file_refuses_user_edited_managed_dest() {
    let root = unique_temp_dir("pray-provisioned-edited");
    write_file_package(&root, "alias ll=ls\n");
    let project = resolve_root(&root);
    write_rendered_targets(&project, &[]).expect("first write");
    let lockfile = lockfile_with_provisioned(
        &project,
        provisioned_lock_records(&project).expect("records"),
    );
    fs::write(root.join(".zshrc"), "my aliases\n").expect("edit");
    fs::write(root.join("packages/shell/exports/zshrc"), "alias la=ls\n").expect("new export");
    let updated = resolve_root(&root);
    let error = write_rendered_targets_with_previous_lockfile(&updated, &[], Some(&lockfile))
        .expect_err("edited");
    assert!(error.to_string().contains(".zshrc"));
    assert_eq!(
        fs::read_to_string(root.join(".zshrc")).expect("kept"),
        "my aliases\n"
    );
}

#[test]
fn hash_gated_prune_deletes_matching_leaf_and_keeps_edited() {
    let root = unique_temp_dir("pray-provisioned-prune");
    write_file_package(&root, "alias ll=ls\n");
    let project = resolve_root(&root);
    write_rendered_targets(&project, &[]).expect("first write");
    let lockfile = lockfile_with_provisioned(
        &project,
        provisioned_lock_records(&project).expect("records"),
    );

    fs::write(root.join("Prayfile"), "prayfile \"1\"\n").expect("removed package");
    let empty = resolve_root(&root);
    write_rendered_targets_with_previous_lockfile(&empty, &[], Some(&lockfile)).expect("prune");
    assert!(!root.join(".zshrc").exists());

    write_file_package(&root, "alias ll=ls\n");
    let project = resolve_root(&root);
    write_rendered_targets(&project, &[]).expect("rewrite");
    let lockfile = lockfile_with_provisioned(
        &project,
        provisioned_lock_records(&project).expect("records"),
    );
    fs::write(root.join(".zshrc"), "my aliases\n").expect("edit");
    fs::write(root.join("Prayfile"), "prayfile \"1\"\n").expect("removed again");
    let empty = resolve_root(&root);
    write_rendered_targets_with_previous_lockfile(&empty, &[], Some(&lockfile))
        .expect("skip prune");
    assert_eq!(
        fs::read_to_string(root.join(".zshrc")).expect("kept"),
        "my aliases\n"
    );
}

#[test]
fn prune_rejects_a_lock_path_outside_the_project() {
    let root = unique_temp_dir("pray-provisioned-lock-escape");
    write_file_package(&root, "alias ll=ls\n");
    let project = resolve_root(&root);
    let outside = root.parent().expect("parent").join(format!(
        "{}-outside",
        root.file_name().expect("root name").to_string_lossy()
    ));
    fs::write(&outside, "keep me\n").expect("outside");
    let lockfile = lockfile_with_provisioned(
        &project,
        vec![ProvisionedFileRecord {
            path: format!(
                "../{}",
                outside.file_name().expect("outside name").to_string_lossy()
            ),
            content_hash: pray_core::hashing::sha256_prefixed(b"keep me\n"),
            package: "sample/shell".to_string(),
            export: "zshrc".to_string(),
        }],
    );

    let error = write_rendered_targets_with_previous_lockfile(&project, &[], Some(&lockfile))
        .expect_err("unsafe lock path");

    assert!(error.to_string().contains("escapes"));
    assert_eq!(fs::read_to_string(outside).expect("kept"), "keep me\n");
}

#[test]
fn provisioned_lock_records_include_path_hash_package_export() {
    let root = unique_temp_dir("pray-provisioned-lock");
    write_file_package(&root, "alias ll=ls\n");
    let project = resolve_root(&root);
    let planned = planned_provisioned_files(&project).expect("planned");
    assert_eq!(planned.len(), 1);
    let records = provisioned_lock_records(&project).expect("records");
    assert_eq!(
        records,
        vec![ProvisionedFileRecord {
            path: ".zshrc".to_string(),
            content_hash: pray_core::hashing::sha256_prefixed(b"alias ll=ls\n"),
            package: "sample/shell".to_string(),
            export: "zshrc".to_string(),
        }]
    );
}

fn lockfile_with_provisioned(
    project: &pray_core::resolve::ResolvedProject,
    provisioned: Vec<ProvisionedFileRecord>,
) -> pray_core::lockfile::Lockfile {
    let mut lockfile = build_lockfile(
        project.manifest_hash.clone(),
        None,
        &project.project_root,
        &project.manifest.sources,
        &project.manifest.targets,
        &[],
        &project.packages,
        &BTreeMap::new(),
        &BTreeMap::new(),
    );
    lockfile.provisioned = provisioned;
    lockfile
}
