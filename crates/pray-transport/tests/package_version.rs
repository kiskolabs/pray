use pray_transport::PackageVersion;
use serde_json::json;

fn package_version(published_at: Option<serde_json::Value>) -> serde_json::Value {
    let mut value = json!({
        "version": "1.0.0",
        "artifact": "v1/artifacts/example/pkg.praypkg",
        "artifact_hash": "sha256:abc",
        "tree_hash": "sha256:def",
        "yanked": false,
        "targets": [],
        "exports": []
    });
    if let Some(published_at) = published_at {
        value["published_at"] = published_at;
    }
    value
}

#[test]
fn package_version_omits_unknown_publish_timestamp() {
    let version: PackageVersion = serde_json::from_value(package_version(None)).unwrap();
    let canonical = serde_json::to_value(version).unwrap();

    assert!(canonical.get("published_at").is_none());
}

#[test]
fn package_version_normalizes_legacy_timestamps() {
    let version: PackageVersion =
        serde_json::from_value(package_version(Some(json!("1234567890")))).unwrap();
    assert_eq!(
        serde_json::to_value(version).unwrap()["published_at"],
        1_234_567_890_u64
    );

    let rfc3339: PackageVersion =
        serde_json::from_value(package_version(Some(json!("2020-01-01T00:00:00.999Z")))).unwrap();
    assert_eq!(
        serde_json::to_value(rfc3339).unwrap()["published_at"],
        1_577_836_800_u64
    );

    let absent: PackageVersion =
        serde_json::from_value(package_version(Some(json!(null)))).unwrap();
    assert!(serde_json::to_value(absent)
        .unwrap()
        .get("published_at")
        .is_none());
}
