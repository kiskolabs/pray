use super::registry_cache_directory;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct CacheFixture {
    source_key: String,
    package_name: String,
    version: String,
    relative_path: String,
}

#[test]
fn registry_cache_path_matches_shared_fixture() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata/shared/registry-cache/identity-first.json");
    let fixture: CacheFixture =
        serde_json::from_str(&fs::read_to_string(fixture_path).expect("fixture"))
            .expect("valid fixture");

    let path = registry_cache_directory(
        Path::new("/project"),
        &fixture.source_key,
        &fixture.package_name,
        &fixture.version,
    )
    .expect("cache path");

    assert_eq!(path, Path::new("/project").join(fixture.relative_path));
}

#[test]
fn registry_cache_path_rejects_unsafe_identity_segments() {
    for package_name in [
        "sample",
        "sample/base/extra",
        "sample//base",
        "./base",
        "../base",
        "sample/..",
        r"sample\base",
    ] {
        assert!(
            registry_cache_directory(Path::new("/project"), "source", package_name, "1.0.0")
                .is_err(),
            "{package_name:?} should be rejected"
        );
    }

    for version in ["", ".", "..", "1/2", r"1\2"] {
        assert!(
            registry_cache_directory(Path::new("/project"), "source", "sample/base", version)
                .is_err(),
            "{version:?} should be rejected"
        );
    }
}
