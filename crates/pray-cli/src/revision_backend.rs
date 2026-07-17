use pray_core::{PrayError, PrayResult};
use serde::Deserialize;
use std::path::Path;
use std::process::{Command, Output};

#[derive(Debug, Clone)]
pub(crate) struct RepositoryRevisionConfig {
    pub(crate) remote: Option<String>,
    pub(crate) push: bool,
    pub(crate) commit_message: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct OtherRevisionConfig {
    pub(crate) commit_command: CommandSpec,
    pub(crate) push_command: Option<CommandSpec>,
    pub(crate) push: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct CommandSpec {
    pub(crate) program: String,

    #[serde(default)]
    pub(crate) args: Vec<String>,
}

impl CommandSpec {
    pub(crate) fn validate(self) -> PrayResult<Self> {
        if self.program.trim().is_empty() {
            return Err(PrayError::Unsupported(
                "revision command program cannot be empty".to_string(),
            ));
        }
        Ok(self)
    }
}

pub(crate) fn record_git_revision(
    root: &Path,
    config: RepositoryRevisionConfig,
    commit_message: String,
) -> PrayResult<()> {
    run_command_success(root, "git", &["add", "-A"])?;
    if git_has_staged_changes(root)? {
        run_command_success(root, "git", &["commit", "-m", &commit_message])?;
    }

    if config.push {
        let remote = config.remote.as_deref().ok_or_else(|| {
            PrayError::Unsupported(
                "git revisioning requires a remote when push is enabled".to_string(),
            )
        })?;
        let branch = git_current_branch(root)?;
        let refspec = format!("HEAD:refs/heads/{branch}");
        push_git_revision(root, remote, &refspec, &branch)?;
    }

    Ok(())
}

pub(crate) fn record_hg_revision(
    root: &Path,
    config: RepositoryRevisionConfig,
    commit_message: String,
) -> PrayResult<()> {
    run_command_success(root, "hg", &["addremove"])?;
    if !hg_has_changes(root)? {
        return Ok(());
    }

    run_command_success(root, "hg", &["commit", "-m", &commit_message])?;

    if config.push {
        let remote = config.remote.as_deref().ok_or_else(|| {
            PrayError::Unsupported(
                "hg revisioning requires a remote when push is enabled".to_string(),
            )
        })?;
        run_command_success(root, "hg", &["push", remote])?;
    }

    Ok(())
}

pub(crate) fn record_other_revision(root: &Path, config: OtherRevisionConfig) -> PrayResult<()> {
    run_command_spec(root, &config.commit_command)?;
    if config.push {
        let push_command = config.push_command.ok_or_else(|| {
            PrayError::Unsupported(
                "revision backend 'other' requires push_command when push is enabled".to_string(),
            )
        })?;
        run_command_spec(root, &push_command)?;
    }
    Ok(())
}

fn git_has_staged_changes(root: &Path) -> PrayResult<bool> {
    let output = run_command_output(root, "git", &["diff", "--cached", "--quiet"])?;
    if output.status.success() {
        return Ok(false);
    }
    if output.status.code() == Some(1) {
        return Ok(true);
    }
    Err(command_error("git diff --cached --quiet", output))
}

fn hg_has_changes(root: &Path) -> PrayResult<bool> {
    let output = run_command_output(root, "hg", &["status"])?;
    if !output.status.success() {
        return Err(command_error("hg status", output));
    }
    Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

fn git_current_branch(root: &Path) -> PrayResult<String> {
    let output = run_command_output(root, "git", &["branch", "--show-current"])?;
    if !output.status.success() {
        return Err(command_error("git branch --show-current", output));
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        return Err(PrayError::Resolution(
            "git revisioning requires a checked-out branch to push".to_string(),
        ));
    }
    Ok(branch)
}

fn run_command_success(root: &Path, program: &str, arguments: &[&str]) -> PrayResult<Output> {
    let output = run_command_output(root, program, arguments)?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(command_error(program, output))
    }
}

fn run_command_output(root: &Path, program: &str, arguments: &[&str]) -> PrayResult<Output> {
    Command::new(command_program(program))
        .current_dir(root)
        .args(arguments)
        .output()
        .map_err(|error| PrayError::Unsupported(format!("failed to run `{program}`: {error}")))
}

fn command_program(program: &str) -> String {
    if program == "git" {
        [
            "/usr/bin/git",
            "/opt/homebrew/bin/git",
            "/usr/local/bin/git",
            "git",
        ]
        .into_iter()
        .find(|candidate| std::path::Path::new(candidate).exists() || *candidate == "git")
        .unwrap_or("git")
        .to_string()
    } else {
        program.to_string()
    }
}

fn push_git_revision(root: &Path, remote: &str, refspec: &str, branch: &str) -> PrayResult<()> {
    let output = run_command_output(root, "git", &["push", remote, refspec])?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    if stderr.contains("non-fast-forward")
        || stderr.contains("fetch first")
        || stderr.contains("rejected")
        || stderr.contains("diverged")
    {
        return Err(PrayError::Resolution(format!(
            "git push was rejected for branch {branch}; fetch or rebase the remote changes and retry"
        )));
    }

    Err(command_error("git push", output))
}

fn run_command_spec(root: &Path, spec: &CommandSpec) -> PrayResult<()> {
    let output = Command::new(&spec.program)
        .current_dir(root)
        .args(&spec.args)
        .output()
        .map_err(|error| {
            PrayError::Unsupported(format!("failed to run `{}`: {error}", spec.program))
        })?;
    if output.status.success() {
        Ok(())
    } else {
        Err(command_error(&spec.program, output))
    }
}

fn command_error(program: &str, output: Output) -> PrayError {
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
