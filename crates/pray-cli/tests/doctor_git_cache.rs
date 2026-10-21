use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn run_pray(repo: &std::path::Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_pray"))
        .args(arguments)
        .current_dir(repo)
        .env_remove("PRAY_PATH")
        .env_remove("PRAY_FILE_PATH")
        .env_remove("PRAY_ENV")
        .env_remove("PRAY_QUIET")
        .output()
        .expect("run pray")
}

fn temporary_directory(prefix: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{stamp}"));
    fs::create_dir_all(&path).expect("temp dir");
    path
}

#[test]
fn doctor_reports_absent_project_git_cache() {
    let root = temporary_directory("pray-doctor");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
"#,
    )
    .expect("Prayfile");
    let output = run_pray(&root, &["doctor"]);
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("git cache layout"), "{stdout}");
    assert!(stdout.contains("project-git absent"), "{stdout}");
}

#[test]
fn doctor_redacts_url_userinfo() {
    let root = temporary_directory("pray-doctor-redact");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
source "dist", "git+https://user:secret@example.test/prayers.git"
target :tool_a do
  output "INSTRUCTIONS.md"
end
"#,
    )
    .expect("Prayfile");
    let output = run_pray(&root, &["doctor"]);
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("https://***@example.test/prayers.git"),
        "{stdout}"
    );
    assert!(!stdout.contains("user:secret"), "{stdout}");
}

#[test]
fn doctor_quiet_suppresses_status_stdout() {
    let root = temporary_directory("pray-doctor-quiet");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
"#,
    )
    .expect("Prayfile");
    let output = run_pray(&root, &["-q", "doctor"]);
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "quiet doctor should print no status stdout"
    );
}
