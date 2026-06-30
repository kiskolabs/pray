#[path = "install_support.rs"]
mod support;

use pray_core::lockfile::read_lockfile;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use support::{
    create_add_fixture, create_derived_fixture, create_fixture, create_tree_fixture,
    read_package_archive, run_pray, temporary_directory,
};

#[test]
fn installs_renders_and_verifies_a_local_package() {
    let repo = temporary_directory("pray-install");
    create_fixture(&repo);

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let rendered = fs::read_to_string(repo.join("INSTRUCTIONS.md")).expect("rendered file exists");
    assert!(rendered.contains("<!-- pray:"));

    let lockfile = repo.join("Prayfile.lock");
    let initial_modified = fs::metadata(&lockfile)
        .expect("lockfile exists")
        .modified()
        .expect("lockfile modified time");
    sleep(Duration::from_secs(1));

    let reinstall = run_pray(&repo, &["install"]);
    assert!(
        reinstall.status.success(),
        "reinstall failed: {}",
        String::from_utf8_lossy(&reinstall.stderr)
    );

    let next_modified = fs::metadata(&lockfile)
        .expect("lockfile exists")
        .modified()
        .expect("lockfile modified time");
    assert_eq!(initial_modified, next_modified);

    let verify = run_pray(&repo, &["verify"]);
    assert!(
        verify.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
}

#[test]
fn plan_reports_changes_without_writing_files() {
    let repo = temporary_directory("pray-plan");
    create_fixture(&repo);

    let plan = run_pray(&repo, &["plan"]);
    assert!(
        plan.status.success(),
        "plan failed: {}",
        String::from_utf8_lossy(&plan.stderr)
    );
    assert!(!repo.join("Prayfile.lock").exists());
    assert!(!repo.join("INSTRUCTIONS.md").exists());
    let stdout = String::from_utf8_lossy(&plan.stdout);
    assert!(stdout.contains("Prayfile.lock"));
    assert!(stdout.contains("INSTRUCTIONS.md"));
}

#[test]
fn apply_materializes_like_install() {
    let repo = temporary_directory("pray-apply");
    create_fixture(&repo);

    let apply = run_pray(&repo, &["apply"]);
    assert!(
        apply.status.success(),
        "apply failed: {}",
        String::from_utf8_lossy(&apply.stderr)
    );
    assert!(repo.join("Prayfile.lock").exists());
    assert!(repo.join("INSTRUCTIONS.md").exists());

    let verify = run_pray(&repo, &["verify"]);
    assert!(
        verify.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
}

#[test]
fn add_remove_and_update_package_declaration() {
    let repo = temporary_directory("pray-add-remove-update");
    create_add_fixture(&repo);

    let add = run_pray(&repo, &["add", "sample/base", "--path", "packages/base"]);
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let manifest = fs::read_to_string(repo.join("Prayfile")).expect("manifest exists");
    assert!(manifest.contains("agent \"sample/base\", path: \"packages/base\""));

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
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
end
"#,
    )
    .expect("rewrite prayspec");

    let update = run_pray(&repo, &["update", "sample/base"]);
    assert!(
        update.status.success(),
        "update failed: {}",
        String::from_utf8_lossy(&update.stderr)
    );
    let stdout = String::from_utf8_lossy(&update.stdout);
    assert!(stdout.contains("Update summary"));
    assert!(stdout.contains("sample/base 1.4.3 -> 1.4.4"));
    assert!(stdout.contains("source: path:packages/base"));
    assert!(stdout.contains("exports affected: testing-basics"));
    assert!(stdout.contains("targets affected: tool_a"));
    assert!(stdout.contains("rendered files affected: INSTRUCTIONS.md"));
    assert!(stdout.contains("warnings: none"));
    let lockfile = fs::read_to_string(repo.join("Prayfile.lock")).expect("lockfile exists");
    assert!(lockfile.contains("1.4.4"));

    let remove = run_pray(&repo, &["remove", "sample/base"]);
    assert!(
        remove.status.success(),
        "remove failed: {}",
        String::from_utf8_lossy(&remove.stderr)
    );
    let manifest = fs::read_to_string(repo.join("Prayfile")).expect("manifest exists");
    assert!(!manifest.contains("sample/base"));
    let lockfile = fs::read_to_string(repo.join("Prayfile.lock")).expect("lockfile exists");
    assert!(!lockfile.contains("sample/base"));
    let rendered = fs::read_to_string(repo.join("INSTRUCTIONS.md")).expect("rendered file exists");
    assert!(!rendered.contains("Testing guidance"));
}

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

#[test]
fn package_builds_a_tar_zst_archive_from_package_contents() {
    let repo = temporary_directory("pray-package");
    create_add_fixture(&repo);

    let add = run_pray(&repo, &["add", "sample/base", "--path", "packages/base"]);
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let package = run_pray(&repo, &["package"]);
    assert!(
        package.status.success(),
        "package failed: {}",
        String::from_utf8_lossy(&package.stderr)
    );

    let archive = repo.join("sample-base-1.4.3.praypkg");
    assert!(archive.is_file());

    let entries = read_package_archive(&archive);
    let metadata = entries.get("metadata.json").expect("metadata");
    assert!(metadata.contains("\"name\": \"sample/base\""));
    assert!(metadata.contains("\"version\": \"1.4.3\""));
    assert!(metadata.contains("\"files\": ["));
    assert!(metadata.contains("README.md"));
    assert!(metadata.contains("exports/testing-basics.md"));
    assert_eq!(
        entries.get("README.md").expect("archive readme"),
        "package readme\n"
    );
    assert_eq!(
        entries
            .get("exports/testing-basics.md")
            .expect("archive export"),
        "Testing guidance\n"
    );
}

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
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
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
      type: "fragment",
      path: "exports/common-basics.md",
      summary: "Common guidance"
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
      type: "fragment",
      path: "exports/common-basics.md",
      summary: "Common guidance"
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

#[test]
fn vendor_copies_package_contents_into_pray_vendor() {
    let repo = temporary_directory("pray-vendor");
    create_add_fixture(&repo);

    let add = run_pray(&repo, &["add", "sample/base", "--path", "packages/base"]);
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let vendor = run_pray(&repo, &["vendor"]);
    assert!(
        vendor.status.success(),
        "vendor failed: {}",
        String::from_utf8_lossy(&vendor.stderr)
    );

    let vendored = repo.join(".pray/vendor/sample-base/1.4.3");
    assert!(vendored.is_dir());
    assert!(vendored.join("metadata.json").exists());
    assert_eq!(
        fs::read_to_string(vendored.join("README.md")).expect("vendored readme"),
        "package readme\n"
    );
    assert_eq!(
        fs::read_to_string(vendored.join("exports/testing-basics.md")).expect("vendored export"),
        "Testing guidance\n"
    );
}

#[test]
fn verify_reports_custom_implementation() {
    let repo = temporary_directory("pray-verify");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let mut rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    rendered = rendered.replace("Testing guidance", "Changed guidance");
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let verify = run_pray(&repo, &["verify"]);
    assert!(!verify.status.success());
    assert_eq!(verify.status.code(), Some(6));
    let stderr = String::from_utf8_lossy(&verify.stderr);
    assert!(stderr.contains("custom_implementation") || stderr.contains("verify error"));
    assert!(stderr.contains("sample/base::testing-basics"));
    assert!(stderr.contains("rerun pray install"));
}

#[test]
fn verify_warns_on_orphan_markers_and_strict_fails() {
    let repo = temporary_directory("pray-verify-orphan-marker");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let mut rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    rendered.push_str("<!-- pray:abc123 -->\nOrphan marker body\n<!-- pray:abc123 -->\n");
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let verify = run_pray(&repo, &["verify"]);
    assert!(verify.status.success());
    let stderr = String::from_utf8_lossy(&verify.stderr);
    assert!(stderr.contains("orphan_marker"));

    let strict_verify = run_pray(&repo, &["verify", "--strict"]);
    assert!(!strict_verify.status.success());
    assert_eq!(strict_verify.status.code(), Some(6));
    let strict_stderr = String::from_utf8_lossy(&strict_verify.stderr);
    assert!(strict_stderr.contains("orphan_marker"));
}

#[test]
#[ignore = "managed patching is not implemented yet"]
fn install_preserves_unmanaged_content_when_patching_rendered_files() {
    let repo = temporary_directory("pray-install-patch-preserve");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered.replace(
        "## Shared instructions\n\n",
        "## Shared instructions\n\nUser note: keep this line.\n\n",
    );
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    assert!(rendered.contains("User note: keep this line."));
    assert!(rendered.contains("Testing guidance"));
}

#[test]
#[ignore = "conflict detection is not implemented yet"]
fn install_rejects_conflicting_managed_changes_when_conflict_policy_is_fail() {
    let repo = temporary_directory("pray-install-conflict");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered.replace("Testing guidance", "Conflicting guidance");
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let install = run_pray(&repo, &["install"]);
    assert!(!install.status.success());
    assert!(String::from_utf8_lossy(&install.stderr).contains("conflict"));
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    assert!(rendered.contains("Conflicting guidance"));
}

#[test]
fn verify_reports_missing_managed_span_with_package_context_and_recovery_guidance() {
    let repo = temporary_directory("pray-verify-missing-span");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered
        .replace("<!-- pray:", "<!-- removed pray:")
        .replace(" -->", " -->");
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let verify = run_pray(&repo, &["verify"]);
    assert!(!verify.status.success());
    assert_eq!(verify.status.code(), Some(6));
    let stderr = String::from_utf8_lossy(&verify.stderr);
    assert!(stderr.contains("removed_prayer"));
    assert!(stderr.contains("INSTRUCTIONS.md"));
    assert!(stderr.contains("sample/base::testing-basics"));
    assert!(stderr.contains("rerun pray install"));
}

#[test]
fn install_reports_missing_required_local_file_with_recovery_guidance() {
    let repo = temporary_directory("pray-install-missing-local");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    fs::remove_file(repo.join("agent/local/project.md")).expect("remove local file");

    let install = run_pray(&repo, &["install"]);
    assert!(!install.status.success());
    assert_eq!(install.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&install.stderr);
    assert!(stderr.contains("missing local file"));
    assert!(stderr.contains("agent/local/project.md"));
    assert!(stderr.contains("restore the file"));
    assert!(stderr.contains("rerun pray install"));
}

#[test]
fn beta_flow_rejects_corrupted_lockfile_after_clean_install() {
    let repo = temporary_directory("pray-beta-lockfile");
    create_fixture(&repo);

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let verify = run_pray(&repo, &["verify"]);
    assert!(
        verify.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&verify.stderr)
    );

    let drift = run_pray(&repo, &["drift"]);
    assert!(
        drift.status.success(),
        "drift failed: {}",
        String::from_utf8_lossy(&drift.stderr)
    );

    let format = run_pray(&repo, &["format"]);
    assert!(
        format.status.success(),
        "format failed: {}",
        String::from_utf8_lossy(&format.stderr)
    );

    fs::write(repo.join("Prayfile.lock"), "this is not a valid lockfile\n")
        .expect("corrupt lockfile");

    let corrupted_verify = run_pray(&repo, &["verify"]);
    assert!(!corrupted_verify.status.success());
    assert_eq!(corrupted_verify.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&corrupted_verify.stderr);
    assert!(stderr.contains("lockfile parse error") || stderr.contains("parse error"));
}

