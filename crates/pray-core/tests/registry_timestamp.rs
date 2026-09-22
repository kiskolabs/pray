use pray_core::registry::RegistryPackageMetadata;

fn package_with_timestamp(timestamp: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "name": "sample/base",
        "versions": [{
            "version": "1.0.0",
            "artifact": "v1/artifacts/sample/base/1.0.0/package.praypkg",
            "published_at": timestamp
        }]
    })
}

fn read_published_at(timestamp: serde_json::Value) -> Option<u64> {
    let metadata: RegistryPackageMetadata =
        serde_json::from_value(package_with_timestamp(timestamp)).expect("read timestamp");
    metadata.versions[0].published_at
}

#[test]
fn registry_reader_normalizes_legacy_publish_timestamps() {
    assert_eq!(
        read_published_at(serde_json::json!("1234567890")),
        Some(1_234_567_890)
    );
    assert_eq!(
        read_published_at(serde_json::json!("2020-01-01T00:00:00.999Z")),
        Some(1_577_836_800)
    );
    assert_eq!(
        read_published_at(serde_json::json!("2020-01-01T00:00:00Z")),
        Some(1_577_836_800)
    );
    assert_eq!(
        read_published_at(serde_json::json!("2020-01-01T01:00:00+01:00")),
        Some(1_577_836_800)
    );
    assert_eq!(
        read_published_at(serde_json::json!("2020-02-29T00:00:00Z")),
        Some(1_582_934_400)
    );
    assert_eq!(read_published_at(serde_json::Value::Null), None);
    assert_eq!(
        read_published_at(serde_json::json!(1_577_836_800_u64)),
        Some(1_577_836_800)
    );

    let metadata: RegistryPackageMetadata =
        serde_json::from_value(package_with_timestamp(serde_json::json!("1234567890")))
            .expect("legacy numeric timestamp");
    let canonical = serde_json::to_value(metadata).expect("canonical metadata");
    assert_eq!(canonical["versions"][0]["published_at"], 1_234_567_890_u64);

    let absent: RegistryPackageMetadata =
        serde_json::from_value(package_with_timestamp(serde_json::Value::Null)).expect("null");
    let omitted = serde_json::to_value(absent).expect("omit null timestamp");
    assert!(omitted["versions"][0].get("published_at").is_none());
}

#[test]
fn registry_reader_rejects_invalid_publish_timestamps() {
    for timestamp in [
        serde_json::json!("not-a-timestamp"),
        serde_json::json!("tomorrow"),
        serde_json::json!("2020-02-30T00:00:00Z"),
        serde_json::json!(1.5),
        serde_json::json!(-1),
        serde_json::json!(253_402_300_800_u64),
    ] {
        assert!(
            serde_json::from_value::<RegistryPackageMetadata>(package_with_timestamp(timestamp))
                .is_err()
        );
    }
}
