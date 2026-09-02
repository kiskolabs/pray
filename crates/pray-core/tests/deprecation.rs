use pray_core::deprecation::package_spec_deprecation_warnings;
use pray_core::manifest::parse_manifest;
use pray_core::package_spec::parse_package_spec;

#[test]
fn parses_legacy_target_output_agent_with_deprecation_warnings() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", "~> 1.0"
"#,
    )
    .expect("parse");

    assert_eq!(
        manifest.deprecated_keywords,
        vec![
            "target".to_string(),
            "output".to_string(),
            "agent".to_string()
        ]
    );
    let warnings = manifest.deprecation_warnings();
    assert_eq!(warnings.len(), 3);
    assert!(warnings.iter().any(|warning| warning.contains("`target`")));
    assert!(warnings.iter().any(|warning| warning.contains("`output`")));
    assert!(warnings.iter().any(|warning| warning.contains("`agent`")));
    assert!(warnings.iter().all(|warning| warning.contains("version 2")));
}

#[test]
fn parses_legacy_skills_keyword_with_deprecation_warning() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
target :tool_a do
  skills ".agents/skills"
end
compose "AGENTS.md" do
end
tree ".agents/skills" do
end
skills ".agents/vendor" do
  pray "sample/base", "~> 1.0"
end
"#,
    )
    .expect("parse");

    assert!(manifest
        .deprecated_keywords
        .iter()
        .any(|keyword| keyword == "skills"));
    let warnings = manifest.deprecation_warnings();
    assert!(warnings.iter().any(|warning| warning.contains("`skills`")));
    assert!(warnings.iter().any(|warning| warning.contains("`tree`")));
}

#[test]
fn folder_export_type_is_not_a_prayfile_deprecation() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
tree ".agents/skills" do
  pray "sample/base", "~> 1.0"
end
"#,
    )
    .expect("parse");
    assert!(!manifest.deprecated_keywords.iter().any(|k| k == "skills"));
}

#[test]
fn spec_skills_and_skill_export_type_warn() {
    let spec = parse_package_spec(
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/legacy"
  spec.version = "1.0.0"
  spec.files = ["folders/review/README.md"]
  spec.exports = {
    "review" => { type: "skill", path: "folders/review" }
  }
  spec.skills = {
    "other" => { path: "folders/other", summary: "other" }
  }
end
"#,
    )
    .expect("spec");
    let warnings = package_spec_deprecation_warnings(&spec);
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("`spec.skills`")),
        "{warnings:?}"
    );
    assert!(
        warnings.iter().any(|warning| warning.contains("`skill`")),
        "{warnings:?}"
    );
    assert!(warnings.iter().all(|warning| warning.contains("version 2")));
}

#[test]
fn recommended_compose_pray_forms_are_not_deprecated() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
compose "AGENTS.md" do
  pray "sample/base", "~> 1.0"
end
"#,
    )
    .expect("parse");
    assert!(manifest.deprecated_keywords.is_empty());
    assert!(manifest.deprecation_warnings().is_empty());
}

#[test]
fn top_level_output_compose_alias_is_deprecated() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
output "AGENTS.md" do
  pray "sample/base", "~> 1.0"
end
"#,
    )
    .expect("parse");
    assert_eq!(manifest.deprecated_keywords, vec!["output".to_string()]);
}
