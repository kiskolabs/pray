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
}
