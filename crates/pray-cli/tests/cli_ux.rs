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
        assert!(stdout.contains("Usage: pray [OPTIONS] <COMMAND>"));
        assert!(stdout.contains("Getting started:"));
        assert!(stdout.contains("See 'pray help <command>'"));
        assert!(stdout.contains("Options:"));
        assert!(!stdout.contains("Documentation:"));
        assert!(!stdout.contains("Exit codes:"));
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
        assert!(stdout.contains("Usage: pray install"));
        assert!(!stdout.contains("Documentation:"));
    }
}

#[test]
fn listed_commands_have_per_command_help() {
    for command in [
        "remove", "list", "format", "fmt", "render", "version", "login", "sync",
    ] {
        let output = run_pray(&["help", command]);
        assert!(
            output.status.success(),
            "pray help {command} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Usage: pray"),
            "pray help {command} missing Usage:\n{stdout}"
        );
        assert!(
            !stdout.contains("unknown command"),
            "pray help {command} treated as unknown:\n{stdout}"
        );
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
    assert!(stderr.contains("See 'pray --help'."));
    assert!(!stderr.contains("unsupported feature"));
}

#[test]
fn no_input_flag_is_documented_in_help() {
    let output = run_pray(&["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--no-input"));
    assert!(stdout.contains("completion bash|zsh|fish"));
}

#[test]
fn completion_bash_prints_script() {
    let output = run_pray(&["completion", "bash"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("complete -F _pray pray"));
    assert!(stdout.contains("install"));
    assert!(stdout.contains("completion"));
}

#[test]
fn completion_unknown_shell_exits_usage() {
    let output = run_pray(&["completion", "tcsh"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("usage error:"));
    assert!(stderr.contains("bash, zsh, or fish"));
}

#[cfg(not(feature = "auth"))]
#[test]
fn slim_build_help_omits_serve() {
    let output = run_pray(&["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!stdout.contains("serve [--root PATH]"));
}
