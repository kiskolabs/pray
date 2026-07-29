use pray_core::manifest::parse_manifest;
use pray_core::package_spec::parse_package_spec;
use proptest::prelude::*;

proptest! {
    #[test]
    fn parse_manifest_does_not_panic(input in "\\PC*") {
        let _ = parse_manifest(&input);
    }

    #[test]
    fn parse_package_spec_does_not_panic(input in "\\PC*") {
        let _ = parse_package_spec(&input);
    }

    #[test]
    fn parse_manifest_lossy_bytes_do_not_panic(bytes in prop::collection::vec(any::<u8>(), 0..512)) {
        let input = String::from_utf8_lossy(&bytes);
        let _ = parse_manifest(&input);
    }

    #[test]
    fn parse_package_spec_lossy_bytes_do_not_panic(
        bytes in prop::collection::vec(any::<u8>(), 0..512)
    ) {
        let input = String::from_utf8_lossy(&bytes);
        let _ = parse_package_spec(&input);
    }
}

#[test]
fn known_valid_manifest_still_parses_under_fuzz_suite() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
pray "sample/base", "~> 1.0"
"#,
    )
    .expect("valid manifest");
    assert_eq!(manifest.packages[0].name, "sample/base");
}

#[test]
fn known_valid_package_spec_still_parses_under_fuzz_suite() {
    let package = parse_package_spec(
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.0.0"
  spec.files = ["README.md"]
end
"#,
    )
    .expect("valid package spec");
    assert_eq!(package.name, "sample/base");
}
