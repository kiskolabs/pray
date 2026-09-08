#[path = "install_support.rs"]
mod support;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use support::{create_add_fixture, run_pray, temporary_directory};

fn assert_success(output: &Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git(directory: &Path, arguments: &[&str]) -> Output {
    Command::new("git")
        .current_dir(directory)
        .args(arguments)
        .output()
        .expect("run git")
}

fn create_extra_package(repo: &Path) {
    fs::create_dir_all(repo.join("packages/extra/exports")).expect("extra directories");
    fs::write(
        repo.join("packages/extra/sample-extra.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/extra"
  spec.version = "1.0.0"
  spec.summary = "extra guidance"
  spec.files = ["README.md", "exports/extra-note.md"]
  spec.exports = {
    "extra-note" => {
      type: "fragment",
      path: "exports/extra-note.md",
      summary: "Extra guidance"
    }
  }
end
"#,
    )
    .expect("write extra prayspec");
    fs::write(repo.join("packages/extra/README.md"), "extra readme\n").expect("write extra readme");
    fs::write(
        repo.join("packages/extra/exports/extra-note.md"),
        "Extra guidance\n",
    )
    .expect("write extra export");
}

fn write_consumer_prayfile(consumer: &Path, distribution: &Path, include_extra: bool) {
    let extra = if include_extra {
        "agent \"sample/extra\", \"~> 1.0\", source: \"dist\"\n"
    } else {
        ""
    };
    fs::write(
        consumer.join("Prayfile"),
        format!(
            r#"
prayfile "1"
source "dist", "git+file://{distribution}"
agent "sample/base", "~> 1.4", source: "dist"
{extra}target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
"#,
            distribution = distribution.display()
        ),
    )
    .expect("write consumer Prayfile");
}

fn init_distribution(distribution: &Path) {
    assert_success(&git(distribution, &["init", "-b", "main"]), "git init");
    assert_success(
        &git(distribution, &["config", "user.name", "Pray Test"]),
        "git user.name",
    );
    assert_success(
        &git(distribution, &["config", "user.email", "pray@example.com"]),
        "git user.email",
    );
    assert_success(
        &git(distribution, &["config", "commit.gpgsign", "false"]),
        "git gpgsign",
    );
    assert_success(&git(distribution, &["add", "-A"]), "git add");
    assert_success(
        &git(distribution, &["commit", "-m", "initial distribution"]),
        "git commit",
    );
}

#[test]
fn install_refreshes_when_a_new_catalog_package_is_declared() {
    let workspace = temporary_directory("pray-install-git-catalog");
    let source_repo = workspace.join("source");
    let distribution_repo = workspace.join("distribution");
    let prayers_root = distribution_repo.join("prayers");
    let consumer_repo = workspace.join("consumer");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&distribution_repo).expect("distribution workspace");
    fs::create_dir_all(&consumer_repo).expect("consumer workspace");

    create_add_fixture(&source_repo);
    assert_success(
        &run_pray(
            &source_repo,
            &["add", "sample/base", "--path", "packages/base"],
        ),
        "add base",
    );
    assert_success(
        &run_pray(
            &source_repo,
            &[
                "publish",
                "--root",
                prayers_root.to_str().expect("distribution path"),
            ],
        ),
        "publish base",
    );
    init_distribution(&distribution_repo);
    let initial_commit =
        String::from_utf8_lossy(&git(&distribution_repo, &["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();
    write_consumer_prayfile(&consumer_repo, &distribution_repo, false);
    assert_success(&run_pray(&consumer_repo, &["install"]), "install base");

    create_extra_package(&source_repo);
    assert_success(
        &run_pray(
            &source_repo,
            &["add", "sample/extra", "--path", "packages/extra"],
        ),
        "add extra",
    );
    assert_success(
        &run_pray(
            &source_repo,
            &[
                "publish",
                "--root",
                prayers_root.to_str().expect("distribution path"),
            ],
        ),
        "publish extra",
    );
    assert_success(&git(&distribution_repo, &["add", "-A"]), "git add extra");
    assert_success(
        &git(
            &distribution_repo,
            &["commit", "-m", "publish extra package"],
        ),
        "git commit extra",
    );
    let updated_commit =
        String::from_utf8_lossy(&git(&distribution_repo, &["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();
    assert_ne!(initial_commit, updated_commit);

    write_consumer_prayfile(&consumer_repo, &distribution_repo, true);

    let locked_install = run_pray(&consumer_repo, &["install", "--locked"]);
    assert!(
        !locked_install.status.success(),
        "locked install should refuse a package absent from the pinned catalog"
    );
    let locked_stderr = String::from_utf8_lossy(&locked_install.stderr);
    assert!(
        locked_stderr.contains(&initial_commit),
        "locked failure should name the pinned revision:\n{locked_stderr}"
    );
    assert!(
        locked_stderr.contains("pray update"),
        "locked failure should name pray update:\n{locked_stderr}"
    );
    assert!(
        !locked_stderr.contains("check the package name"),
        "locked failure should not look like a missing package name:\n{locked_stderr}"
    );

    let install = run_pray(&consumer_repo, &["install"]);
    assert!(
        install.status.success(),
        "install should refresh the catalog for a newly declared package: {}",
        String::from_utf8_lossy(&install.stderr)
    );
    let lockfile = fs::read_to_string(consumer_repo.join("Prayfile.lock")).expect("lockfile");
    assert!(
        lockfile.contains(&updated_commit),
        "install should move the source pin to the catalog that has the new package:\n{lockfile}"
    );
    assert!(
        !lockfile.contains(&initial_commit),
        "install should not keep the stale catalog revision:\n{lockfile}"
    );
    assert!(lockfile.contains("sample/extra"));
}

#[test]
fn install_names_locked_revision_when_catalog_package_never_appears() {
    let workspace = temporary_directory("pray-install-git-catalog-miss");
    let source_repo = workspace.join("source");
    let distribution_repo = workspace.join("distribution");
    let prayers_root = distribution_repo.join("prayers");
    let consumer_repo = workspace.join("consumer");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&distribution_repo).expect("distribution workspace");
    fs::create_dir_all(&consumer_repo).expect("consumer workspace");

    create_add_fixture(&source_repo);
    assert_success(
        &run_pray(
            &source_repo,
            &["add", "sample/base", "--path", "packages/base"],
        ),
        "add base",
    );
    assert_success(
        &run_pray(
            &source_repo,
            &[
                "publish",
                "--root",
                prayers_root.to_str().expect("distribution path"),
            ],
        ),
        "publish base",
    );
    init_distribution(&distribution_repo);
    let initial_commit =
        String::from_utf8_lossy(&git(&distribution_repo, &["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();
    write_consumer_prayfile(&consumer_repo, &distribution_repo, false);
    assert_success(&run_pray(&consumer_repo, &["install"]), "install base");

    fs::write(
        consumer_repo.join("Prayfile"),
        format!(
            r#"
prayfile "1"
source "dist", "git+file://{distribution}"
agent "sample/base", "~> 1.4", source: "dist"
agent "sample/missing", "~> 1.0", source: "dist"
target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
"#,
            distribution = distribution_repo.display()
        ),
    )
    .expect("write consumer Prayfile with missing package");

    let install = run_pray(&consumer_repo, &["install"]);
    assert!(
        !install.status.success(),
        "install should fail when the package is absent after refresh"
    );
    let stderr = String::from_utf8_lossy(&install.stderr);
    assert!(
        stderr.contains(&initial_commit),
        "failure after refresh should name the locked revision:\n{stderr}"
    );
    assert!(
        stderr.contains("pray update"),
        "failure after refresh should name pray update:\n{stderr}"
    );
    assert!(
        !stderr.contains("check the package name"),
        "failure after refresh should not look like a missing package name:\n{stderr}"
    );
}
