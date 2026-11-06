use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMPORARY_DIRECTORY_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn run_pray(arguments: &[&str]) -> std::process::Output {
    run_pray_with_env(arguments, &[])
}

fn run_pray_with_env(arguments: &[&str], environment: &[(&str, &str)]) -> std::process::Output {
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_pray"));
    let binary = binary.canonicalize().unwrap_or(binary);
    let mut command = Command::new(binary);
    command.args(arguments);
    for (key, value) in environment {
        command.env(key, value);
    }
    command.output().expect("run pray")
}

fn empty_project_root() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let suffix = TEMPORARY_DIRECTORY_COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("pray-cli-upgrade-{stamp}-{suffix}"));
    fs::create_dir_all(&root).expect("temporary project root");
    fs::write(root.join("Prayfile"), "prayfile \"1\"\n").expect("minimal Prayfile");
    root
}

#[test]
fn upgrade_command_is_documented_in_help() {
    let output = run_pray(&["help", "upgrade"]);
    assert!(
        output.status.success(),
        "pray help upgrade failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cargo install"));
    assert!(stdout.contains("pray upgrade"));
}

#[test]
fn concise_help_lists_upgrade_command() {
    let output = run_pray(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("upgrade"));
}

#[test]
fn upgrade_notice_uses_changelog_link() {
    // Use an empty project so the notice path does not depend on workspace
    // remote trust or interactive consent for amkisko/prayers sources.
    let project = empty_project_root();
    let output = run_pray_with_env(
        &["--path", project.to_str().expect("project path"), "list"],
        &[("PRAY_TEST_LATEST_VERSION", "99.0.0"), ("CI", "0")],
    );
    let _ = fs::remove_dir_all(&project);
    assert!(
        output.status.success(),
        "pray list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("A new version of pray is available"));
    assert!(stderr.contains("Run: pray upgrade"));
    assert!(stderr.contains("Changelog: https://github.com/kiskolabs/pray/blob/main/CHANGELOG.md"));
}
