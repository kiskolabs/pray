use std::path::PathBuf;
use std::process::Command;

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
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root");
    let output = run_pray_with_env(
        &[
            "--path",
            workspace.to_str().expect("workspace path"),
            "list",
        ],
        &[("PRAY_TEST_LATEST_VERSION", "99.0.0"), ("CI", "0")],
    );
    assert!(
        output.status.success(),
        "pray list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("A new version of pray is available"));
    assert!(stderr.contains("Run: pray upgrade"));
    assert!(stderr.contains(
        "Changelog: https://github.com/kiskolabs/pray/blob/main/CHANGELOG.md"
    ));
}
