use pray_core::package_spec::PackageSpec;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct TreeHashFixture {
    files: Vec<FixtureFile>,
    tree_hash: String,
}

#[derive(Deserialize)]
struct FixtureFile {
    path: String,
    content: String,
}

#[test]
fn tree_hash_matches_shared_byte_order_fixture() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata/shared/package-tree/byte-order.json");
    let fixture: TreeHashFixture =
        serde_json::from_str(&fs::read_to_string(fixture_path).expect("fixture"))
            .expect("valid fixture");
    let files = fixture
        .files
        .into_iter()
        .map(|file| (file.path, file.content.into_bytes()))
        .collect::<BTreeMap<_, _>>();

    assert_eq!(
        PackageSpec::tree_hash_from_file_bytes(&files).expect("tree hash"),
        fixture.tree_hash
    );
}
