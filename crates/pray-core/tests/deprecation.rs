use pray_core::manifest::parse_manifest;

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
