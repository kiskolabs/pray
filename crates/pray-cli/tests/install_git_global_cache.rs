#[path = "install_network_support.rs"]
mod support;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use support::{create_add_fixture, run_pray, temporary_directory};

fn git(directory: &Path, arguments: &[&str]) -> Output {
    Command::new("git")
        .current_dir(directory)
        .args(arguments)
        .output()
        .expect("run git")
}

fn assert_success(output: &Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_pray_with_cache(repo: &Path, cache_root: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pray"))
        .env("PRAY_CACHE", cache_root)
        .args(arguments)
        .current_dir(repo)
        .output()
        .expect("run pray")
}

fn consumer_prayfile(distribution_repo: &Path) -> String {
    consumer_prayfile_with_constraint(distribution_repo, "~> 1.4")
}

fn consumer_prayfile_with_constraint(distribution_repo: &Path, constraint: &str) -> String {
    format!(
        r#"
prayfile "1"
source "dist", "git+file://{distribution}"
agent "sample/base", "{constraint}", source: "dist"
target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
"#,
        distribution = distribution_repo.display(),
        constraint = constraint,
    )
}

#[test]
fn update_refreshes_project_cache_seeded_from_stale_global_mirror() {
    let workspace = temporary_directory("pray-global-git-cache-refresh");
    let global_cache = workspace.join("global-cache");
    let source_repo = workspace.join("source");
    let distribution_repo = workspace.join("distribution");
    let prayers_root = distribution_repo.join("prayers");
    let first_consumer = workspace.join("first-consumer");
    let second_consumer = workspace.join("second-consumer");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&distribution_repo).expect("distribution workspace");
    fs::create_dir_all(&first_consumer).expect("first consumer");
    fs::create_dir_all(&second_consumer).expect("second consumer");

    create_add_fixture(&source_repo);
    assert_success(
        &run_pray(
            &source_repo,
            &["add", "sample/base", "--path", "packages/base"],
        ),
        "add",
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
        "publish",
    );

    assert_success(
        &git(&distribution_repo, &["init", "-b", "main"]),
        "git init",
    );
    assert_success(
        &git(&distribution_repo, &["config", "user.name", "Pray Test"]),
        "git user.name",
    );
    assert_success(
        &git(
            &distribution_repo,
            &["config", "user.email", "pray@example.com"],
        ),
        "git user.email",
    );
    assert_success(&git(&distribution_repo, &["add", "-A"]), "git add");
    assert_success(
        &git(
            &distribution_repo,
            &["commit", "-m", "initial distribution"],
        ),
        "git commit",
    );

    let initial_commit =
        String::from_utf8_lossy(&git(&distribution_repo, &["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();

    fs::write(
        first_consumer.join("Prayfile"),
        consumer_prayfile(&distribution_repo),
    )
    .expect("first consumer Prayfile");
    assert_success(
        &run_pray_with_cache(&first_consumer, &global_cache, &["install"]),
        "first install",
    );
    assert!(
        global_cache.join("git").is_dir(),
        "first install should populate the shared global git cache"
    );

    fs::write(
        source_repo.join("packages/base/exports/testing-basics.md"),
        "Updated testing guidance after republish\n",
    )
    .expect("rewrite export");
    assert_success(
        &run_pray(
            &source_repo,
            &[
                "publish",
                "--root",
                prayers_root.to_str().expect("distribution path"),
            ],
        ),
        "republish",
    );
    assert_success(
        &git(&distribution_repo, &["add", "-A"]),
        "git add republish",
    );
    assert_success(
        &git(
            &distribution_repo,
            &["commit", "-m", "republish same version"],
        ),
        "git commit republish",
    );

    let updated_commit =
        String::from_utf8_lossy(&git(&distribution_repo, &["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();
    assert_ne!(initial_commit, updated_commit);

    fs::write(
        second_consumer.join("Prayfile"),
        consumer_prayfile(&distribution_repo),
    )
    .expect("second consumer Prayfile");
    assert_success(
        &run_pray_with_cache(&second_consumer, &global_cache, &["install"]),
        "second install",
    );

    let stale_lockfile =
        fs::read_to_string(second_consumer.join("Prayfile.lock")).expect("lockfile");
    assert!(
        stale_lockfile.contains(&initial_commit),
        "second consumer should seed from the stale global mirror on install:\n{stale_lockfile}"
    );

    assert_success(
        &run_pray_with_cache(&second_consumer, &global_cache, &["update"]),
        "second update",
    );

    let updated_lockfile =
        fs::read_to_string(second_consumer.join("Prayfile.lock")).expect("lockfile");
    assert!(
        updated_lockfile.contains(&updated_commit),
        "update should fetch from the original source URL, not the stale mirror:\n{updated_lockfile}"
    );
    let updated_instructions =
        fs::read_to_string(second_consumer.join("INSTRUCTIONS.md")).expect("instructions");
    assert!(updated_instructions.contains("Updated testing guidance after republish"));
}

#[test]
fn install_refreshes_stale_global_mirror_when_constraints_require_newer_versions() {
    let workspace = temporary_directory("pray-global-git-cache-major-install");
    let global_cache = workspace.join("global-cache");
    let source_repo = workspace.join("source");
    let distribution_repo = workspace.join("distribution");
    let prayers_root = distribution_repo.join("prayers");
    let first_consumer = workspace.join("first-consumer");
    let second_consumer = workspace.join("second-consumer");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&distribution_repo).expect("distribution workspace");
    fs::create_dir_all(&first_consumer).expect("first consumer");
    fs::create_dir_all(&second_consumer).expect("second consumer");

    create_add_fixture(&source_repo);
    assert_success(
        &run_pray(
            &source_repo,
            &["add", "sample/base", "--path", "packages/base"],
        ),
        "add",
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
        "publish",
    );

    assert_success(
        &git(&distribution_repo, &["init", "-b", "main"]),
        "git init",
    );
    assert_success(
        &git(&distribution_repo, &["config", "user.name", "Pray Test"]),
        "git user.name",
    );
    assert_success(
        &git(
            &distribution_repo,
            &["config", "user.email", "pray@example.com"],
        ),
        "git user.email",
    );
    assert_success(&git(&distribution_repo, &["add", "-A"]), "git add");
    assert_success(
        &git(
            &distribution_repo,
            &["commit", "-m", "initial distribution"],
        ),
        "git commit",
    );

    let initial_commit =
        String::from_utf8_lossy(&git(&distribution_repo, &["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();

    fs::write(
        first_consumer.join("Prayfile"),
        consumer_prayfile(&distribution_repo),
    )
    .expect("first consumer Prayfile");
    assert_success(
        &run_pray_with_cache(&first_consumer, &global_cache, &["install"]),
        "first install",
    );

    fs::write(
        source_repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "2.0.0"
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
    .expect("rewrite prayspec for major");
    assert_success(
        &run_pray(
            &source_repo,
            &[
                "publish",
                "--root",
                prayers_root.to_str().expect("distribution path"),
            ],
        ),
        "republish major",
    );
    assert_success(&git(&distribution_repo, &["add", "-A"]), "git add major");
    assert_success(
        &git(
            &distribution_repo,
            &["commit", "-m", "publish major version"],
        ),
        "git commit major",
    );

    let updated_commit =
        String::from_utf8_lossy(&git(&distribution_repo, &["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();
    assert_ne!(initial_commit, updated_commit);

    fs::write(
        second_consumer.join("Prayfile"),
        consumer_prayfile_with_constraint(&distribution_repo, "~> 2.0"),
    )
    .expect("second consumer Prayfile");
    assert_success(
        &run_pray_with_cache(&second_consumer, &global_cache, &["install"]),
        "second install",
    );

    let lockfile = fs::read_to_string(second_consumer.join("Prayfile.lock")).expect("lockfile");
    assert!(
        lockfile.contains(&updated_commit),
        "install should refresh the stale global mirror when constraints require newer versions:\n{lockfile}"
    );
    assert!(
        lockfile.contains("2.0.0"),
        "install should resolve the major version from the refreshed distribution:\n{lockfile}"
    );
}
