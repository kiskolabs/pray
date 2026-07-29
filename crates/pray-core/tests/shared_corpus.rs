use pray_core::manifest::{
    parse_manifest, DestinationEntry, DestinationMode, ExportRole, Manifest,
};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn corpus_root() -> PathBuf {
    workspace_root().join("testdata/shared/manifest")
}

#[derive(Debug, Deserialize)]
struct ExpectedCorpus {
    targets: Vec<ExpectedTarget>,
    packages: Vec<ExpectedPackage>,
    local: Vec<ExpectedLocal>,
}

#[derive(Debug, Deserialize)]
struct ExpectedTarget {
    name: String,
    mode: String,
    scoped: bool,
    #[serde(default)]
    outputs: Vec<String>,
    #[serde(default)]
    skills: Vec<String>,
    entries: Vec<ExpectedEntry>,
}

#[derive(Debug, Deserialize)]
struct ExpectedEntry {
    kind: String,
    name: Option<String>,
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExpectedPackage {
    name: String,
    bound: bool,
    roles: Vec<String>,
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExpectedLocal {
    path: String,
    bound: bool,
}

fn assert_matches_expected(manifest: &Manifest, expected: &ExpectedCorpus) {
    assert_eq!(manifest.targets.len(), expected.targets.len());
    for (target, want) in manifest.targets.iter().zip(expected.targets.iter()) {
        assert_eq!(target.name, want.name);
        let mode = match want.mode.as_str() {
            "compose" => DestinationMode::Compose,
            "tree" => DestinationMode::Tree,
            "legacy" => DestinationMode::Legacy,
            other => panic!("unknown mode {other}"),
        };
        assert_eq!(target.mode, mode);
        assert_eq!(target.scoped, want.scoped);
        assert_eq!(target.outputs, want.outputs);
        assert_eq!(target.skills, want.skills);
        assert_eq!(target.entries.len(), want.entries.len());
        for (entry, want_entry) in target.entries.iter().zip(want.entries.iter()) {
            match (entry, want_entry.kind.as_str()) {
                (DestinationEntry::Local { path }, "local") => {
                    assert_eq!(path, want_entry.path.as_deref().unwrap_or_default());
                }
                (DestinationEntry::Package { name }, "package") => {
                    assert_eq!(name, want_entry.name.as_deref().unwrap_or_default());
                }
                _ => panic!("entry kind mismatch: {entry:?} vs {}", want_entry.kind),
            }
        }
    }

    assert_eq!(manifest.packages.len(), expected.packages.len());
    for (package, want) in manifest.packages.iter().zip(expected.packages.iter()) {
        assert_eq!(package.name, want.name);
        assert_eq!(package.bound, want.bound);
        assert_eq!(package.file, want.file);
        assert_eq!(package.path, want.path);
        let roles: Vec<String> = package
            .roles
            .iter()
            .map(|role| match role {
                ExportRole::Fragment => "fragment".to_string(),
                ExportRole::Folder => "folder".to_string(),
                ExportRole::File => "file".to_string(),
            })
            .collect();
        assert_eq!(roles, want.roles);
    }

    assert_eq!(manifest.local.len(), expected.local.len());
    for (local, want) in manifest.local.iter().zip(expected.local.iter()) {
        assert_eq!(local.path, want.path);
        assert_eq!(local.bound, want.bound);
    }
}

#[test]
fn shared_manifest_corpus_cases_parse() {
    let root = corpus_root();
    let mut cases = Vec::new();
    for entry in fs::read_dir(&root).expect("corpus root") {
        let entry = entry.expect("corpus entry");
        if entry.file_type().expect("type").is_dir() {
            cases.push(entry.file_name());
        }
    }
    cases.sort();
    assert!(
        !cases.is_empty(),
        "expected at least one shared corpus case under {}",
        root.display()
    );

    for case in cases {
        let dir = root.join(&case);
        let text = fs::read_to_string(dir.join("Prayfile"))
            .unwrap_or_else(|error| panic!("Prayfile in {:?}: {error}", case));
        let expected_text = fs::read_to_string(dir.join("expected.json"))
            .unwrap_or_else(|error| panic!("expected.json in {:?}: {error}", case));
        let expected: ExpectedCorpus = serde_json::from_str(&expected_text)
            .unwrap_or_else(|error| panic!("expected.json parses in {:?}: {error}", case));
        let manifest = parse_manifest(&text)
            .unwrap_or_else(|error| panic!("manifest parses in {:?}: {error}", case));
        assert_matches_expected(&manifest, &expected);
    }
}
