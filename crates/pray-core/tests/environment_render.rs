use pray_core::environment::{package_matches_environment, validate_environment};
use pray_core::manifest::parse_manifest;
use pray_core::render::render_project;
use pray_core::resolve::resolve_project_in_context;
use pray_core::resolve_context::ResolveOptions;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{stamp}"))
}

fn write_grouped_fixture(root: &Path) {
    fs::create_dir_all(root.join("packages/shared/exports")).expect("dirs");
    fs::create_dir_all(root.join("packages/dev-only/exports")).expect("dirs");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/shared", "~> 1.0", path: "packages/shared"
group :development, :test do
  agent "sample/dev-only", "~> 1.0", path: "packages/dev-only"
end
"#,
    )
    .expect("prayfile");
    for (directory, package_name, note) in [
        ("shared", "sample/shared", "note for shared"),
        ("dev-only", "sample/dev-only", "note for dev-only"),
    ] {
        let package_root = root.join(format!("packages/{directory}"));
        fs::write(
            package_root.join(format!("{directory}.prayspec")),
            format!(
                r#"
Package::Specification.new do |spec|
  spec.name = "{package_name}"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["exports/note.md"]
  spec.exports = {{
    "note" => {{
      type: "fragment",
      path: "exports/note.md",
      summary: "note"
    }}
  }}
end
"#
            ),
        )
        .expect("prayspec");
        fs::write(package_root.join("exports/note.md"), format!("{note}\n")).expect("export");
    }
}

#[test]
fn grouped_packages_keep_membership_in_canonical_manifest() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
group :development, :test do
  agent "sample/base", "*"
end
"#,
    )
    .expect("parse");
    assert_eq!(manifest.packages.len(), 1);
    assert_eq!(manifest.packages[0].groups, vec!["development", "test"]);
    let hash = manifest.manifest_hash().expect("hash");
    assert!(hash.starts_with("sha256:"));
}

#[test]
fn render_filters_packages_by_environment() {
    let root = unique_temp_dir("pray-env-render");
    write_grouped_fixture(&root);
    let options = ResolveOptions {
        environment: Some("development".to_string()),
        ..ResolveOptions::default()
    };
    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &options).expect("resolve");
    assert_eq!(project.packages.len(), 2);
    let rendered = render_project(&project).expect("render");
    let content = &rendered[0].content;
    assert!(content.contains("note for shared"));
    assert!(content.contains("note for dev-only"));
}

#[test]
fn omitted_environment_renders_only_ungrouped_packages() {
    let root = unique_temp_dir("pray-env-default");
    write_grouped_fixture(&root);
    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect("resolve");
    let rendered = render_project(&project).expect("render");
    let content = &rendered[0].content;
    assert!(content.contains("note for shared"));
    assert!(!content.contains("note for dev-only"));
}

#[test]
fn unknown_environment_fails_with_available_groups() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
group :development do
  agent "sample/base", "*"
end
"#,
    )
    .expect("parse");
    let error =
        validate_environment(&manifest, Some("production")).expect_err("unknown environment");
    assert!(error.to_string().contains("development"));
}

#[test]
fn package_matches_environment_supports_multiple_group_names() {
    assert!(package_matches_environment(
        &["development".to_string(), "test".to_string()],
        Some("test")
    ));
    assert!(!package_matches_environment(
        &["development".to_string()],
        None
    ));
}
