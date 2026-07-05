use crate::ssh_identity::active_ssh_user_fingerprint;
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const SSH_PUBLISHERS_PATH: &str = "v1/ssh_publishers.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshPublisherConfig {
    #[serde(default)]
    pub publishers: Vec<SshPublisherEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshPublisherEntry {
    pub fingerprint: String,
    pub id: String,
    #[serde(default)]
    pub push: bool,
}

pub fn read_ssh_publishers(root: &Path) -> PrayResult<Option<SshPublisherConfig>> {
    let path = root.join(SSH_PUBLISHERS_PATH);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let config: SshPublisherConfig =
        serde_json::from_str(&text).map_err(|error| PrayError::Parse {
            kind: "ssh publishers",
            message: error.to_string(),
        })?;
    Ok(Some(config))
}

pub fn active_ssh_publisher_id() -> Option<String> {
    active_ssh_user_fingerprint()
}

pub fn authorize_ssh_push(root: &Path) -> PrayResult<()> {
    let Some(config) = read_ssh_publishers(root)? else {
        return Ok(());
    };
    if config.publishers.is_empty() {
        return Ok(());
    }

    let publisher_id = active_ssh_publisher_id().ok_or_else(|| {
        PrayError::Resolution(
            "ssh push requires an SSH user fingerprint (set PRAY_SSH_USER_FINGERPRINT, SSH_USER_FINGERPRINT, or PRAY_SSH_PUBLISHER) when v1/ssh_publishers.json is configured".to_string(),
        )
    })?;

    let authorized = config.publishers.iter().any(|entry| {
        entry.push
            && (crate::ssh_identity::normalize_identity(&entry.id)
                == crate::ssh_identity::normalize_identity(&publisher_id)
                || crate::ssh_identity::normalize_identity(&entry.fingerprint)
                    == crate::ssh_identity::normalize_identity(&publisher_id))
    });
    if authorized {
        Ok(())
    } else {
        Err(PrayError::Resolution(format!(
            "ssh publisher {publisher_id} is not authorized to push"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn authorize_allows_push_when_publishers_file_missing() {
        let workspace = std::env::temp_dir().join(format!(
            "pray-ssh-publishers-missing-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&workspace);
        fs::create_dir_all(&workspace).expect("workspace");
        authorize_ssh_push(&workspace).expect("open mode");
        let _ = fs::remove_dir_all(&workspace);
    }

    #[test]
    fn authorize_requires_publisher_when_config_present() {
        let workspace =
            std::env::temp_dir().join(format!("pray-ssh-publishers-gate-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        fs::create_dir_all(workspace.join("v1")).expect("v1");
        let mut file = fs::File::create(workspace.join(SSH_PUBLISHERS_PATH)).expect("publishers");
        writeln!(
            file,
            r#"{{"publishers":[{{"fingerprint":"SHA256:abc","id":"team-ci","push":true}}]}}"#
        )
        .expect("write publishers");

        let error = authorize_ssh_push(&workspace).expect_err("missing publisher");
        assert!(error.to_string().contains("SSH user fingerprint"));

        std::env::set_var("PRAY_SSH_USER_FINGERPRINT", "SHA256:abc");
        authorize_ssh_push(&workspace).expect("authorized fingerprint");
        std::env::remove_var("PRAY_SSH_USER_FINGERPRINT");
        let _ = fs::remove_dir_all(&workspace);
    }
}
