use crate::Command;
use pray_core::{PrayError, PrayResult};
use std::path::PathBuf;

pub(crate) fn parse_login_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let mut servers = Vec::new();
    let mut email = None;
    let mut credential_id = None;
    let mut passkey_key = None;
    let mut public_key = None;
    let mut ssh_agent = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--server" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "login requires a URL after --server".to_string(),
                    ));
                };
                servers.push(value);
            }
            "--email" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "login requires an email after --email".to_string(),
                    ));
                };
                email = Some(value);
            }
            "--credential-id" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "login requires a credential id after --credential-id".to_string(),
                    ));
                };
                credential_id = Some(value);
            }
            "--passkey-key" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "login requires a path after --passkey-key".to_string(),
                    ));
                };
                passkey_key = Some(PathBuf::from(value));
            }
            "--public-key" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "login requires a path after --public-key".to_string(),
                    ));
                };
                public_key = Some(PathBuf::from(value));
            }
            "--ssh-agent" => ssh_agent = true,
            other if other.starts_with("--") => {
                return Err(PrayError::Unsupported(format!(
                    "unknown login flag: {other}"
                )))
            }
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unexpected login argument: {other}"
                )))
            }
        }
    }
    if servers.is_empty() {
        return Err(PrayError::Unsupported(
            "login requires at least one --server URL".to_string(),
        ));
    }
    let email = email
        .ok_or_else(|| PrayError::Unsupported("login requires --email ADDRESS".to_string()))?;
    if passkey_key.is_some() == ssh_agent || (passkey_key.is_none() && public_key.is_none()) {
        return Err(PrayError::Unsupported(
            "login requires exactly one authentication mode".to_string(),
        ));
    }
    if passkey_key.is_some() && credential_id.is_none() {
        return Err(PrayError::Unsupported(
            "passkey login requires --credential-id".to_string(),
        ));
    }
    if ssh_agent && public_key.is_none() {
        return Err(PrayError::Unsupported(
            "ssh-agent login requires --public-key".to_string(),
        ));
    }
    Ok(Command::Login {
        servers,
        email,
        credential_id,
        passkey_key,
        public_key,
        ssh_agent,
    })
}

#[cfg(feature = "auth")]
pub(crate) fn parse_serve_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let mut root = PathBuf::from(".");
    let mut host = "127.0.0.1".to_string();
    let mut port = 7429u16;
    let mut stdio = false;
    let mut allow_open_push = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "serve requires a path after --root".to_string(),
                    ));
                };
                root = PathBuf::from(value);
            }
            "--host" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "serve requires a host after --host".to_string(),
                    ));
                };
                host = value;
            }
            "--port" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "serve requires a port after --port".to_string(),
                    ));
                };
                port = value
                    .parse::<u16>()
                    .map_err(|error| PrayError::Unsupported(error.to_string()))?;
            }
            "--stdio" => stdio = true,
            "--allow-open-push" => allow_open_push = true,
            other if other.starts_with("--") => {
                return Err(PrayError::Unsupported(format!(
                    "unknown serve flag: {other}"
                )))
            }
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unexpected serve argument: {other}"
                )))
            }
        }
    }
    Ok(Command::Serve {
        root,
        host,
        port,
        stdio,
        allow_open_push,
    })
}
