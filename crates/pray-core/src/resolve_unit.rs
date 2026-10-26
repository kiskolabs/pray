use super::{discover_distribution_root, project_root_from_manifest};
use std::fs;
use std::path::Path;

#[test]
fn project_root_from_manifest_uses_cwd_for_bare_filename() {
    let root = project_root_from_manifest(Path::new("Prayfile"));
    assert_eq!(root, Path::new("."));
}

#[test]
fn project_root_from_manifest_uses_parent_directory() {
    let root = project_root_from_manifest(Path::new("examples/simple-project/Prayfile"));
    assert_eq!(root, Path::new("examples/simple-project"));
}

#[test]
fn discover_distribution_root_finds_root_and_prayers_subdirectory() {
    let workspace =
        std::env::temp_dir().join(format!("pray-discover-distribution-{}", std::process::id()));
    let _ = fs::remove_dir_all(&workspace);
    let repo_root = workspace.join("repo");
    let prayers_root = repo_root.join("prayers");
    fs::create_dir_all(prayers_root.join("v1/packages")).expect("prayers distribution");
    fs::create_dir_all(repo_root.join("v1/packages")).expect("root distribution");

    assert_eq!(
        discover_distribution_root(&repo_root),
        Some(repo_root.clone())
    );

    fs::remove_dir_all(repo_root.join("v1")).expect("remove root distribution");
    assert_eq!(discover_distribution_root(&repo_root), Some(prayers_root));
    let _ = fs::remove_dir_all(&workspace);
}

#[test]
fn discover_distribution_root_returns_none_without_registry_layout() {
    let workspace =
        std::env::temp_dir().join(format!("pray-discover-missing-{}", std::process::id()));
    let _ = fs::remove_dir_all(&workspace);
    let repo_root = workspace.join("repo");
    fs::create_dir_all(&repo_root).expect("repo root");
    assert_eq!(discover_distribution_root(&repo_root), None);
    let _ = fs::remove_dir_all(&workspace);
}
