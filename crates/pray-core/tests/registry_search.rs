use pray_core::derived_metadata::RegistryDerivedMetadata;
use pray_core::registry::{RegistryIndex, RegistryPackageMetadata, RegistryPackageVersion};
use pray_core::registry_search::{
    latest_non_yanked_summary, search_local_registry, search_registry_index_names,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn search_matches_package_names_case_insensitively() {
    let index = RegistryIndex {
        spec: "prayfile-distribution-1".to_string(),
        packages: vec![
            "sample/base".to_string(),
            "sample/webapp".to_string(),
            "other/tool".to_string(),
        ],
    };
    assert_eq!(
        search_registry_index_names(&index, "SAMPLE"),
        vec!["sample/base".to_string(), "sample/webapp".to_string()]
    );
}

#[test]
fn local_search_includes_summary_from_metadata() {
    let root = temporary_root("registry-search");
    fs::create_dir_all(root.join("v1/packages/sample")).expect("dirs");
    fs::write(
        root.join("v1/index.json"),
        r#"{"spec":"prayfile-distribution-1","packages":["sample/base","sample/webapp"]}"#,
    )
    .expect("index");
    let metadata = RegistryPackageMetadata {
        name: "sample/base".to_string(),
        versions: vec![RegistryPackageVersion {
            version: "1.0.0".to_string(),
            artifact: "v1/artifacts/sample/base/1.0.0/package.praypkg".to_string(),
            artifact_hash: Some("sha256:a".to_string()),
            tree_hash: Some("sha256:t".to_string()),
            yanked: false,
            derived_metadata: Some(RegistryDerivedMetadata {
                summary: "shared guidance".to_string(),
                ..RegistryDerivedMetadata::default()
            }),
            ..RegistryPackageVersion::default()
        }],
    };
    fs::write(
        root.join("v1/packages/sample/base.json"),
        serde_json::to_string_pretty(&metadata).expect("json"),
    )
    .expect("metadata");

    let hits = search_local_registry(&root, "base", true).expect("search");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].name, "sample/base");
    assert_eq!(hits[0].summary.as_deref(), Some("shared guidance"));
    assert_eq!(
        latest_non_yanked_summary(&metadata).as_deref(),
        Some("shared guidance")
    );
    let _ = fs::remove_dir_all(&root);
}

fn temporary_root(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "pray-{prefix}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("root");
    path
}
