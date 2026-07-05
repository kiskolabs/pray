#[path = "install_support.rs"]
mod support;

use std::fs;

use support::{create_derived_fixture, create_fixture, run_pray, temporary_directory};

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
    assert!(stderr.contains("pray install"));
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
    assert!(stderr.contains("pray install"));
}

#[test]
fn install_reports_missing_required_local_file_with_recovery_guidance() {
    let repo = temporary_directory("pray-install-missing-local");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    fs::remove_file(repo.join(".agents/project.md")).expect("remove local file");

    let install = run_pray(&repo, &["install"]);
    assert!(!install.status.success());
    assert_eq!(install.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&install.stderr);
    assert!(stderr.contains("Prayfile lists"));
    assert!(stderr.contains(".agents/project.md"));
    assert!(stderr.contains("Create the file"));
    assert!(stderr.contains("pray install"));
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
        repo.join(".agents/project.md"),
        "Local guidance\nExtra local guidance\n",
    )
    .expect("rewrite local file");

    let locked = run_pray(&repo, &["install", "--locked"]);
    assert!(!locked.status.success());
    assert_eq!(locked.status.code(), Some(6));
    let stderr = String::from_utf8_lossy(&locked.stderr);
    assert!(stderr.contains("lockfile needs update"));
    assert!(stderr.contains("pray install"));
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
        "Do not edit managed blocks in `INSTRUCTIONS.md` or skills under `.agents/`.",
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
fn install_offline_accepts_derived_package_paths_when_files_exist() {
    let repo = temporary_directory("pray-install-offline-derived");
    create_derived_fixture(&repo);

    let offline = run_pray(&repo, &["install", "--offline"]);
    assert!(
        offline.status.success(),
        "offline install failed: {}",
        String::from_utf8_lossy(&offline.stderr)
    );
    assert!(repo.join("INSTRUCTIONS.md").is_file());
}
