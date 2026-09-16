#[path = "install_network_support.rs"]
mod support;

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use support::temporary_directory;

fn run_pray(directory: &std::path::Path, arguments: &[&str]) -> std::process::Output {
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_pray"));
    let binary = binary.canonicalize().unwrap_or(binary);
    Command::new(binary)
        .args(arguments)
        .current_dir(directory)
        .output()
        .expect("run pray")
}

#[test]
fn prayer_init_scaffolds_a_package_repository_layout() {
    let repository = temporary_directory("pray-prayer-init");

    let output = run_pray(&repository, &["prayer", "init"]);
    assert!(
        output.status.success(),
        "prayer init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let prayspec_files: Vec<_> = fs::read_dir(&repository)
        .expect("read repository")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("prayspec"))
        .collect();
    assert_eq!(prayspec_files.len(), 1);
    assert!(repository.join("README.md").is_file());
    assert!(repository.join("exports").is_dir());
}

#[test]
fn prayer_init_in_a_project_writes_an_unversioned_path_package() {
    let repository = temporary_directory("pray-prayer-init-local");
    fs::write(repository.join("Prayfile"), "prayfile \"1\"\n").expect("Prayfile");

    let output = run_pray(&repository, &["prayer", "init"]);
    assert!(
        output.status.success(),
        "prayer init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let prayspec = fs::read_to_string(repository.join("prayers/project/project.prayspec"))
        .expect("local prayspec");
    assert!(prayspec.contains("spec.name = \"local/project\""));
    assert!(!prayspec.contains("spec.version"));
    assert!(repository
        .join("prayers/project/exports/project.md")
        .is_file());

    let manifest = fs::read_to_string(repository.join("Prayfile")).expect("Prayfile");
    assert!(manifest.contains("source \"local\", path: \"prayers\""));
    assert!(manifest.contains("pray \"local/project\""));
    assert!(!manifest.contains("path: \"prayers/project\""));
}

#[test]
fn prayer_init_uses_an_author_chosen_path_source_directory() {
    let repository = temporary_directory("pray-prayer-init-path");
    fs::write(repository.join("Prayfile"), "prayfile \"1\"\n").expect("Prayfile");

    let output = run_pray(
        &repository,
        &["prayer", "init", "notes", "--path", "guidance"],
    );
    assert!(
        output.status.success(),
        "prayer init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(repository.join("guidance/notes/notes.prayspec").is_file());
    let manifest = fs::read_to_string(repository.join("Prayfile")).expect("Prayfile");
    assert!(manifest.contains("source \"local\", path: \"guidance\""));
    assert!(manifest.contains("pray \"local/notes\""));
    assert!(!manifest.contains("path: \"guidance/notes\""));
}

#[test]
fn prayer_init_declares_the_prayer_once_inside_compose() {
    let repository = temporary_directory("pray-prayer-init-compose");
    fs::write(
        repository.join("Prayfile"),
        "prayfile \"1\"\ncompose \"AGENTS.md\" do\n  pray \".agents/project.md\"\nend\n",
    )
    .expect("Prayfile");

    let output = run_pray(&repository, &["prayer", "init"]);
    assert!(
        output.status.success(),
        "prayer init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let manifest = fs::read_to_string(repository.join("Prayfile")).expect("Prayfile");
    let declarations = manifest.matches("pray \"local/project\"").count();
    assert_eq!(
        declarations, 1,
        "expected one pray \"local/project\", got:\n{manifest}"
    );
    assert!(manifest.contains("compose \"AGENTS.md\" do\n  pray \"local/project\""));
    assert!(!manifest.contains("path: \"prayers/project\""));
}

#[test]
fn prayer_init_reuses_an_existing_path_source() {
    let repository = temporary_directory("pray-prayer-init-existing-source");
    fs::write(
        repository.join("Prayfile"),
        "prayfile \"1\"\nsource \"local\", path: \"guidance\"\n",
    )
    .expect("Prayfile");

    let output = run_pray(&repository, &["prayer", "init"]);
    assert!(
        output.status.success(),
        "prayer init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(repository
        .join("guidance/project/project.prayspec")
        .is_file());
    let manifest = fs::read_to_string(repository.join("Prayfile")).expect("Prayfile");
    assert_eq!(
        manifest.matches("source \"local\"").count(),
        1,
        "expected one path source, got:\n{manifest}"
    );
    assert!(!repository.join("prayers/project").exists());
}

#[test]
fn prayer_init_install_locks_local_and_package_refuses() {
    let repository = temporary_directory("pray-prayer-init-install");
    fs::write(repository.join("Prayfile"), "prayfile \"1\"\n").expect("Prayfile");

    let init = run_pray(&repository, &["prayer", "init", "notes"]);
    assert!(
        init.status.success(),
        "prayer init failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let install = run_pray(&repository, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );
    let lock = fs::read_to_string(repository.join("Prayfile.lock")).expect("lock");
    assert!(
        lock.contains("name = \"local/notes\"") && lock.contains("version = \"local\""),
        "expected local lock version, got:\n{lock}"
    );

    let package = run_pray(&repository, &["package"]);
    assert!(!package.status.success());
    let stderr = String::from_utf8_lossy(&package.stderr);
    assert!(
        stderr.contains("needs a version"),
        "expected pack refusal, got: {stderr}"
    );
}

#[test]
fn prayer_init_refuses_distribution_layout_name() {
    let repository = temporary_directory("pray-prayer-init-v1");
    fs::write(repository.join("Prayfile"), "prayfile \"1\"\n").expect("Prayfile");

    let output = run_pray(&repository, &["prayer", "init", "v1"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("reserved"),
        "expected reserved-name error, got: {stderr}"
    );
}

#[test]
fn repo_init_scaffolds_a_distribution_repository_layout() {
    let repository = temporary_directory("pray-repo-init");

    let output = run_pray(&repository, &["repo", "init"]);
    assert!(
        output.status.success(),
        "repo init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(repository.join("prayers").is_dir());
    assert!(repository.join("prayers/v1/packages").is_dir());
    assert!(repository.join("prayers/v1/artifacts").is_dir());

    let index_text = fs::read_to_string(repository.join("prayers/v1/index.json")).expect("index");
    let index: Value = serde_json::from_str(&index_text).expect("index json");
    assert_eq!(index["spec"], "prayfile-distribution-1");
    assert!(index["packages"].as_array().expect("packages").is_empty());

    let trust_text = fs::read_to_string(repository.join("prayers/v1/trust.json")).expect("trust");
    let trust: Value = serde_json::from_str(&trust_text).expect("trust json");
    assert_eq!(trust["email_confirmation"], "required");
    assert!(!trust["passkeys_enabled"].as_bool().expect("passkeys"));
    assert!(!trust["ssh_keys_enabled"].as_bool().expect("ssh keys"));
    assert!(!trust["ssh_agent_signing_enabled"]
        .as_bool()
        .expect("ssh agent"));

    let distribution_text =
        fs::read_to_string(repository.join("prayers/v1/distribution.json")).expect("distribution");
    let distribution: Value = serde_json::from_str(&distribution_text).expect("distribution json");
    assert_eq!(distribution["spec"], "pray-distribution-config-1");
    assert!(distribution["protocols"]
        .as_array()
        .expect("protocols")
        .is_empty());
}
