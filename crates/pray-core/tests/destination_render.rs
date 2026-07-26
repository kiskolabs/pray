use pray_core::lockfile::build_lockfile;
use pray_core::render::{planned_provisioned_files, render_project, write_rendered_targets};
use pray_core::resolve::resolve_project_in_context;
use pray_core::resolve_context::ResolveOptions;
use pray_core::verify::verify_project;
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

#[allow(clippy::too_many_arguments)]
fn write_package(
    root: &Path,
    directory: &str,
    package_name: &str,
    export_name: &str,
    export_kind: &str,
    export_path: &str,
    body: &str,
    files: &[&str],
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
      summary: "{export_name}"
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
fn legacy_shape_still_fans_out_fragments_and_skills() {
    let root = unique_temp_dir("pray-legacy-fanout");
    fs::create_dir_all(root.join(".agents")).expect("agents");
    write_package(
        &root,
        "rules",
        "sample/rules",
        "rules",
        "fragment",
        "exports/rules.md",
        "Legacy rules\n",
        &["exports/rules.md"],
    );
    fs::create_dir_all(root.join("packages/audit/skills/audit")).expect("skill dir");
    fs::write(
        root.join("packages/audit/skills/audit/SKILL.md"),
        "# Audit skill\n",
    )
    .expect("skill file");
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
    .expect("audit prayspec");
    fs::write(root.join(".agents/project.md"), "Local note\n").expect("local");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
  skills ".agents/skills"
end
agent "sample/rules", "~> 1.0", path: "packages/rules"
agent "sample/audit", "~> 1.0", path: "packages/audit"
local ".agents/project.md"
"#,
    )
    .expect("prayfile");

    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect("resolve");
    let rendered = render_project(&project).expect("render");
    assert_eq!(rendered.len(), 1);
    assert!(rendered[0].content.contains("Legacy rules"));
    assert!(rendered[0].content.contains("Local note"));
    assert!(rendered[0].content.contains("## Shared instructions"));

    let planned = planned_provisioned_files(&project).expect("planned");
    assert!(planned
        .iter()
        .any(|file| file.path.ends_with(".agents/skills/audit/SKILL.md")));
}

#[test]
fn compose_tree_and_file_bindings_are_isolated() {
    let root = unique_temp_dir("pray-destination-dsl");
    fs::create_dir_all(root.join(".agents")).expect("agents");
    write_package(
        &root,
        "rules",
        "sample/rules",
        "rules",
        "fragment",
        "exports/rules.md",
        "Compose rules\n",
        &["exports/rules.md"],
    );
    write_package(
        &root,
        "unbound",
        "sample/unbound",
        "unbound",
        "fragment",
        "exports/unbound.md",
        "Should not appear\n",
        &["exports/unbound.md"],
    );
    fs::create_dir_all(root.join("packages/audit/skills/audit")).expect("skill dir");
    fs::write(
        root.join("packages/audit/skills/audit/SKILL.md"),
        "# Audit skill\n",
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
    fs::create_dir_all(root.join("packages/security/exports")).expect("security dirs");
    fs::write(
        root.join("packages/security/exports/SECURITY.md"),
        "# Security Policy\n",
    )
    .expect("security body");
    fs::write(
        root.join("packages/security/security.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/security"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["exports/SECURITY.md"]
  spec.exports = {
    "security" => {
      type: "file",
      path: "exports/SECURITY.md",
      default_path: "SECURITY.md"
    }
  }
end
"#,
    )
    .expect("security prayspec");
    fs::write(root.join(".agents/project.md"), "Local first\n").expect("local");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
compose "AGENTS.md" do
  pray ".agents/project.md"
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
tree ".agents/skills" do
  pray "sample/audit", "~> 1.0", path: "packages/audit"
end
pray "sample/security", "~> 1.0", path: "packages/security", file: "SECURITY.md"
agent "sample/unbound", "~> 1.0", path: "packages/unbound"
"#,
    )
    .expect("prayfile");

    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect("resolve");
    let rendered = render_project(&project).expect("render");
    assert_eq!(rendered.len(), 1);
    let content = &rendered[0].content;
    assert!(content.contains("Local first"));
    assert!(content.contains("Compose rules"));
    assert!(!content.contains("Should not appear"));
    assert!(!content.contains("## Shared instructions"));

    let planned = planned_provisioned_files(&project).expect("planned");
    assert!(planned.iter().any(|file| file.path == *"SECURITY.md"));
    assert!(planned
        .iter()
        .any(|file| file.path.ends_with(".agents/skills/audit/SKILL.md")));
    assert!(!planned
        .iter()
        .any(|file| { file.path.to_string_lossy().contains("security/SECURITY.md") }));

    write_rendered_targets(&project, &rendered).expect("write");
    let security = fs::read_to_string(root.join("SECURITY.md")).expect("security written");
    assert_eq!(security, "# Security Policy\n");

    let lockfile = build_lockfile(
        project.manifest_hash.clone(),
        None,
        &root,
        &project.manifest.sources,
        &project.manifest.targets,
        &rendered,
        &project.packages,
        &BTreeMap::new(),
        &BTreeMap::new(),
    );
    let report = verify_project(&project, &lockfile, true).expect("verify");
    assert!(report.is_clean());
}

#[test]
fn folder_export_only_filter_limits_provisioned_tree() {
    let root = unique_temp_dir("pray-folder-only");
    fs::create_dir_all(root.join("packages/templates/templates")).expect("templates");
    fs::write(
        root.join("packages/templates/templates/issue.md"),
        "issue\n",
    )
    .expect("issue");
    fs::write(root.join("packages/templates/templates/pr.md"), "pr\n").expect("pr");
    fs::write(
        root.join("packages/templates/templates/draft.md"),
        "draft\n",
    )
    .expect("draft");
    fs::write(
        root.join("packages/templates/templates.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/templates"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["templates/issue.md", "templates/pr.md", "templates/draft.md"]
  spec.exports = {
    "templates" => {
      type: "folder",
      path: "templates",
      only: ["issue.md", "pr.md"]
    }
  }
end
"#,
    )
    .expect("prayspec");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
tree ".agents/templates" do
  pray "sample/templates", "~> 1.0", path: "packages/templates"
end
"#,
    )
    .expect("prayfile");

    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect("resolve");
    let planned = planned_provisioned_files(&project).expect("planned");
    let paths: Vec<_> = planned
        .iter()
        .map(|file| file.path.to_string_lossy().to_string())
        .collect();
    assert!(paths.iter().any(|path| path.ends_with("issue.md")));
    assert!(paths.iter().any(|path| path.ends_with("pr.md")));
    assert!(!paths.iter().any(|path| path.ends_with("draft.md")));
}

#[test]
fn export_singular_and_aliases_parse_for_resolution() {
    let root = unique_temp_dir("pray-export-alias");
    write_package(
        &root,
        "rules",
        "sample/rules",
        "rules",
        "fragment",
        "exports/rules.md",
        "Alias rules\n",
        &["exports/rules.md"],
    );
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
compose "AGENTS.md" do
  include "sample/rules", "~> 1.0", path: "packages/rules", export: "rules"
end
"#,
    )
    .expect("prayfile");
    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect("resolve");
    assert_eq!(
        project.packages[0].selected_exports,
        vec!["rules".to_string()]
    );
    let rendered = render_project(&project).expect("render");
    assert!(rendered[0].content.contains("Alias rules"));
}
