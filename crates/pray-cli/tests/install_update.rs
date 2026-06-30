#[path = "install_support.rs"]
mod support;

use std::fs;

use support::{create_add_fixture, create_tree_fixture, run_pray, temporary_directory};

#[test]
fn update_rejects_unknown_package_selection() {
    let repo = temporary_directory("pray-update-unknown");
    create_add_fixture(&repo);

    let update = run_pray(&repo, &["update", "missing/base"]);
    assert!(!update.status.success());
    assert_eq!(update.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&update.stderr);
    assert!(stderr.contains("package missing/base not found"));
}

#[test]
fn update_refreshes_only_the_selected_package_version() {
    let repo = temporary_directory("pray-update-selected");
    create_tree_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    fs::write(
        repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.4"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type = "fragment"
      path = "exports/testing-basics.md"
      summary = "Testing guidance"
    }
  }
  spec.add_dependency "sample/common", "~> 1.0"
end
"#,
    )
    .expect("rewrite base prayspec");
    fs::write(
        repo.join("packages/common/sample-common.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/common"
  spec.version = "1.1.0"
  spec.summary = "common guidance"
  spec.files = ["README.md", "exports/common-basics.md"]
  spec.exports = {
    "common-basics" => {
      type = "fragment"
      path = "exports/common-basics.md"
      summary = "Common guidance"
    }
  }
end
"#,
    )
    .expect("rewrite common prayspec");

    let update = run_pray(&repo, &["update", "sample/base"]);
    assert!(
        update.status.success(),
        "update failed: {}",
        String::from_utf8_lossy(&update.stderr)
    );
    let stdout = String::from_utf8_lossy(&update.stdout);
    assert!(stdout.contains("sample/base 1.4.3 -> 1.4.4"));
    assert!(!stdout.contains("dependent packages affected"));

    let lockfile = fs::read_to_string(repo.join("Prayfile.lock")).expect("lockfile exists");
    assert!(lockfile.contains("sample/base"));
    assert!(lockfile.contains("1.4.4"));
    assert!(lockfile.contains("sample/common"));
    assert!(lockfile.contains("1.0.0"));
    assert!(!lockfile.contains("1.1.0"));
}

#[test]
fn update_reports_dependent_packages_affected() {
    let repo = temporary_directory("pray-update-dependent");
    create_tree_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    fs::write(
        repo.join("packages/common/sample-common.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/common"
  spec.version = "1.1.0"
  spec.summary = "common guidance"
  spec.files = ["README.md", "exports/common-basics.md"]
  spec.exports = {
    "common-basics" => {
      type = "fragment"
      path = "exports/common-basics.md"
      summary = "Common guidance"
    }
  }
end
"#,
    )
    .expect("rewrite common prayspec");

    let update = run_pray(&repo, &["update", "sample/common"]);
    assert!(
        update.status.success(),
        "update failed: {}",
        String::from_utf8_lossy(&update.stderr)
    );
    let stdout = String::from_utf8_lossy(&update.stdout);
    assert!(stdout.contains("sample/common 1.0.0 -> 1.1.0"));
    assert!(stdout.contains("dependent packages affected: sample/base"));
    assert!(stdout.contains("\"updated_packages\""));
    assert!(stdout.contains("\"dependent_packages_affected\""));

    let lockfile = fs::read_to_string(repo.join("Prayfile.lock")).expect("lockfile exists");
    assert!(lockfile.contains("sample/common"));
    assert!(lockfile.contains("1.1.0"));
    assert!(lockfile.contains("sample/base"));
    assert!(lockfile.contains("1.4.3"));
}
