#[path = "install_support.rs"]
mod support;

use std::fs;
use std::path::PathBuf;

use support::{
    create_add_fixture, create_fixture, create_tree_fixture, run_pray, temporary_directory,
};

#[test]
fn list_reports_the_resolved_package_set() {
    let repo = temporary_directory("pray-list");
    create_fixture(&repo);

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let list = run_pray(&repo, &["list"]);
    assert!(
        list.status.success(),
        "list failed: {}",
        String::from_utf8_lossy(&list.stderr)
    );

    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("Package list"));
    assert!(stdout.contains("sample/base 1.4.3"));
    assert!(stdout.contains("source=path:packages/base"));
    assert!(stdout.contains("exports="));
    assert!(stdout.contains("testing-basics"));
    assert!(stdout.contains("security-basics"));
}

#[test]
fn outdated_reports_when_the_resolved_version_changes() {
    let repo = temporary_directory("pray-outdated");
    create_fixture(&repo);

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

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

    let outdated = run_pray(&repo, &["outdated"]);
    assert!(
        outdated.status.success(),
        "outdated failed: {}",
        String::from_utf8_lossy(&outdated.stderr)
    );

    let stdout = String::from_utf8_lossy(&outdated.stdout);
    assert!(stdout.contains("Outdated packages"));
    assert!(stdout.contains("sample/base 1.4.3 -> 1.4.4"));
}

#[test]
fn explain_reports_package_details_and_lockfile_context() {
    let repo = temporary_directory("pray-explain");
    create_fixture(&repo);

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let explain = run_pray(&repo, &["explain", "sample/base"]);
    assert!(
        explain.status.success(),
        "explain failed: {}",
        String::from_utf8_lossy(&explain.stderr)
    );

    let stdout = String::from_utf8_lossy(&explain.stdout);
    assert!(stdout.contains("Package explanation"));
    assert!(stdout.contains("name: sample/base"));
    assert!(stdout.contains("constraint: ~> 1.4"));
    assert!(stdout.contains("resolved version: 1.4.3"));
    assert!(stdout.contains("source: path:packages/base"));
    assert!(stdout.contains("exports:"));
    assert!(stdout.contains("testing-basics"));
    assert!(stdout.contains("security-basics"));
    assert!(stdout.contains("dependencies: none"));
    assert!(stdout.contains("lockfile version: 1.4.3"));
}

#[test]
fn tree_reports_dependency_graph() {
    let repo = temporary_directory("pray-tree");
    create_tree_fixture(&repo);

    let tree = run_pray(&repo, &["tree"]);
    assert!(
        tree.status.success(),
        "tree failed: {}",
        String::from_utf8_lossy(&tree.stderr)
    );

    let stdout = String::from_utf8_lossy(&tree.stdout);
    assert!(stdout.contains("Dependency tree"));
    assert!(stdout.contains("sample/base 1.4.3"));
    assert!(stdout.contains("sample/common 1.0.0"));
    assert!(stdout.contains("  sample/common 1.0.0"));
}

#[test]
fn clean_removes_local_ephemeral_state() {
    let repo = temporary_directory("pray-clean");
    create_add_fixture(&repo);
    fs::create_dir_all(repo.join(".pray/cache")).expect("cache directory");
    fs::create_dir_all(repo.join(".pray/vendor")).expect("vendor directory");
    fs::write(repo.join(".pray/state.json"), "{}\n").expect("state file");
    fs::write(repo.join(".pray/cache/item.bin"), "cached\n").expect("cache file");
    fs::write(repo.join(".pray/vendor/item.bin"), "vendored\n").expect("vendor file");

    let clean = run_pray(&repo, &["clean"]);
    assert!(
        clean.status.success(),
        "clean failed: {}",
        String::from_utf8_lossy(&clean.stderr)
    );
    assert!(!repo.join(".pray/cache").exists());
    assert!(!repo.join(".pray/vendor").exists());
    assert!(!repo.join(".pray/state.json").exists());
}

#[test]
fn format_normalizes_pray_markers_and_line_endings() {
    let repo = temporary_directory("pray-format");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered
        .replace("<!-- pray:", "<!--  pray:")
        .replace(" -->", "   -->")
        .replace("\n", "\r\n");
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let format = run_pray(&repo, &["format"]);
    assert!(
        format.status.success(),
        "format failed: {}",
        String::from_utf8_lossy(&format.stderr)
    );

    let formatted = fs::read_to_string(&rendered_path).expect("formatted file exists");
    assert!(!formatted.contains("\r"));
    assert!(formatted.contains("<!-- pray:"));
    assert!(formatted.contains("<!-- pray:0 ignore-comments -->"));
    assert!(!formatted.contains("<!--  pray:"));
    assert!(!formatted.contains("   -->"));

    let verify = run_pray(&repo, &["verify"]);
    assert!(
        verify.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
}
