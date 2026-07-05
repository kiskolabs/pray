use crate::client_trust::{effective_trust_home, gate_pray_ssh_host, gate_pray_ssh_publisher};
use crate::ssh_identity::active_ssh_user_fingerprint;
use crate::ssh_rpc::{call_stdio, RpcRequest, RpcResponse, SSH_RPC_SPEC};
use crate::{PrayError, PrayResult};
use serde_json::Value;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PraySshTarget {
    pub user: Option<String>,
    pub host: String,
    pub port: u16,
    pub root: Option<PathBuf>,
}

pub fn is_pray_ssh_url(url: &str) -> bool {
    url.starts_with("pray+ssh://") || url.starts_with("ssh+pray://")
}

pub fn parse_pray_ssh_url(url: &str) -> PrayResult<PraySshTarget> {
    let remainder = url
        .strip_prefix("pray+ssh://")
        .or_else(|| url.strip_prefix("ssh+pray://"))
        .ok_or_else(|| PrayError::Parse {
            kind: "pray ssh url",
            message: format!("expected pray+ssh:// url, got {url}"),
        })?;

    let (authority, root) = match remainder.split_once('/') {
        Some((authority, path)) if !path.is_empty() => {
            (authority, Some(PathBuf::from(format!("/{path}"))))
        }
        _ => (remainder, None),
    };

    let (credentials, host_port) = match authority.rsplit_once('@') {
        Some((user, host_port)) => (Some(user.to_string()), host_port),
        None => (None, authority),
    };

    let (host, port) = match host_port.rsplit_once(':') {
        Some((host, port_text)) if !host.contains(']') => {
            let port = port_text.parse::<u16>().map_err(|error| PrayError::Parse {
                kind: "pray ssh url",
                message: format!("invalid port in {url}: {error}"),
            })?;
            (host.to_string(), port)
        }
        _ => (host_port.to_string(), 22),
    };

    if host.is_empty() {
        return Err(PrayError::Parse {
            kind: "pray ssh url",
            message: format!("missing host in {url}"),
        });
    }

    Ok(PraySshTarget {
        user: credentials,
        host,
        port,
        root,
    })
}

pub struct SshRpcSession {
    child: Child,
    stdin: Option<ChildStdin>,
    reader: BufReader<ChildStdout>,
}

impl SshRpcSession {
    pub fn connect(target: &PraySshTarget) -> PrayResult<Self> {
        if target.host == "stdio-host" {
            if let Ok(root) = std::env::var("PRAY_TEST_SSH_STDIO_ROOT") {
                let mut command = Command::new(pray_program());
                command.arg("serve").arg("--stdio").arg("--root").arg(root);
                return Self::connect_stdio(command);
            }
        }

        let mut command = Command::new(ssh_program());
        command
            .arg("-p")
            .arg(target.port.to_string())
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("StrictHostKeyChecking=accept-new");
        if let Some(user) = &target.user {
            command.arg(format!("{user}@{}", target.host));
        } else {
            command.arg(&target.host);
        }
        let mut remote_command = String::from("pray serve --stdio");
        if let Some(root) = &target.root {
            remote_command.push_str(" --root ");
            remote_command.push_str(&shell_escape(root.to_string_lossy().as_ref()));
        }
        command.arg(remote_command);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let mut child = command.spawn().map_err(|error| {
            PrayError::Unsupported(format!("failed to start ssh for pray rpc: {error}"))
        })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| PrayError::Unsupported("rpc stdin unavailable".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| PrayError::Unsupported("rpc stdout unavailable".to_string()))?;
        Ok(Self {
            child,
            stdin: Some(stdin),
            reader: BufReader::new(stdout),
        })
    }

    pub fn connect_stdio(mut command: Command) -> PrayResult<Self> {
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let mut child = command.spawn().map_err(|error| {
            PrayError::Unsupported(format!("failed to start stdio rpc: {error}"))
        })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| PrayError::Unsupported("rpc stdin unavailable".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| PrayError::Unsupported("rpc stdout unavailable".to_string()))?;
        Ok(Self {
            child,
            stdin: Some(stdin),
            reader: BufReader::new(stdout),
        })
    }

    pub fn call(&mut self, method: &str, params: Value) -> PrayResult<RpcResponse> {
        let request_id = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed).to_string();
        let request = RpcRequest::new(request_id, method, params);
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| PrayError::Unsupported("rpc stdin unavailable".to_string()))?;
        let response = call_stdio(&mut self.reader, stdin, &request)?;
        if response.spec != SSH_RPC_SPEC {
            return Err(PrayError::Resolution(format!(
                "unexpected rpc spec in response: {}",
                response.spec
            )));
        }
        Ok(response)
    }

    pub fn call_json(&mut self, method: &str, params: Value) -> PrayResult<Value> {
        let response = self.call(method, params)?;
        if response.status / 100 != 2 {
            let message = response
                .body
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("rpc request failed");
            return Err(PrayError::Resolution(format!(
                "rpc {method} failed with status {}: {message}",
                response.status
            )));
        }
        Ok(response.body)
    }

    pub fn call_bytes(&mut self, method: &str, params: Value) -> PrayResult<Vec<u8>> {
        let response = self.call(method, params)?;
        if response.status / 100 != 2 {
            let message = response
                .body
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("rpc request failed");
            return Err(PrayError::Resolution(format!(
                "rpc {method} failed with status {}: {message}",
                response.status
            )));
        }
        response.decode_body_bytes()
    }
}

impl Drop for SshRpcSession {
    fn drop(&mut self) {
        if let Some(mut stdin) = self.stdin.take() {
            let _ = stdin.flush();
        }
        let _ = self.child.wait();
    }
}

pub fn with_pray_ssh_session<T>(
    source_url: &str,
    operation: impl FnOnce(&mut SshRpcSession) -> PrayResult<T>,
) -> PrayResult<T> {
    let target = parse_pray_ssh_url(source_url)?;
    let home = effective_trust_home()?;
    let _host_key = gate_pray_ssh_host(&home, source_url, &target.host, target.port)?;
    if let Some(publisher) = active_ssh_user_fingerprint() {
        gate_pray_ssh_publisher(&home, source_url, &publisher)?;
    }
    let mut session = SshRpcSession::connect(&target)?;
    operation(&mut session)
}

pub fn ssh_program() -> String {
    [
        "/usr/bin/ssh",
        "/opt/homebrew/bin/ssh",
        "/usr/local/bin/ssh",
        "ssh",
    ]
    .into_iter()
    .find(|candidate| *candidate == "ssh" || Path::new(candidate).exists())
    .unwrap_or("ssh")
    .to_string()
}

fn pray_program() -> String {
    if let Ok(path) = std::env::var("PRAY_TEST_BINARY") {
        return path;
    }
    std::env::current_exe()
        .ok()
        .and_then(|path| path.to_str().map(str::to_string))
        .unwrap_or_else(|| "pray".to_string())
}

fn shell_escape(value: &str) -> String {
    if value.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '/')
    }) {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pray_ssh_url_reads_user_host_port_and_root() {
        let target = parse_pray_ssh_url("pray+ssh://pray@prayers.internal:2222/var/lib/pray")
            .expect("parse url");
        assert_eq!(target.user.as_deref(), Some("pray"));
        assert_eq!(target.host, "prayers.internal");
        assert_eq!(target.port, 2222);
        assert_eq!(target.root, Some(PathBuf::from("/var/lib/pray")));
    }
}
