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
fn packages_list_describes_update() {
    let output = run_pray(&["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("re-resolve packages and git sources within constraints"),
        "packages list should describe update:\n{stdout}"
    );
}

#[test]
fn workflow_help_names_what_each_command_compares() {
    let install = stdout_of(&["help", "install"]);
    assert!(install.contains("listed local compose"));
    assert!(install.contains("spec.upstream"));
    assert!(install.contains("pray update"));
    assert!(install.contains("pray plan"));

    let verify = stdout_of(&["help", "verify"]);
    assert!(verify.contains("dest managed spans versus Prayfile.lock"));
    assert!(verify.contains("pray drift"));
    assert!(verify.contains("pray plan"));

    let drift = stdout_of(&["help", "drift"]);
    assert!(drift.contains("fresh render"));
    assert!(drift.contains("pray plan"));

    let render = stdout_of(&["help", "render"]);
    assert!(render.contains("write dest and Prayfile.lock"));
    assert!(!render.contains("without updating the lockfile"));
    assert!(render.contains("pray plan"));
    assert!(render.contains("--check"));

    let plan = stdout_of(&["help", "plan"]);
    assert!(plan.contains("dry-run"));
    assert!(plan.contains("install"));

    let update = stdout_of(&["help", "update"]);
    assert!(update.contains("pray install"));
    assert!(update.contains("local compose"));
    assert!(update.contains("--latest"));

    let prayer = stdout_of(&["help", "prayer"]);
    assert!(prayer.contains("prayers/"));
    assert!(prayer.contains("pray prayer init"));
}

#[test]
fn prayer_and_repo_help_name_product_and_catalog_layouts() {
    let prayer = stdout_of(&["help", "prayer"]);
    assert!(prayer.contains("prayers/v1"));
    assert!(prayer.contains("prayers/<name>/"));
    assert!(!prayer.contains("root packages/"));
    let repo = stdout_of(&["help", "repo"]);
    assert!(repo.contains("prayers/v1"));
    assert!(repo.contains("prayers/<name>/"));
    assert!(!repo.contains("root packages/"));
}

fn stdout_of(arguments: &[&str]) -> String {
    let output = run_pray(arguments);
    assert!(
        output.status.success(),
        "pray {:?} failed: {}",
        arguments,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
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
