use pray_core::derived_metadata::RegistryDerivedMetadata;
use pray_core::registry::{RegistryIndex, RegistryPackageMetadata, RegistryPackageVersion};
use std::fs;
use std::path::Path;

pub fn package_name(index: usize) -> String {
    format!("ns{:04}/pkg{:04}", index, index)
}

pub fn compact_index(package_count: usize) -> RegistryIndex {
    RegistryIndex {
        spec: "prayfile-distribution-1".to_string(),
        packages: (0..package_count).map(package_name).collect(),
    }
}

pub fn compact_index_json(package_count: usize) -> String {
    serde_json::to_string(&compact_index(package_count)).expect("index json")
}

pub fn names_heap_bytes(index: &RegistryIndex) -> usize {
    index.packages.iter().map(String::len).sum()
}

pub fn metadata_with_versions(name: &str, version_count: usize) -> RegistryPackageMetadata {
    let versions = (0..version_count)
        .map(|index| RegistryPackageVersion {
            version: format!("1.{index}.0"),
            artifact: format!("v1/artifacts/{name}/1.{index}.0/package.praypkg"),
            artifact_hash: Some("sha256:a".to_string()),
            tree_hash: Some("sha256:t".to_string()),
            derived_metadata: Some(RegistryDerivedMetadata {
                summary: format!("summary {index}"),
                ..RegistryDerivedMetadata::default()
            }),
            ..RegistryPackageVersion::default()
        })
        .collect();
    RegistryPackageMetadata {
        name: name.to_string(),
        versions,
    }
}

pub fn write_search_fixture(root: &Path, package_count: usize, with_summaries: bool) {
    fs::create_dir_all(root.join("v1/packages")).expect("packages dir");
    let index = compact_index(package_count);
    fs::write(
        root.join("v1/index.json"),
        serde_json::to_vec(&index).expect("index json"),
    )
    .expect("write index");
    if !with_summaries {
        return;
    }
    for name in &index.packages {
        let namespace = name.split('/').next().expect("namespace");
        fs::create_dir_all(root.join("v1/packages").join(namespace)).expect("namespace");
        let metadata = metadata_with_versions(name, 1);
        fs::write(
            root.join(format!("v1/packages/{name}.json")),
            serde_json::to_vec(&metadata).expect("metadata"),
        )
        .expect("write metadata");
    }
}
