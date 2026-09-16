#[path = "install_support.rs"]
mod support;

use std::fs;

use support::{run_pray, temporary_directory};

fn create_scoped_local_fixture(repo: &std::path::Path) {
    fs::create_dir_all(repo.join("packages/base/exports")).expect("package directories");
    fs::create_dir_all(repo.join(".agents")).expect("local directories");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
compose "AGENTS.md" do
  pray ".agents/project.md"
  pray "sample/base", "~> 1.4", path: "packages/base"
end
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");
    fs::write(
        repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.summary = "shared guidance"
  spec.files = ["exports/testing-basics.md"]
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
    .expect("write prayspec");
    fs::write(
        repo.join("packages/base/exports/testing-basics.md"),
        "Testing guidance\n",
    )
    .expect("write export");
    fs::write(repo.join(".agents/project.md"), "Local guidance\n").expect("write local");
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn install_rewrites_compose_when_a_local_source_changes() {
    let repo = temporary_directory("pray-install-local-change");
    create_scoped_local_fixture(&repo);

    let first = run_pray(&repo, &["install"]);
    assert!(
        first.status.success(),
        "first install failed: {}",
        stderr(&first)
    );
    let first_stdout = stdout(&first);
    assert!(
        first_stdout.contains(".agents/project.md"),
        "expected local artifact in install report, got:\n{first_stdout}"
    );
    assert!(
        first_stdout.contains("sha256:"),
        "expected checksum in install report, got:\n{first_stdout}"
    );

    fs::write(
        repo.join(".agents/project.md"),
        "Local guidance\nChanged local guidance\n",
    )
    .expect("rewrite local file");

    let second = run_pray(&repo, &["install"]);
    assert!(
        second.status.success(),
        "second install failed: {}",
        stderr(&second)
    );
    let second_stdout = stdout(&second);
    assert!(
        second_stdout.contains(".agents/project.md"),
        "expected local artifact on changed install, got:\n{second_stdout}"
    );
    assert!(
        second_stdout.contains("sha256:"),
        "expected checksum on changed install, got:\n{second_stdout}"
    );
    assert!(
        second_stdout.contains("was sha256:"),
        "expected previous checksum when local source changed, got:\n{second_stdout}"
    );
    assert!(
        second_stdout.contains("AGENTS.md") && second_stdout.contains("updated"),
        "expected AGENTS.md updated, got:\n{second_stdout}"
    );

    let rendered = fs::read_to_string(repo.join("AGENTS.md")).expect("rendered file");
    assert!(
        rendered.contains("Changed local guidance"),
        "compose dest must re-embed the changed local source"
    );
}

#[test]
fn install_reports_checked_when_a_local_source_is_unchanged() {
    let repo = temporary_directory("pray-install-local-checked");
    create_scoped_local_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let second = run_pray(&repo, &["install"]);
    assert!(
        second.status.success(),
        "reinstall failed: {}",
        stderr(&second)
    );
    let second_stdout = stdout(&second);
    assert!(
        second_stdout.contains(".agents/project.md") && second_stdout.contains("checked"),
        "expected checked local checksum, got:\n{second_stdout}"
    );
    assert!(
        second_stdout.contains("AGENTS.md unchanged"),
        "expected dest unchanged when local source is unchanged, got:\n{second_stdout}"
    );
    assert!(
        second_stdout.contains("1 local file"),
        "expected footer to count the local artifact separately from packages, got:\n{second_stdout}"
    );
}

#[test]
fn plan_and_outdated_report_a_changed_local_source() {
    let repo = temporary_directory("pray-plan-outdated-local");
    create_scoped_local_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    fs::write(repo.join(".agents/project.md"), "Same line count change\n")
        .expect("rewrite local file");

    let plan = run_pray(&repo, &["plan"]);
    assert!(plan.status.success(), "plan failed: {}", stderr(&plan));
    let plan_stdout = stdout(&plan);
    assert!(
        plan_stdout.contains(".agents/project.md"),
        "expected local artifact in plan, got:\n{plan_stdout}"
    );
    assert!(
        plan_stdout.contains("sha256:"),
        "expected checksum in plan, got:\n{plan_stdout}"
    );
    assert!(
        plan_stdout.contains("AGENTS.md") && plan_stdout.contains("would be updated"),
        "expected dest would be updated, got:\n{plan_stdout}"
    );

    let outdated = run_pray(&repo, &["outdated"]);
    assert!(
        outdated.status.success(),
        "outdated failed: {}",
        stderr(&outdated)
    );
    let outdated_stdout = stdout(&outdated);
    assert!(
        outdated_stdout.contains(".agents/project.md"),
        "expected local artifact in outdated, got:\n{outdated_stdout}"
    );
    assert!(
        outdated_stdout.contains("sha256:"),
        "expected checksum in outdated, got:\n{outdated_stdout}"
    );
}

#[test]
fn update_rewrites_compose_when_a_local_source_changes() {
    let repo = temporary_directory("pray-update-local-change");
    create_scoped_local_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    fs::write(
        repo.join(".agents/project.md"),
        "Local guidance\nUpdated by update\n",
    )
    .expect("rewrite local file");

    let update = run_pray(&repo, &["update"]);
    assert!(
        update.status.success(),
        "update failed: {}",
        stderr(&update)
    );
    let update_stdout = stdout(&update);
    assert!(
        update_stdout.contains(".agents/project.md"),
        "expected local artifact in update, got:\n{update_stdout}"
    );
    assert!(
        update_stdout.contains("was sha256:"),
        "expected previous checksum in update, got:\n{update_stdout}"
    );

    let rendered = fs::read_to_string(repo.join("AGENTS.md")).expect("rendered file");
    assert!(
        rendered.contains("Updated by update"),
        "update must re-embed the changed local source"
    );
}
