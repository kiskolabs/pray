use pray_core::client_trust::effective_trust_home;
use pray_core::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionFile {
    pub server_url: String,
    pub email: String,
    pub token: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_fingerprint: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum SessionDocument {
    Single(SessionFile),
    Multiple { sessions: Vec<SessionFile> },
}

pub fn persist_session(root: &Path, session: SessionFile) -> PrayResult<SessionFile> {
    let path = prepare_session_path(root)?;
    let mut sessions = load_sessions(&path)?.unwrap_or_default();
    if let Some(existing) = sessions
        .iter_mut()
        .find(|existing| existing.server_url == session.server_url)
    {
        *existing = session.clone();
    } else {
        sessions.push(session.clone());
    }
    write_sessions(&path, sessions)?;
    Ok(session)
}

pub fn current_signer(root: &Path) -> Option<String> {
    load_latest_session(root)
        .ok()
        .flatten()
        .map(|session| session.email)
        .filter(|email| !email.trim().is_empty())
}

pub fn current_signer_fingerprint(root: &Path) -> Option<String> {
    load_latest_session(root)
        .ok()
        .flatten()?
        .signer_fingerprint
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn load_latest_session(root: &Path) -> PrayResult<Option<SessionFile>> {
    let path = prepare_session_path(root)?;
    Ok(load_sessions(&path)?.and_then(|sessions| {
        sessions
            .into_iter()
            .rev()
            .find(|session| !session.email.trim().is_empty())
    }))
}

fn prepare_session_path(root: &Path) -> PrayResult<PathBuf> {
    let path = effective_trust_home()?.join("session.json");
    let legacy = root.join(".pray/session.json");
    if legacy.is_file() {
        let mut sessions = load_sessions(&path)?.unwrap_or_default();
        for session in load_sessions(&legacy)?.unwrap_or_default() {
            if !sessions
                .iter()
                .any(|entry| entry.server_url == session.server_url)
            {
                sessions.push(session);
            }
        }
        write_sessions(&path, sessions)?;
        fs::remove_file(legacy)?;
    }
    Ok(path)
}

fn load_sessions(path: &Path) -> PrayResult<Option<Vec<SessionFile>>> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(None);
    };
    let document: SessionDocument =
        serde_json::from_str(&text).map_err(|error| PrayError::Parse {
            kind: "session file",
            message: error.to_string(),
        })?;
    Ok(Some(match document {
        SessionDocument::Single(session) => vec![session],
        SessionDocument::Multiple { sessions } => sessions,
    }))
}

fn write_sessions(path: &Path, mut sessions: Vec<SessionFile>) -> PrayResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let document = if sessions.len() == 1 {
        SessionDocument::Single(sessions.remove(0))
    } else {
        SessionDocument::Multiple { sessions }
    };
    let bytes = format!(
        "{}\n",
        serde_json::to_string_pretty(&document)
            .map_err(|error| PrayError::Manifest(error.to_string()))?
    );
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    file.write_all(bytes.as_bytes())?;
    file.sync_all()?;
    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temporary, path)?;
    Ok(())
}
