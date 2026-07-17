#[path = "install_support.rs"]
mod support;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use support::{create_add_fixture, read_package_archive, run_pray, temporary_directory};

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

fn remove_registry_cache(repo: &Path) {
    let cache = repo.join(".pray/cache/registry");
    if cache.exists() {
        fs::remove_dir_all(&cache).expect("remove registry cache");
    }
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
fn install_offline_uses_vendored_packages_when_cache_is_missing() {
    let workspace = temporary_directory("pray-vendor-offline");
    let source_repo = workspace.join("source");
    let distribution_repo = workspace.join("distribution");
    let prayers_root = distribution_repo.join("prayers");
    let consumer_repo = workspace.join("consumer");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&distribution_repo).expect("distribution workspace");
    fs::create_dir_all(&consumer_repo).expect("consumer workspace");

    create_add_fixture(&source_repo);
    let add = run_pray(
        &source_repo,
        &["add", "sample/base", "--path", "packages/base"],
    );
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let publish = run_pray(
        &source_repo,
        &[
            "publish",
            "--root",
            prayers_root.to_str().expect("distribution path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
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

    fs::write(
        consumer_repo.join("Prayfile"),
        format!(
            r#"
prayfile "1"
source "dist", "git+file://{distribution}"
agent "sample/base", "~> 1.4", source: "dist"
target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
"#,
            distribution = distribution_repo.display()
        ),
    )
    .expect("write consumer Prayfile");

    let install = run_pray(&consumer_repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let vendor = run_pray(&consumer_repo, &["vendor"]);
    assert!(
        vendor.status.success(),
        "vendor failed: {}",
        String::from_utf8_lossy(&vendor.stderr)
    );

    remove_registry_cache(&consumer_repo);

    let offline = run_pray(&consumer_repo, &["install", "--offline"]);
    assert!(
        offline.status.success(),
        "offline install from vendor failed: {}",
        String::from_utf8_lossy(&offline.stderr)
    );
    assert!(consumer_repo.join("INSTRUCTIONS.md").is_file());
}
