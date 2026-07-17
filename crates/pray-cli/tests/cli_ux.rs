use std::path::PathBuf;
use std::process::Command;

fn run_pray(arguments: &[&str]) -> std::process::Output {
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_pray"));
    let binary = binary.canonicalize().unwrap_or(binary);
    Command::new(binary)
        .args(arguments)
        .output()
        .expect("run pray")
}

#[test]
fn bare_invocation_prints_concise_help() {
    let cases: Vec<&[&str]> = vec![&[], &["--help"], &["-h"], &["help"]];
    for arguments in cases {
        let output = run_pray(arguments);
        assert!(
            output.status.success(),
            "pray {:?} failed: {}",
            arguments,
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("reproducible inference input"));
        assert!(stdout.contains("pray help"));
        assert!(stdout.contains("Getting started:"));
    }
}

#[test]
fn per_command_help_for_install() {
    let cases: Vec<&[&str]> = vec![
        &["help", "install"],
        &["install", "--help"],
        &["install", "-h"],
    ];
    for arguments in cases {
        let output = run_pray(arguments);
        assert!(
            output.status.success(),
            "pray {:?} failed: {}",
            arguments,
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("--offline"));
        assert!(stdout.contains("install"));
    }
}

#[test]
fn unknown_command_suggests_install_for_typo() {
    let output = run_pray(&["instal"]);
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("usage error:"));
    assert!(stderr.contains("unknown command: instal"));
    assert!(stderr.contains("Did you mean `install`?"));
    assert!(!stderr.contains("unsupported feature"));
}

#[test]
fn no_input_flag_is_documented_in_help() {
    let output = run_pray(&["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--no-input"));
}
