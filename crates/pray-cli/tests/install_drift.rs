#[path = "install_support.rs"]
mod support;

use std::fs;

use support::{create_fixture, run_pray, temporary_directory};

#[test]
fn drift_reports_renderer_changes_in_sections() {
    let repo = temporary_directory("pray-drift-renderer");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    fs::write(
        repo.join("agent/local/project.md"),
        "Local guidance\nChanged local guidance\n",
    )
    .expect("rewrite local file");

    let drift = run_pray(&repo, &["drift"]);
    assert!(!drift.status.success());
    assert_eq!(drift.status.code(), Some(6));
    let stderr = String::from_utf8_lossy(&drift.stderr);
    assert!(stderr.contains("Rendered file changes"));
    assert!(stderr.contains("renderer_drift"));
}

#[test]
fn drift_reports_position_changes_in_sections() {
    let repo = temporary_directory("pray-drift-position");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered.replace(
        "## Shared instructions\n\n<!-- pray:",
        "## Shared instructions\n\n\n<!-- pray:",
    );
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let drift = run_pray(&repo, &["drift"]);
    assert!(!drift.status.success());
    assert_eq!(drift.status.code(), Some(6));
    let stderr = String::from_utf8_lossy(&drift.stderr);
    assert!(stderr.contains("Managed span changes"));
    assert!(stderr.contains("position_drift"));
}

#[test]
fn drift_semantic_summarizes_package_version_changes() {
    let repo = temporary_directory("pray-drift-semantic");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    fs::write(
        repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.4"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md", "exports/security-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    },
    "security-basics" => {
      type: "fragment",
      path: "exports/security-basics.md",
      summary: "Security guidance"
    }
  }
end
"#,
    )
    .expect("rewrite prayspec");
    fs::write(
        repo.join("packages/base/exports/security-basics.md"),
        "Security guidance\n",
    )
    .expect("write second export");

    let semantic = run_pray(&repo, &["drift", "--semantic"]);
    assert!(!semantic.status.success());
    assert_eq!(semantic.status.code(), Some(6));
    let stderr = String::from_utf8_lossy(&semantic.stderr);
    assert!(stderr.contains("Semantic diff"));
    assert!(stderr.contains("sample/base 1.4.3 -> 1.4.4 would change 2 managed spans"));
}
