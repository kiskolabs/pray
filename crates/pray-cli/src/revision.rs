use crate::revision_backend::{
    record_git_revision, record_hg_revision, record_other_revision, CommandSpec,
    OtherRevisionConfig, RepositoryRevisionConfig,
};
use pray_core::{PrayError, PrayResult};
use serde::Deserialize;
use std::fs;
use std::path::Path;

const REVISION_CONFIG_PATH: &str = "v1/revision.json";

#[derive(Clone, Copy, Debug)]
pub enum RevisionAction {
    Publish,
    Sync,
}

impl RevisionAction {
    fn default_commit_message(self) -> &'static str {
        match self {
            Self::Publish => "pray publish: update distribution state",
            Self::Sync => "pray sync: update distribution state",
        }
    }
}

#[derive(Debug)]
enum RevisionContext {
    Git(RepositoryRevisionConfig),
    Hg(RepositoryRevisionConfig),
    Other(OtherRevisionConfig),
}

#[derive(Debug, Deserialize)]
struct RevisionConfigFile {
    #[serde(default)]
    backend: Option<RevisionBackend>,

    #[serde(default)]
    remote: Option<String>,

    #[serde(default)]
    push: Option<bool>,

    #[serde(default)]
    commit_message: Option<String>,

    #[serde(default)]
    commit_command: Option<CommandSpec>,

    #[serde(default)]
    push_command: Option<CommandSpec>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RevisionBackend {
    Git,
    Hg,
    Other,
}

pub fn record_root_revision(root: &Path, action: RevisionAction) -> PrayResult<()> {
    match load_revision_context(root, action)? {
        Some(context) => context.record(root, action),
        None => Ok(()),
    }
}

fn load_revision_context(
    root: &Path,
    action: RevisionAction,
) -> PrayResult<Option<RevisionContext>> {
    let config_path = root.join(REVISION_CONFIG_PATH);
    if config_path.exists() {
        let text = fs::read_to_string(&config_path)?;
        let config: RevisionConfigFile =
            serde_json::from_str(&text).map_err(|error| PrayError::Parse {
                kind: "revision config",
                message: error.to_string(),
            })?;
        return Ok(Some(revision_context_from_config(root, config, action)?));
    }

    Ok(detect_revision_context(root, action))
}

fn revision_context_from_config(
    root: &Path,
    config: RevisionConfigFile,
    action: RevisionAction,
) -> PrayResult<RevisionContext> {
    let backend = config.backend.unwrap_or_else(|| {
        if config.commit_command.is_some() || config.push_command.is_some() {
            RevisionBackend::Other
        } else if root.join(".git").exists() {
            RevisionBackend::Git
        } else if root.join(".hg").exists() {
            RevisionBackend::Hg
        } else {
            RevisionBackend::Git
        }
    });

    match backend {
        RevisionBackend::Git => {
            let remote = normalize_optional_string(config.remote);
            let push = config.push.unwrap_or(remote.is_some());
            Ok(RevisionContext::Git(RepositoryRevisionConfig {
                remote,
                push,
                commit_message: normalize_optional_string(config.commit_message)
                    .or_else(|| Some(action.default_commit_message().to_string())),
            }))
        }
        RevisionBackend::Hg => {
            let remote = normalize_optional_string(config.remote);
            let push = config.push.unwrap_or(remote.is_some());
            Ok(RevisionContext::Hg(RepositoryRevisionConfig {
                remote,
                push,
                commit_message: normalize_optional_string(config.commit_message)
                    .or_else(|| Some(action.default_commit_message().to_string())),
            }))
        }
        RevisionBackend::Other => {
            let commit_command = config
                .commit_command
                .ok_or_else(|| {
                    PrayError::Unsupported(
                        "revision backend 'other' requires commit_command".to_string(),
                    )
                })?
                .validate()?;
            let push_command_present = config.push_command.is_some();
            let push_command = config.push_command.map(CommandSpec::validate).transpose()?;
            Ok(RevisionContext::Other(OtherRevisionConfig {
                commit_command,
                push_command,
                push: config.push.unwrap_or(push_command_present),
            }))
        }
    }
}

fn detect_revision_context(root: &Path, action: RevisionAction) -> Option<RevisionContext> {
    if root.join(".git").exists() {
        return Some(RevisionContext::Git(RepositoryRevisionConfig {
            remote: None,
            push: false,
            commit_message: Some(action.default_commit_message().to_string()),
        }));
    }

    if root.join(".hg").exists() {
        return Some(RevisionContext::Hg(RepositoryRevisionConfig {
            remote: None,
            push: false,
            commit_message: Some(action.default_commit_message().to_string()),
        }));
    }

    None
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|entry| {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

impl RevisionContext {
    fn record(self, root: &Path, action: RevisionAction) -> PrayResult<()> {
        match self {
            Self::Git(config) => {
                let commit_message = config
                    .commit_message
                    .clone()
                    .unwrap_or_else(|| action.default_commit_message().to_string());
                record_git_revision(root, config, commit_message)
            }
            Self::Hg(config) => {
                let commit_message = config
                    .commit_message
                    .clone()
                    .unwrap_or_else(|| action.default_commit_message().to_string());
                record_hg_revision(root, config, commit_message)
            }
            Self::Other(config) => record_other_revision(root, config),
        }
    }
}
