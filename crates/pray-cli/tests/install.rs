#[path = "install_support.rs"]
mod support;

use std::fs;
use std::thread::sleep;
use std::time::Duration;

use support::{
    create_add_fixture, create_fixture, create_prayer_install_fixture, run_pray,
    temporary_directory,
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
    assert!(rendered.contains("### .agents/project.md"));
    assert!(!rendered.contains("/Users/"));

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
fn materializes_package_skill_directories_into_target_skills_path() {
    let repo = temporary_directory("pray-install-skill-tree");
    fs::create_dir_all(repo.join("packages/audit-skill/skills/audit")).expect("skill tree");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :agents do
  output "INSTRUCTIONS.md"
  folder ".agents/skills"
end
agent "sample/audit-skill", path: "packages/audit-skill", exports: ["audit"]
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");
    fs::write(
        repo.join("packages/audit-skill/audit-skill.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/audit-skill"
  spec.version = "1.0.0"
  spec.summary = "Audit skill package"
  spec.files = [
    "skills/audit/SKILL.md",
    "skills/audit/details.md"
  ]
  spec.exports = {
    "audit" => {
      type: "folder",
      path: "skills/audit",
      summary: "Audit skill"
    }
  }
end
"#,
    )
    .expect("write prayspec");
    fs::write(
        repo.join("packages/audit-skill/skills/audit/SKILL.md"),
        "# Audit skill\n",
    )
    .expect("write skill");
    fs::write(
        repo.join("packages/audit-skill/skills/audit/details.md"),
        "# Details\n",
    )
    .expect("write details");

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    assert!(repo.join(".agents/skills/audit/SKILL.md").is_file());
    assert!(repo.join(".agents/skills/audit/details.md").is_file());

    let rendered = fs::read_to_string(repo.join("INSTRUCTIONS.md")).expect("rendered file");
    assert!(
        !rendered.contains("# Audit skill"),
        "skill exports must not be inlined into root target files when skills path is configured"
    );
}

#[test]
fn materializes_file_exports_into_target_folder() {
    let repo = temporary_directory("pray-install-file-export");
    fs::create_dir_all(repo.join("packages/sample-rule/rules")).expect("rule directory");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :agents do
  output "INSTRUCTIONS.md"
  folder ".agents/rules"
end
agent "sample/rule", path: "packages/sample-rule", exports: ["review-checklist"]
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");
    fs::write(
        repo.join("packages/sample-rule/sample-rule.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/rule"
  spec.version = "1.0.0"
  spec.summary = "Review checklist"
  spec.files = ["rules/checklist.md"]
  spec.exports = {
    "review-checklist" => {
      type: "file",
      path: "rules/checklist.md",
      summary: "Review checklist"
    }
  }
end
"#,
    )
    .expect("write prayspec");
    fs::write(
        repo.join("packages/sample-rule/rules/checklist.md"),
        "# Review checklist\n",
    )
    .expect("write file export");

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    assert!(repo
        .join(".agents/rules/review-checklist/checklist.md")
        .is_file());

    let rendered = fs::read_to_string(repo.join("INSTRUCTIONS.md")).expect("rendered file");
    assert!(
        !rendered.contains("# Review checklist"),
        "file exports must not be inlined into root target files"
    );
}

#[test]
fn preserves_prayfile_package_order_in_rendered_output() {
    let repo = temporary_directory("pray-install-order");
    fs::create_dir_all(repo.join("packages/zebra/exports")).expect("zebra directories");
    fs::create_dir_all(repo.join("packages/alpha/exports")).expect("alpha directories");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/zebra", path: "packages/zebra", exports: ["zebra"]
agent "sample/alpha", path: "packages/alpha", exports: ["alpha"]
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");
    for (name, export_name, body) in [
        ("zebra", "zebra", "Zebra guidance\n"),
        ("alpha", "alpha", "Alpha guidance\n"),
    ] {
        fs::write(
            repo.join(format!("packages/{name}/{name}.prayspec")),
            format!(
                r#"
Package::Specification.new do |spec|
  spec.name = "sample/{name}"
  spec.version = "1.0.0"
  spec.summary = "{name} guidance"
  spec.files = ["exports/{export_name}.md"]
  spec.exports = {{
    "{export_name}" => {{
      type: "fragment",
      path: "exports/{export_name}.md",
      summary: "{name} guidance"
    }}
  }}
end
"#
            ),
        )
        .expect("write prayspec");
        fs::write(
            repo.join(format!("packages/{name}/exports/{export_name}.md")),
            body,
        )
        .expect("write export");
    }

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let rendered = fs::read_to_string(repo.join("INSTRUCTIONS.md")).expect("rendered file");
    let zebra_index = rendered.find("Zebra guidance").expect("zebra content");
    let alpha_index = rendered.find("Alpha guidance").expect("alpha content");
    assert!(
        zebra_index < alpha_index,
        "rendered output should follow Prayfile package order"
    );
}

#[test]
fn installs_prayer_package_into_a_managed_skill_path() {
    let repo = temporary_directory("pray-install-prayer");
    create_prayer_install_fixture(&repo);

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let materialized_skill = repo.join(".agents/skills/prayer-publisher/SKILL.md");
    assert!(materialized_skill.is_file(), "materialized skill file missing");

    let skill_text = fs::read_to_string(&materialized_skill).expect("materialized skill text");
    assert!(skill_text.contains("Prayer Publisher"));

    let agents = fs::read_to_string(repo.join("AGENTS.md")).expect("rendered agents file");
    assert!(agents.contains("<!-- pray:"));
    assert!(
        !agents.contains("Prayer Publisher"),
        "skill exports must not be inlined into AGENTS.md when skills path is configured"
    );

    let lockfile = fs::read_to_string(repo.join("Prayfile.lock")).expect("lockfile");
    assert!(lockfile.contains("prayer-publisher"));
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
    assert!(stdout.contains("Plan"));
    assert!(stdout.contains("Prayfile.lock"));
    assert!(stdout.contains("INSTRUCTIONS.md"));
}

#[test]
fn apply_reports_materialization_summary() {
    let repo = temporary_directory("pray-apply-summary");
    create_fixture(&repo);

    let apply = run_pray(&repo, &["apply"]);
    assert!(
        apply.status.success(),
        "apply failed: {}",
        String::from_utf8_lossy(&apply.stderr)
    );
    let stdout = String::from_utf8_lossy(&apply.stdout);
    assert!(stdout.contains("Applying"));
    assert!(stdout.contains("sample/base"));
    assert!(stdout.contains("Prayfile.lock created"));
    assert!(stdout.contains("INSTRUCTIONS.md written"));
    assert!(stdout.contains("Apply complete"));

    let second_apply = run_pray(&repo, &["apply"]);
    assert!(
        second_apply.status.success(),
        "second apply failed: {}",
        String::from_utf8_lossy(&second_apply.stderr)
    );
    let second_stdout = String::from_utf8_lossy(&second_apply.stdout);
    assert!(second_stdout.contains("everything up to date"));
    assert!(second_stdout.contains("unchanged"));
}

#[test]
fn install_reports_materialization_summary() {
    let repo = temporary_directory("pray-install-summary");
    create_fixture(&repo);

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );
    let stdout = String::from_utf8_lossy(&install.stdout);
    assert!(stdout.contains("Installing"));
    assert!(stdout.contains("sample/base"));
    assert!(stdout.contains("Prayfile.lock created"));
    assert!(stdout.contains("INSTRUCTIONS.md written"));
    assert!(stdout.contains("Install complete"));

    let second_install = run_pray(&repo, &["install"]);
    assert!(
        second_install.status.success(),
        "second install failed: {}",
        String::from_utf8_lossy(&second_install.stderr)
    );
    let second_stdout = String::from_utf8_lossy(&second_install.stdout);
    assert!(second_stdout.contains("everything up to date"));
}

#[test]
fn install_reports_provisioned_skill_folder_changes() {
    let repo = temporary_directory("pray-install-skill-summary");
    fs::create_dir_all(repo.join("packages/audit-skill/skills/audit")).expect("skill tree");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :agents do
  output "INSTRUCTIONS.md"
  folder ".agents/skills"
end
agent "sample/audit-skill", path: "packages/audit-skill", exports: ["audit"]
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");
    fs::write(
        repo.join("packages/audit-skill/audit-skill.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/audit-skill"
  spec.version = "1.0.0"
  spec.summary = "Audit skill package"
  spec.files = [
    "skills/audit/SKILL.md",
    "skills/audit/details.md"
  ]
  spec.exports = {
    "audit" => {
      type: "folder",
      path: "skills/audit",
      summary: "Audit skill"
    }
  }
end
"#,
    )
    .expect("write prayspec");
    fs::write(
        repo.join("packages/audit-skill/skills/audit/SKILL.md"),
        "# Audit skill\n",
    )
    .expect("write skill");
    fs::write(
        repo.join("packages/audit-skill/skills/audit/details.md"),
        "# Details\n",
    )
    .expect("write details");

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );
    let stdout = String::from_utf8_lossy(&install.stdout);
    assert!(stdout.contains(".agents/skills/audit/"));
    assert!(stdout.contains("2 files"));
    assert!(stdout.contains("provisioned file"));
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
    assert!(!stdout.contains("\"updated_packages\""));
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
