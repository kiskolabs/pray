use pray_core::derived_metadata::RegistryDerivedMetadata;
use pray_core::registry::{RegistryIndex, RegistryPackageMetadata, RegistryPackageVersion};
use pray_core::registry_search::search_local_registry;
use pray_core::resource_limits::MAX_HTTP_RESPONSE_BYTES;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn compact_index_bytes_scale_with_package_count() {
    let small = compact_index_json(200);
    let large = compact_index_json(1_000);
    let small_bytes = small.len();
    let large_bytes = large.len();
    let bytes_per_name = large_bytes as f64 / 1_000.0;
    let size_ratio = large_bytes as f64 / small_bytes as f64;
    assert!(
        (4.5..=5.5).contains(&size_ratio),
        "index size should track package count; got ratio {size_ratio:.2}"
    );
    assert!(
        (15.0..40.0).contains(&bytes_per_name),
        "unexpected compact name cost {bytes_per_name:.1} bytes"
    );
    let packages_at_http_ceiling = (MAX_HTTP_RESPONSE_BYTES as f64 / bytes_per_name) as u64;
    assert!(
        packages_at_http_ceiling > 1_500_000,
        "HTTP GET ceiling should admit more than 1.5M compact names; got {packages_at_http_ceiling}"
    );
}

#[test]
fn summary_search_reads_one_metadata_file_per_match() {
    let root = temporary_root("index-summary-fanout");
    let names: Vec<String> = (0..12)
        .map(|index| format!("sample/pkg-{index:04}"))
        .collect();
    let index = RegistryIndex {
        spec: "prayfile-distribution-1".to_string(),
        packages: names.clone(),
    };
    fs::create_dir_all(root.join("v1/packages/sample")).expect("dirs");
    fs::write(
        root.join("v1/index.json"),
        serde_json::to_vec(&index).expect("index json"),
    )
    .expect("index");
    for name in &names {
        let metadata = RegistryPackageMetadata {
            name: name.clone(),
            versions: vec![RegistryPackageVersion {
                version: "1.0.0".to_string(),
                artifact: format!("v1/artifacts/{name}/1.0.0/package.praypkg"),
                derived_metadata: Some(RegistryDerivedMetadata {
                    summary: format!("summary for {name}"),
                    ..RegistryDerivedMetadata::default()
                }),
                ..RegistryPackageVersion::default()
            }],
        };
        fs::write(
            root.join(format!("v1/packages/{name}.json")),
            serde_json::to_vec(&metadata).expect("metadata"),
        )
        .expect("metadata");
    }
    let hits = search_local_registry(&root, "sample/pkg-", true).expect("search");
    assert_eq!(hits.len(), 12);
    assert!(hits.iter().all(|hit| hit.summary.is_some()));
    let _ = fs::remove_dir_all(&root);
}

fn compact_index_json(package_count: usize) -> String {
    let index = RegistryIndex {
        spec: "prayfile-distribution-1".to_string(),
        packages: (0..package_count)
            .map(|index| format!("ns{index:04}/pkg{index:04}"))
            .collect(),
    };
    serde_json::to_string(&index).expect("index json")
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
