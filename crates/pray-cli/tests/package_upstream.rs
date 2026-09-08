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

fn write_fork_package(catalog: &Path, source: &Path) {
    let root = catalog.join("packages/fork-base");
    fs::create_dir_all(root.join("exports")).expect("fork directories");
    fs::copy(
        source.join("packages/base/README.md"),
        root.join("README.md"),
    )
    .expect("copy readme");
    fs::copy(
        source.join("packages/base/exports/testing-basics.md"),
        root.join("exports/testing-basics.md"),
    )
    .expect("copy testing");
    fs::write(
        root.join("fork-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "fork/base"
  spec.version = "1.0.0"
  spec.summary = "forked guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
  spec.upstream "sample/base", "~> 1.4"
end
"#,
    )
    .expect("write fork prayspec");
}

fn write_fork_prayfile(catalog: &Path, distribution: &Path) {
    fs::write(
        catalog.join("Prayfile"),
        format!(
            r#"
prayfile "1"
source "sample", "git+file://{distribution}"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "fork/base", "~> 1.0", path: "packages/fork-base"
render mode: :managed, conflict: :fail, churn: :minimal
"#,
            distribution = distribution.display()
        ),
    )
    .expect("write fork Prayfile");
}

fn bump_upstream_version(source: &Path) {
    fs::write(
        source.join("packages/base/sample-base.prayspec"),
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
    .expect("rewrite upstream prayspec");
    fs::write(
        source.join("packages/base/exports/testing-basics.md"),
        "Testing guidance v2\n",
    )
    .expect("rewrite upstream export");
}

#[test]
fn update_replaces_clean_fork_from_locked_upstream() {
    let workspace = temporary_directory("pray-package-upstream");
    let source_repo = workspace.join("source");
    let distribution_repo = workspace.join("distribution");
    let prayers_root = distribution_repo.join("prayers");
    let catalog_repo = workspace.join("catalog");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&distribution_repo).expect("distribution workspace");
    fs::create_dir_all(&catalog_repo).expect("catalog workspace");

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

    write_fork_package(&catalog_repo, &source_repo);
    write_fork_prayfile(&catalog_repo, &distribution_repo);
    assert_success(&run_pray(&catalog_repo, &["install"]), "install fork");
    let lockfile = fs::read_to_string(catalog_repo.join("Prayfile.lock")).expect("lockfile");
    assert!(
        lockfile.contains("sample/base"),
        "install should record upstream name:\n{lockfile}"
    );
    assert!(
        lockfile.contains("1.4.3"),
        "install should lock upstream 1.4.3:\n{lockfile}"
    );

    bump_upstream_version(&source_repo);
    assert_success(
        &run_pray(
            &source_repo,
            &[
                "publish",
                "--root",
                prayers_root.to_str().expect("distribution path"),
            ],
        ),
        "publish base 1.4.4",
    );
    assert_success(&git(&distribution_repo, &["add", "-A"]), "git add bump");
    assert_success(
        &git(&distribution_repo, &["commit", "-m", "publish 1.4.4"]),
        "git commit bump",
    );

    let update = run_pray(&catalog_repo, &["update"]);
    assert_success(&update, "update fork");
    let testing =
        fs::read_to_string(catalog_repo.join("packages/fork-base/exports/testing-basics.md"))
            .expect("fork export");
    assert_eq!(testing, "Testing guidance v2\n");
    let fork_spec = fs::read_to_string(catalog_repo.join("packages/fork-base/fork-base.prayspec"))
        .expect("spec");
    assert!(fork_spec.contains("fork/base"));
    assert!(fork_spec.contains("1.0.0"));
    let updated_lock = fs::read_to_string(catalog_repo.join("Prayfile.lock")).expect("lockfile");
    assert!(
        updated_lock.contains("1.4.4"),
        "update should lock upstream 1.4.4:\n{updated_lock}"
    );
}
