use crate::{PrayError, PrayResult};
use std::path::Path;
use std::process::Command;

pub(crate) fn run_git_success(root: &Path, arguments: &[&str]) -> PrayResult<()> {
    let output = run_git_command(root, arguments)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(command_error("git", output))
    }
}

pub(crate) fn run_git_command(root: &Path, arguments: &[&str]) -> PrayResult<std::process::Output> {
    Command::new(git_program())
        .current_dir(root)
        .args(arguments)
        .output()
        .map_err(|error| PrayError::Unsupported(format!("failed to run `git`: {error}")))
}

pub(crate) fn git_program() -> String {
    [
        "/usr/bin/git",
        "/opt/homebrew/bin/git",
        "/usr/local/bin/git",
        "git",
    ]
    .into_iter()
    .find(|candidate| Path::new(candidate).exists() || *candidate == "git")
    .unwrap_or("git")
    .to_string()
}

pub(crate) fn command_error(program: &str, output: std::process::Output) -> PrayError {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let mut message = format!("{program} failed with status {}", output.status);
    if !stderr.is_empty() {
        message.push_str(&format!(": {stderr}"));
    } else if !stdout.is_empty() {
        message.push_str(&format!(": {stdout}"));
    }
    PrayError::Resolution(message)
}
