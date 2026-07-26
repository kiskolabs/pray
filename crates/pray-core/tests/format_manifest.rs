use pray_core::format_manifest::{
    classify_format_hints, format_recommended, recommend_manifest, uses_destination_dsl,
};
use pray_core::manifest::{parse_manifest, DestinationMode, ExportRole};
use pray_core::resolve::resolve_project_in_context;
use pray_core::resolve_context::ResolveOptions;
use std::collections::BTreeMap;
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

fn write_package(
    root: &Path,
    directory: &str,
    package_name: &str,
    export_name: &str,
    export_kind: &str,
    export_path: &str,
    body: &str,
    files: &[&str],
    default_path: Option<&str>,
) {
    let package_root = root.join(format!("packages/{directory}"));
    if let Some(parent) = Path::new(export_path).parent() {
        fs::create_dir_all(package_root.join(parent)).expect("export dirs");
    } else {
        fs::create_dir_all(&package_root).expect("package dir");
    }
    let files_literal = files
        .iter()
        .map(|file| format!("\"{file}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let default_path_literal = default_path
        .map(|path| format!(",\n      default_path: \"{path}\""))
        .unwrap_or_default();
    fs::write(
        package_root.join(format!("{directory}.prayspec")),
        format!(
            r#"
Package::Specification.new do |spec|
  spec.name = "{package_name}"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = [{files_literal}]
  spec.exports = {{
    "{export_name}" => {{
      type: "{export_kind}",
      path: "{export_path}",
      summary: "{export_name}"{default_path_literal}
    }}
  }}
end
"#
        ),
    )
    .expect("prayspec");
    fs::write(package_root.join(export_path), body).expect("export body");
}

#[test]
fn formats_legacy_prayfile_to_destination_dsl() {
    let root = unique_temp_dir("pray-format-legacy");
    fs::create_dir_all(root.join(".agents")).expect("agents");
    write_package(
        &root,
        "rules",
        "sample/rules",
        "rules",
        "fragment",
        "exports/rules.md",
        "Rules\n",
        &["exports/rules.md"],
        None,
    );
    fs::create_dir_all(root.join("packages/audit/skills/audit")).expect("skill dir");
    fs::write(
        root.join("packages/audit/skills/audit/SKILL.md"),
        "# Audit\n",
    )
    .expect("skill");
    fs::write(
        root.join("packages/audit/audit.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/audit"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["skills/audit/SKILL.md"]
  spec.exports = {
    "audit" => {
      type: "skill",
      path: "skills/audit",
      summary: "audit"
    }
  }
end
"#,
    )
    .expect("prayspec");
    write_package(
        &root,
        "security",
        "sample/security",
        "security",
        "file",
        "exports/SECURITY.md",
        "# Security\n",
        &["exports/SECURITY.md"],
        Some("SECURITY.md"),
    );
    fs::write(root.join(".agents/project.md"), "Local\n").expect("local");
    let original = r#"
prayfile "1"
target :tool_a do
  output "AGENTS.md"
  skills ".agents/skills"
end
agent "sample/rules", "~> 1.0", path: "packages/rules"
agent "sample/audit", "~> 1.0", path: "packages/audit"
agent "sample/security", "~> 1.0", path: "packages/security"
local ".agents/project.md", at: :start
"#;
    fs::write(root.join("Prayfile"), original).expect("prayfile");

    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect("resolve");
    let hints = classify_format_hints(&project);
    let manifest = parse_manifest(original).expect("parse");
    assert!(!uses_destination_dsl(&manifest));

    let formatted = format_recommended(&manifest, &hints).expect("format");
    assert!(formatted.contains("compose \"AGENTS.md\" do"));
    assert!(formatted.contains("pray \".agents/project.md\""));
    assert!(formatted.contains("pray \"sample/rules\""));
    assert!(formatted.contains("tree \".agents/skills\" do"));
    assert!(formatted.contains("pray \"sample/audit\""));
    assert!(formatted.contains("file: \"SECURITY.md\""));
    assert!(!formatted.contains("target :tool_a"));
    assert!(!formatted.contains("agent "));

    let reparsed = parse_manifest(&formatted).expect("reparse");
    assert!(uses_destination_dsl(&reparsed));
    assert_eq!(reparsed.targets[0].mode, DestinationMode::Compose);
    assert_eq!(reparsed.targets[1].mode, DestinationMode::Tree);
    let security = reparsed
        .packages
        .iter()
        .find(|package| package.name == "sample/security")
        .expect("security");
    assert_eq!(security.file.as_deref(), Some("SECURITY.md"));
    assert!(security.roles.contains(&ExportRole::File));

    let again = format_recommended(&reparsed, &hints).expect("format again");
    assert_eq!(again, formatted);
}

#[test]
fn omits_source_keyword_when_manifest_has_one_source() {
    let original = r#"
prayfile "1"
source "sample", path: "packages"
compose "AGENTS.md" do
  pray ".agents/project.md"
  pray "sample/rules", "~> 1.0", source: "sample"
end
"#;
    let manifest = parse_manifest(original).expect("parse");
    let formatted = format_recommended(&manifest, &BTreeMap::new()).expect("format");
    assert!(formatted.contains("source \"sample\""));
    assert!(formatted.contains("pray \"sample/rules\", \"~> 1.0\""));
    assert!(!formatted.contains("source: \"sample\""));
}

#[test]
fn omits_source_keyword_when_namespace_matches_source_handle() {
    let original = r#"
prayfile "1"
source "amkisko", path: "packages/amkisko"
source "other", path: "packages/other"
compose "AGENTS.md" do
  pray "amkisko/rules", "~> 1.0", source: "amkisko"
  pray "other/notes", "~> 1.0", source: "other"
end
"#;
    let manifest = parse_manifest(original).expect("parse");
    let formatted = format_recommended(&manifest, &BTreeMap::new()).expect("format");
    assert!(formatted.contains("pray \"amkisko/rules\", \"~> 1.0\""));
    assert!(formatted.contains("pray \"other/notes\", \"~> 1.0\""));
    assert!(!formatted.contains("source: \"amkisko\""));
    assert!(!formatted.contains("source: \"other\""));
}

#[test]
fn resolves_package_from_sole_source_without_source_keyword() {
    let root = unique_temp_dir("pray-sole-source");
    write_package(
        &root,
        "sample-rules",
        "sample/rules",
        "rules",
        "fragment",
        "exports/rules.md",
        "Rules\n",
        &["exports/rules.md"],
        None,
    );
    let original = r#"
prayfile "1"
source "sample", path: "packages"
compose "AGENTS.md" do
  pray "sample/rules", "~> 1.0"
end
"#;
    fs::write(root.join("Prayfile"), original).expect("prayfile");
    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect("resolve");
    assert_eq!(project.packages.len(), 1);
    assert_eq!(project.packages[0].declaration.name, "sample/rules");
}

#[test]
fn resolves_package_from_namespace_matching_source_handle() {
    let root = unique_temp_dir("pray-namespace-source");
    write_package(
        &root,
        "amkisko-rules",
        "amkisko/rules",
        "rules",
        "fragment",
        "exports/rules.md",
        "Rules\n",
        &["exports/rules.md"],
        None,
    );
    fs::create_dir_all(root.join("packages/other")).expect("other source root");
    let original = r#"
prayfile "1"
source "amkisko", path: "packages"
source "other", path: "packages/other"
compose "AGENTS.md" do
  pray "amkisko/rules", "~> 1.0"
end
"#;
    fs::write(root.join("Prayfile"), original).expect("prayfile");
    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect("resolve");
    assert_eq!(project.packages.len(), 1);
    assert_eq!(project.packages[0].declaration.name, "amkisko/rules");
}

#[test]
fn formats_legacy_prayfile_that_already_has_file_bindings() {
    let root = unique_temp_dir("pray-format-hybrid-file");
    fs::create_dir_all(root.join(".agents")).expect("agents");
    write_package(
        &root,
        "rules",
        "sample/rules",
        "rules",
        "fragment",
        "exports/rules.md",
        "Rules\n",
        &["exports/rules.md"],
        None,
    );
    fs::create_dir_all(root.join("packages/audit/skills/audit")).expect("skill dir");
    fs::write(
        root.join("packages/audit/skills/audit/SKILL.md"),
        "# Audit\n",
    )
    .expect("skill");
    fs::write(
        root.join("packages/audit/audit.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/audit"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["skills/audit/SKILL.md"]
  spec.exports = {
    "audit" => {
      type: "skill",
      path: "skills/audit",
      summary: "audit"
    }
  }
end
"#,
    )
    .expect("prayspec");
    write_package(
        &root,
        "security",
        "sample/security",
        "security",
        "file",
        "exports/SECURITY.md",
        "# Security\n",
        &["exports/SECURITY.md"],
        Some("SECURITY.md"),
    );
    fs::write(root.join(".agents/project.md"), "Local\n").expect("local");
    let original = r#"
prayfile "1"
target :tool_a do
  output "AGENTS.md"
  skills ".agents/skills"
end
agent "sample/rules", "~> 1.0", path: "packages/rules"
agent "sample/audit", "~> 1.0", path: "packages/audit"
pray "sample/security", "~> 1.0", path: "packages/security", file: "SECURITY.md"
local ".agents/project.md", at: :start
"#;
    fs::write(root.join("Prayfile"), original).expect("prayfile");

    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect("resolve");
    let hints = classify_format_hints(&project);
    let manifest = parse_manifest(original).expect("parse");
    assert!(uses_destination_dsl(&manifest));

    let formatted = format_recommended(&manifest, &hints).expect("format");
    assert!(formatted.contains("compose \"AGENTS.md\" do"));
    assert!(formatted.contains("tree \".agents/skills\" do"));
    assert!(formatted.contains("file: \"SECURITY.md\""));
    assert!(!formatted.contains("target :tool_a"));
}

#[test]
fn formats_existing_destination_dsl_idempotently() {
    let root = unique_temp_dir("pray-format-dsl");
    fs::create_dir_all(root.join(".agents")).expect("agents");
    write_package(
        &root,
        "rules",
        "sample/rules",
        "rules",
        "fragment",
        "exports/rules.md",
        "Rules\n",
        &["exports/rules.md"],
        None,
    );
    fs::write(root.join(".agents/project.md"), "Local\n").expect("local");
    let original = r#"
prayfile "1"
compose "AGENTS.md" do
  pray ".agents/project.md"
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
"#;
    fs::write(root.join("Prayfile"), original).expect("prayfile");
    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect("resolve");
    let hints = classify_format_hints(&project);
    let manifest = parse_manifest(original).expect("parse");
    let formatted = format_recommended(&manifest, &hints).expect("format");
    let again = format_recommended(&parse_manifest(&formatted).expect("parse"), &hints)
        .expect("format again");
    assert_eq!(formatted, again);
    assert!(formatted.contains("compose \"AGENTS.md\" do"));
}

#[test]
fn recommend_manifest_classifies_roles_from_hints() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
target :tool_a do
  output "AGENTS.md"
  skills ".agents/skills"
end
agent "sample/rules", "~> 1.0", path: "packages/rules"
agent "sample/audit", "~> 1.0", path: "packages/audit"
"#,
    )
    .expect("parse");
    let mut hints = BTreeMap::new();
    hints.insert(
        "sample/rules".to_string(),
        pray_core::format_manifest::PackageFormatHint {
            roles: vec![ExportRole::Fragment],
            file_path: None,
        },
    );
    hints.insert(
        "sample/audit".to_string(),
        pray_core::format_manifest::PackageFormatHint {
            roles: vec![ExportRole::Folder],
            file_path: None,
        },
    );
    let recommended = recommend_manifest(&manifest, &hints);
    assert_eq!(recommended.targets.len(), 2);
    assert!(recommended.targets[0]
        .entries
        .iter()
        .any(|entry| matches!(entry, pray_core::manifest::DestinationEntry::Package { name } if name == "sample/rules")));
    assert!(recommended.targets[1]
        .entries
        .iter()
        .any(|entry| matches!(entry, pray_core::manifest::DestinationEntry::Package { name } if name == "sample/audit")));
}
