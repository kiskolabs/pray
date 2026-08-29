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
        repo.join(".agents/project.md"),
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
    assert!(stderr.contains("position drift"));
    assert!(stderr.contains("first marker"));
    assert!(stderr.contains("cause:"));
}

#[test]
fn install_groups_position_drift_with_local_cause() {
    let repo = temporary_directory("pray-install-position-cause");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered.replace("Local guidance\n", "Local guidance\nExtra unmarked line\n");
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let install = run_pray(&repo, &["install"]);
    assert!(install.status.success());
    let stderr = String::from_utf8_lossy(&install.stderr);
    let stdout = String::from_utf8_lossy(&install.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(combined.contains("position drift"));
    assert!(combined.contains("cause:"));
    assert!(combined.contains("INSTRUCTIONS.md:"));
    assert!(
        combined.contains(".agents/project.md:") || combined.contains("fresh composition"),
        "expected local or fresh cause, got:\n{combined}"
    );
    let conflict_count = combined.matches("Conflict:").count();
    assert_eq!(
        conflict_count, 1,
        "expected one grouped conflict, got:\n{combined}"
    );
}

#[test]
fn install_records_patched_marker_positions_so_verify_passes() {
    let repo = temporary_directory("pray-install-lock-positions");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered.replace("Local guidance\n", "Local guidance\nExtra unmarked line\n");
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed:\n{}",
        String::from_utf8_lossy(&install.stderr)
    );

    let verify = run_pray(&repo, &["verify"]);
    assert!(
        verify.status.success(),
        "verify failed:\n{}",
        String::from_utf8_lossy(&verify.stderr)
    );
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
