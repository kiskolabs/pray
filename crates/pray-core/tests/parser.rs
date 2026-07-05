use pray_core::manifest::parse_manifest;
use pray_core::package_spec::parse_package_spec;
use pray_core::PrayError;

#[test]
fn parses_minimal_manifest_example() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
source "default", "https://agents.example.com"
target :tool_a do
  output "INSTRUCTIONS.md"
  skills ".agents/skills"
end
agent "sample/base", "~> 1.4",
  exports: ["testing-basics", "security-basics"]
local ".agents/project.md"
render mode: :managed,
  conflict: :fail,
  churn: :minimal
"#,
    )
    .expect("manifest parses");

    assert_eq!(manifest.prayfile_version, "1");
    assert_eq!(manifest.sources[0].name, "default");
    assert_eq!(manifest.targets[0].name, "tool_a");
    assert_eq!(
        manifest.targets[0].outputs,
        vec!["INSTRUCTIONS.md".to_string()]
    );
    assert_eq!(manifest.packages[0].name, "sample/base");
    assert_eq!(manifest.local[0].path, ".agents/project.md");
    assert_eq!(manifest.render.mode, "managed");
}

#[test]
fn parses_minimal_package_spec_example() {
    let package = parse_package_spec(
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
  spec.add_dependency "sample/common", "~> 1.0"
end
"#,
    )
    .expect("package spec parses");

    assert_eq!(package.name, "sample/base");
    assert_eq!(package.version, "1.4.3");
    assert_eq!(
        package.files,
        vec![
            "README.md".to_string(),
            "exports/testing-basics.md".to_string()
        ]
    );
    assert_eq!(
        package.exports["testing-basics"].path,
        "exports/testing-basics.md"
    );
    assert_eq!(package.dependencies[0].name, "sample/common");
}

#[test]
fn parses_git_source_keyword_form() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
source "amkisko", git: "https://github.com/amkisko/prayers"
agent "amkisko/working-rules", "~> 1.0", source: "amkisko"
"#,
    )
    .expect("manifest parses");

    assert_eq!(manifest.sources.len(), 1);
    assert_eq!(manifest.sources[0].name, "amkisko");
    assert_eq!(manifest.sources[0].kind, "git");
    assert_eq!(
        manifest.sources[0].url,
        "git+https://github.com/amkisko/prayers"
    );
}

#[test]
fn rejects_manifest_without_prayfile_version() {
    let error = parse_manifest(
        r#"
target :tool_a do
  output "INSTRUCTIONS.md"
end
"#,
    )
    .expect_err("manifest should reject missing version");

    match error {
        PrayError::Manifest(message) => {
            assert!(message.contains("missing prayfile version"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn rejects_package_spec_without_end() {
    let error = parse_package_spec(
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
"#,
    )
    .expect_err("package spec should reject missing end");

    match error {
        PrayError::Parse { kind, message } => {
            assert_eq!(kind, "prayspec");
            assert!(message.contains("missing 'end'"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