#[test]
fn install_repairs_corrupted_rendered_output_and_lockfile() {
    let repo = temporary_directory("pray-install-repair");
    create_fixture(&repo);

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let original_lockfile = fs::read_to_string(repo.join("Prayfile.lock")).expect("lockfile");
    let original_rendered = fs::read_to_string(repo.join("INSTRUCTIONS.md")).expect("rendered");

    fs::write(repo.join("Prayfile.lock"), "this is not a valid lockfile\n")
        .expect("corrupt lockfile");
    fs::write(repo.join("INSTRUCTIONS.md"), "broken rendered output\n").expect("corrupt rendered");

    let reinstall = run_pray(&repo, &["install"]);
    assert!(
        reinstall.status.success(),
        "reinstall failed: {}",
        String::from_utf8_lossy(&reinstall.stderr)
    );

    assert_eq!(
        fs::read_to_string(repo.join("Prayfile.lock")).expect("restored lockfile"),
        original_lockfile
    );
    assert_eq!(
        fs::read_to_string(repo.join("INSTRUCTIONS.md")).expect("restored rendered"),
        original_rendered
    );

    let verify = run_pray(&repo, &["verify"]);
    assert!(
        verify.status.success(),
        "verify failed after repair: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
}

#[test]
fn install_locked_rejects_lockfile_drift() {
    let repo = temporary_directory("pray-install-locked");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let original_lockfile = fs::read_to_string(repo.join("Prayfile.lock")).expect("lockfile");
    let original_rendered = fs::read_to_string(repo.join("INSTRUCTIONS.md")).expect("rendered");

    fs::write(
        repo.join("agent/local/project.md"),
        "Local guidance\nExtra local guidance\n",
    )
    .expect("rewrite local file");

    let locked = run_pray(&repo, &["install", "--locked"]);
    assert!(!locked.status.success());
    assert_eq!(locked.status.code(), Some(6));
    let stderr = String::from_utf8_lossy(&locked.stderr);
    assert!(stderr.contains("lockfile needs update"));
    assert!(stderr.contains("rerun pray install"));
    assert_eq!(
        fs::read_to_string(repo.join("Prayfile.lock")).expect("preserved lockfile"),
        original_lockfile
    );
    assert_eq!(
        fs::read_to_string(repo.join("INSTRUCTIONS.md")).expect("preserved rendered"),
        original_rendered
    );
}

#[test]
fn install_frozen_rejects_stale_rendered_output() {
    let repo = temporary_directory("pray-install-frozen");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered.replace(
        "Do not edit managed blocks or managed skills.",
        "Managed blocks stay read-only.",
    );
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let frozen = run_pray(&repo, &["install", "--frozen"]);
    assert!(!frozen.status.success());
    assert_eq!(frozen.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&frozen.stderr);
    assert!(stderr.contains("stale"));
    assert!(stderr.contains("pray plan"));
}

#[test]
fn install_offline_accepts_explicit_local_paths() {
    let repo = temporary_directory("pray-install-offline");
    create_fixture(&repo);

    let offline = run_pray(&repo, &["install", "--offline"]);
    assert!(
        offline.status.success(),
        "offline install failed: {}",
        String::from_utf8_lossy(&offline.stderr)
    );
}

#[test]
fn install_offline_rejects_derived_package_paths() {
    let repo = temporary_directory("pray-install-offline-derived");
    create_derived_fixture(&repo);

    let offline = run_pray(&repo, &["install", "--offline"]);
    assert!(!offline.status.success());
    assert_eq!(offline.status.code(), Some(8));
    let stderr = String::from_utf8_lossy(&offline.stderr);
    assert!(stderr.contains("offline mode") || stderr.contains("unsupported feature"));
}

