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
fn version_command_prints_package_version() {
    for arguments in [["version"], ["--version"], ["-V"]] {
        let output = run_pray(&arguments);
        assert!(
            output.status.success(),
            "pray {} failed: {}",
            arguments[0],
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            String::from_utf8_lossy(&output.stdout)
        );
    }
}
