use pray_core::embed::{build_lockfile, resolve_project_with_options, ResolveOptions};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_dir(relative: &str) -> PathBuf {
    workspace_root().join("fixtures").join(relative)
}

fn offline_options() -> ResolveOptions {
    ResolveOptions {
        offline: true,
        ..ResolveOptions::default()
    }
}

#[derive(Debug, Deserialize)]
struct ExpectedResolve {
    packages: Vec<ExpectedLockedPackage>,
}

#[derive(Debug, Deserialize)]
struct ExpectedLockedPackage {
    name: String,
    version: String,
    path: String,
    tree_hash: String,
    artifact: String,
    exports: Vec<String>,
}

fn read_json<T: for<'de> Deserialize<'de>>(dir: &Path) -> T {
    serde_json::from_str(&fs::read_to_string(dir.join("expected.json")).expect("expected.json"))
        .expect("expected json")
}

#[test]
fn resolver_path_package_fixture() {
    let dir = fixture_dir("resolver/path-package");
    let expected: ExpectedResolve = read_json(&dir);
    let project =
        resolve_project_with_options(&dir.join("Prayfile"), &offline_options()).expect("resolve");
    let lockfile = build_lockfile(
        project.manifest_hash.clone(),
        project.environment.clone(),
        &project.project_root,
        &project.manifest.sources,
        &project.manifest.targets,
        &[],
        &project.packages,
        &project.source_revisions,
        &project.source_host_keys,
    );
    let mut packages: Vec<_> = lockfile
        .package
        .iter()
        .map(|package| {
            (
                package.name.clone(),
                package.version.clone(),
                package.path.clone(),
                package.tree_hash.clone(),
                package.artifact.clone(),
                package.exports.clone(),
            )
        })
        .collect();
    packages.sort();
    let mut expected_packages: Vec<_> = expected
        .packages
        .into_iter()
        .map(|package| {
            (
                package.name,
                package.version,
                package.path,
                package.tree_hash,
                package.artifact,
                package.exports,
            )
        })
        .collect();
    expected_packages.sort();
    assert_eq!(packages, expected_packages);
}

#[test]
fn resolver_constraint_mismatch_fixture() {
    let dir = fixture_dir("resolver/constraint-mismatch");
    resolve_project_with_options(&dir.join("Prayfile"), &offline_options())
        .expect_err("constraint mismatch");
}

#[test]
fn resolver_dependency_cycle_fixture() {
    let dir = fixture_dir("resolver/dependency-cycle");
    resolve_project_with_options(&dir.join("Prayfile"), &offline_options())
        .expect_err("dependency cycle");
}
