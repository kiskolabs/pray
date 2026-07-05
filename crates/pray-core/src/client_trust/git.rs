use crate::{PrayError, PrayResult};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::policy::{normalize_key, source_scope_id};

pub fn trust_allowed_signers_path(home: &Path, source_url: &str) -> PathBuf {
    home.join("trust")
        .join("allowed_signers")
        .join(format!("{}.signers", source_scope_id(source_url)))
}

pub fn trust_git_env(home: &Path, source_url: &str) -> PrayResult<Vec<(String, String)>> {
    let path = trust_allowed_signers_path(home, source_url);
    let directory = home.join("trust").join("allowed_signers");
    std::fs::create_dir_all(&directory)?;
    if !path.is_file() {
        std::fs::write(&path, "")?;
    }
    Ok(vec![
        ("GIT_CONFIG_COUNT".to_string(), "1".to_string()),
        (
            "GIT_CONFIG_KEY_0".to_string(),
            "gpg.ssh.allowedSignersFile".to_string(),
        ),
        (
            "GIT_CONFIG_VALUE_0".to_string(),
            path.to_string_lossy().to_string(),
        ),
    ])
}

pub fn trust_git_output(
    home: &Path,
    source_url: &str,
    repository: &Path,
    arguments: &[&str],
) -> PrayResult<String> {
    let environment = trust_git_env(home, source_url)?;
    git_output(repository, arguments, &environment)
}

pub fn trust_git_run(
    home: &Path,
    source_url: &str,
    repository: &Path,
    arguments: &[&str],
) -> PrayResult<()> {
    let environment = trust_git_env(home, source_url)?;
    git_run(repository, arguments, &environment)
}

pub fn commit_signing_key(home: &Path, source_url: &str, repository: &Path) -> Option<String> {
    trust_git_output(home, source_url, repository, &["log", "-1", "--format=%GK"])
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn commit_signing_fingerprint(
    home: &Path,
    source_url: &str,
    repository: &Path,
) -> Option<String> {
    trust_git_output(home, source_url, repository, &["log", "-1", "--format=%GF"])
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn repository_signing_keys(home: &Path, source_url: &str, repository: &Path) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(key) =
        commit_signing_key(home, source_url, repository).map(|value| normalize_key(&value))
    {
        keys.push(key);
    }
    if let Some(fingerprint) =
        commit_signing_fingerprint(home, source_url, repository).map(|value| normalize_key(&value))
    {
        if !keys.iter().any(|existing| existing == &fingerprint) {
            keys.push(fingerprint);
        }
    }
    keys
}

pub fn git_run(
    repository: &Path,
    arguments: &[&str],
    environment: &[(String, String)],
) -> PrayResult<()> {
    let output = git_command(repository, arguments, environment)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(git_command_error("git", arguments, output))
    }
}

pub fn git_output(
    repository: &Path,
    arguments: &[&str],
    environment: &[(String, String)],
) -> PrayResult<String> {
    let output = git_command(repository, arguments, environment)?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(git_command_error("git", arguments, output))
    }
}

fn git_command(
    repository: &Path,
    arguments: &[&str],
    environment: &[(String, String)],
) -> PrayResult<Output> {
    let mut command = Command::new(git_program());
    command.current_dir(repository).args(arguments);
    for (key, value) in environment {
        command.env(key, value);
    }
    command
        .output()
        .map_err(|error| PrayError::Unsupported(format!("failed to run git: {error}")))
}

fn git_program() -> &'static str {
    for candidate in [
        "/usr/bin/git",
        "/opt/homebrew/bin/git",
        "/usr/local/bin/git",
        "git",
    ] {
        if candidate == "git" || Path::new(candidate).exists() {
            return candidate;
        }
    }
    "git"
}

fn git_command_error(program: &str, arguments: &[&str], output: Output) -> PrayError {
    let preview = format!("{program} {}", arguments.join(" "));
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let mut message = format!("{preview} failed with status {}", output.status);
    if !stderr.is_empty() {
        message.push_str(&format!(": {stderr}"));
    } else if !stdout.is_empty() {
        message.push_str(&format!(": {stdout}"));
    }
    PrayError::Integrity(message)
}

pub fn is_remote_git_url(clone_url: &str) -> bool {
    let lower = clone_url.trim().to_ascii_lowercase();
    if lower.starts_with("file://") {
        return false;
    }
    if Path::new(clone_url).exists() {
        return false;
    }
    lower.starts_with("git@")
        || lower.starts_with("ssh://")
        || lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("git://")
}
