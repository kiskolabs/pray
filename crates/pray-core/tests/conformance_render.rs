use pray_core::embed::{
    inspect_locked_destinations, parse_lockfile, render_project, resolve_project_with_options,
    ResolveOptions,
};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_dir(relative: &str) -> PathBuf {
    workspace_root().join("fixtures").join(relative)
}

#[derive(Debug, Deserialize)]
struct ExpectedRender {
    dest_path: String,
    package_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ExpectedFindings {
    finding_kinds: Vec<String>,
}

fn read_json<T: for<'de> Deserialize<'de>>(dir: &Path) -> T {
    serde_json::from_str(&fs::read_to_string(dir.join("expected.json")).expect("expected.json"))
        .expect("expected json")
}

#[test]
fn render_compose_fragment_fixture() {
    let dir = fixture_dir("render/compose-fragment");
    let expected: ExpectedRender = read_json(&dir);
    let project = resolve_project_with_options(
        &dir.join("Prayfile"),
        &ResolveOptions {
            offline: true,
            ..ResolveOptions::default()
        },
    )
    .expect("resolve");
    let names: Vec<_> = project
        .packages
        .iter()
        .map(|package| package.declaration.name.clone())
        .collect();
    assert_eq!(names, expected.package_names);
    let rendered = render_project(&project).expect("render");
    assert_eq!(rendered.len(), 1);
    assert_eq!(
        rendered[0].path.to_string_lossy(),
        expected.dest_path.as_str()
    );
    let dest = fs::read_to_string(dir.join("expected").join(&expected.dest_path)).expect("dest");
    assert_eq!(rendered[0].content, dest);
}

#[test]
fn lockfile_span_matching_fixture() {
    assert_span_kinds("lockfile/span-matching");
}

#[test]
fn lockfile_span_edited_fixture() {
    assert_span_kinds("lockfile/span-edited");
}

#[test]
fn lockfile_span_missing_dest_fixture() {
    assert_span_kinds("lockfile/span-missing-dest");
}

#[test]
fn lockfile_span_removed_fixture() {
    assert_span_kinds("lockfile/span-removed");
}

#[test]
fn lockfile_span_orphan_fixture() {
    assert_span_kinds("lockfile/span-orphan");
}

fn assert_span_kinds(relative: &str) {
    let dir = fixture_dir(relative);
    let expected: ExpectedFindings = read_json(&dir);
    let lockfile =
        parse_lockfile(&fs::read_to_string(dir.join("Prayfile.lock")).expect("lockfile"))
            .expect("parse");
    let report = inspect_locked_destinations(&dir, &lockfile).expect("inspect");
    let mut kinds: Vec<_> = report
        .findings
        .iter()
        .map(|finding| finding.kind.clone())
        .collect();
    kinds.sort();
    let mut expected_kinds = expected.finding_kinds;
    expected_kinds.sort();
    assert_eq!(kinds, expected_kinds, "{:?}", report.findings);
}
