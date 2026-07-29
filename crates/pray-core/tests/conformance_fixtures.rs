use pray_core::manifest::parse_manifest;
use pray_core::package_spec::parse_package_spec;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[derive(Debug, Deserialize)]
struct ExpectedManifest {
    prayfile_version: String,
    package_names: Vec<String>,
    target_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ExpectedPackage {
    name: String,
    version: String,
    files: Vec<String>,
}

#[test]
fn parser_minimal_prayfile_fixture() {
    let dir = workspace_root().join("fixtures/parser/minimal-prayfile");
    let text = fs::read_to_string(dir.join("Prayfile")).expect("Prayfile");
    let expected: ExpectedManifest = serde_json::from_str(
        &fs::read_to_string(dir.join("expected.json")).expect("expected.json"),
    )
    .expect("expected json");
    let manifest = parse_manifest(&text).expect("parse");
    assert_eq!(manifest.prayfile_version, expected.prayfile_version);
    let names: Vec<_> = manifest
        .packages
        .iter()
        .map(|package| package.name.clone())
        .collect();
    assert_eq!(names, expected.package_names);
    let targets: Vec<_> = manifest
        .targets
        .iter()
        .map(|target| target.name.clone())
        .collect();
    assert_eq!(targets, expected.target_names);
}

#[test]
fn prayspec_minimal_package_fixture() {
    let dir = workspace_root().join("fixtures/prayspec/minimal-package");
    let text = fs::read_to_string(dir.join("sample-base.prayspec")).expect("prayspec");
    let expected: ExpectedPackage = serde_json::from_str(
        &fs::read_to_string(dir.join("expected.json")).expect("expected.json"),
    )
    .expect("expected json");
    let package = parse_package_spec(&text).expect("parse");
    assert_eq!(package.name, expected.name);
    assert_eq!(package.version, expected.version);
    assert_eq!(package.files, expected.files);
}
