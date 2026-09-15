use pray_core::embed::{build_lockfile, resolve_project_with_options, ResolveOptions};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_dir(relative: &str) -> PathBuf {
    workspace_root().join("fixtures").join(relative)
}

fn online_options() -> ResolveOptions {
    ResolveOptions {
        offline: false,
        ..ResolveOptions::default()
    }
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
    tree_hash: String,
    exports: Vec<String>,
}

fn read_json<T: for<'de> Deserialize<'de>>(dir: &Path) -> T {
    serde_json::from_str(&fs::read_to_string(dir.join("expected.json")).expect("expected.json"))
        .expect("expected json")
}

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn copy_fixture(relative: &str) -> PathBuf {
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let dest = std::env::temp_dir().join(format!(
        "pray-resolve-{}-{}-{}",
        relative.replace('/', "-"),
        std::process::id(),
        sequence
    ));
    copy_dir(&fixture_dir(relative), &dest);
    dest
}

fn copy_dir(src: &Path, dest: &Path) {
    fs::create_dir_all(dest).expect("dest");
    for entry in fs::read_dir(src).expect("read fixture") {
        let entry = entry.expect("entry");
        let target = dest.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy");
        }
    }
}

fn assert_lock_slice(dir: &Path, project: &pray_core::embed::ResolvedProject) {
    let expected: ExpectedResolve = read_json(dir);
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
                package.tree_hash.clone(),
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
                package.tree_hash,
                package.exports,
            )
        })
        .collect();
    expected_packages.sort();
    assert_eq!(packages, expected_packages);
}

#[test]
fn resolver_git_distribution_fixture() {
    let copied = copy_fixture("resolver/git-distribution");
    let project = resolve_project_with_options(&copied.join("Prayfile"), &online_options())
        .expect("git resolve");
    assert_lock_slice(&fixture_dir("resolver/git-distribution"), &project);
}

#[test]
fn resolver_registry_distribution_fixture() {
    let copied = copy_fixture("resolver/registry-distribution");
    let project = resolve_project_with_options(&copied.join("Prayfile"), &online_options())
        .expect("registry resolve");
    assert_lock_slice(&fixture_dir("resolver/registry-distribution"), &project);
}

#[test]
fn resolver_tarball_package_fixture() {
    let copied = copy_fixture("resolver/tarball-package");
    let project = resolve_project_with_options(&copied.join("Prayfile"), &offline_options())
        .expect("tarball resolve");
    assert_lock_slice(&fixture_dir("resolver/tarball-package"), &project);
}
