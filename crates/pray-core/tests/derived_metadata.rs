use pray_core::derived_metadata::derive_registry_derived_metadata_from_root;
use pray_core::package_spec::parse_package_spec;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn derives_summary_topics_and_embeddings_from_package_contents() {
    let root = unique_temp_dir("pray-core-derived-metadata");
    fs::write(
        root.join("package.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
end
"#,
    )
    .expect("write package spec");
    fs::create_dir_all(root.join("exports")).expect("create export directory");
    fs::write(root.join("README.md"), "package readme\n").expect("write readme");
    fs::write(root.join("exports/testing-basics.md"), "Testing guidance\n").expect("write export");

    let spec_text = fs::read_to_string(root.join("package.prayspec")).expect("read package spec");
    let package = parse_package_spec(&spec_text).expect("parse package spec");
    assert_eq!(package.name, "sample/base");

    let derived =
        derive_registry_derived_metadata_from_root(&root).expect("derive package metadata");

    assert!(derived.summary.contains("shared guidance"));
    assert!(derived.summary.contains("Testing guidance"));
    assert!(derived.topics.iter().any(|topic| topic == "guidance"));
    assert!(derived.topics.iter().any(|topic| topic == "testing"));
    assert!(derived
        .categories
        .iter()
        .any(|category| category == "documentation"));
    assert!(derived
        .categories
        .iter()
        .any(|category| category == "testing"));
    assert_eq!(derived.file_count, Some(2));
    assert!(derived.character_count.expect("character count") > 0);
    assert!(derived.token_count.expect("token count") > 0);
    assert_eq!(derived.embeddings.len(), 1);
    assert_eq!(derived.embeddings[0].model, "pray-hash-bucket-v1");
    assert_eq!(derived.embeddings[0].vector.len(), 16);
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{unique}"));
    fs::create_dir_all(&path).expect("create temp test directory");
    path
}
