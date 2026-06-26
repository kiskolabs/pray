use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use pray_core::auth::{
    AuthPasskeyChallengeRequest, AuthPasskeyChallengeResponse, AuthPasskeyLoginRequest,
    AuthPasskeyLoginResponse, AuthSshKeyChallengeRequest, AuthSshKeyChallengeResponse,
    AuthSshKeyLoginRequest, AuthSshKeyLoginResponse,
};
use pray_core::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionFile {
    pub server_url: String,
    pub email: String,
    pub token: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum SessionDocument {
    Single(SessionFile),
    Multiple { sessions: Vec<SessionFile> },
}

pub fn session_file_path(root: &Path) -> PathBuf {
    root.join(".pray/session.json")
}

#[allow(dead_code)]
pub fn load_session_email(root: &Path) -> PrayResult<Option<String>> {
    let path = session_file_path(root);
    let Some(session) = load_latest_session(&path)? else {
        return Ok(None);
    };
    Ok(Some(session.email))
}

pub fn login_with_passkey(
    server_url: &str,
    credential_id: &str,
    private_key_path: &Path,
    session_root: &Path,
) -> PrayResult<SessionFile> {
    let challenge: AuthPasskeyChallengeResponse = post_json(
        &format!(
            "{}/v1/auth/passkeys/challenge",
            trim_trailing_slash(server_url)
        ),
        &AuthPasskeyChallengeRequest {
            credential_id: credential_id.to_string(),
        },
    )?;
    let private_key_bytes = fs::read(private_key_path)?;
    let seed: [u8; 32] = private_key_bytes.as_slice().try_into().map_err(|_| {
        PrayError::Unsupported("passkey private key must be 32 raw bytes".to_string())
    })?;
    let signing_key = SigningKey::from_bytes(&seed);
    let signature = STANDARD.encode(signing_key.sign(challenge.challenge.as_bytes()).to_bytes());
    let response: AuthPasskeyLoginResponse = post_json(
        &format!("{}/v1/auth/passkeys/login", trim_trailing_slash(server_url)),
        &AuthPasskeyLoginRequest {
            credential_id: credential_id.to_string(),
            challenge_id: challenge.challenge_id,
            signature,
        },
    )?;
    persist_session(
        session_root,
        SessionFile {
            server_url: server_url.to_string(),
            email: response.email,
            token: response.token,
            kind: "passkey".to_string(),
        },
    )
}

pub fn login_with_ssh_agent(
    server_url: &str,
    public_key_path: &Path,
    session_root: &Path,
) -> PrayResult<SessionFile> {
    let public_key = fs::read_to_string(public_key_path)?;
    let challenge: AuthSshKeyChallengeResponse = post_json(
        &format!(
            "{}/v1/auth/ssh-keys/challenge",
            trim_trailing_slash(server_url)
        ),
        &AuthSshKeyChallengeRequest {
            public_key: public_key.trim().to_string(),
        },
    )?;
    let signature = ssh_agent_sign(public_key.trim(), challenge.challenge.as_bytes())?;
    let response: AuthSshKeyLoginResponse = post_json(
        &format!("{}/v1/auth/ssh-keys/login", trim_trailing_slash(server_url)),
        &AuthSshKeyLoginRequest {
            public_key: public_key.trim().to_string(),
            challenge_id: challenge.challenge_id,
            signature,
        },
    )?;
    persist_session(
        session_root,
        SessionFile {
            server_url: server_url.to_string(),
            email: response.email,
            token: response.token,
            kind: "ssh_key".to_string(),
        },
    )
}

pub fn current_signer(root: &Path) -> Option<String> {
    let session_path = session_file_path(root);
    let session = load_latest_session(&session_path).ok().flatten()?;
    if !session.email.trim().is_empty() {
        return Some(session.email);
    }
    None
}

#[allow(dead_code)]
pub fn resolve_session_token(root: &Path) -> PrayResult<Option<String>> {
    let path = session_file_path(root);
    let Some(session) = load_latest_session(&path)? else {
        return Ok(None);
    };
    Ok(Some(session.token))
}

fn persist_session(root: &Path, session: SessionFile) -> PrayResult<SessionFile> {
    let path = session_file_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut sessions = load_sessions(&path)?.unwrap_or_default();
    if let Some(existing) = sessions
        .iter_mut()
        .find(|existing| existing.server_url == session.server_url)
    {
        *existing = session.clone();
    } else {
        sessions.push(session.clone());
    }
    let document = if sessions.len() == 1 {
        SessionDocument::Single(sessions.remove(0))
    } else {
        SessionDocument::Multiple { sessions }
    };
    fs::write(
        &path,
        serde_json::to_string_pretty(&document)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    )?;
    Ok(session)
}

fn load_latest_session(path: &Path) -> PrayResult<Option<SessionFile>> {
    let Some(sessions) = load_sessions(path)? else {
        return Ok(None);
    };
    Ok(sessions
        .into_iter()
        .rev()
        .find(|session| !session.email.trim().is_empty()))
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
    let sessions = match document {
        SessionDocument::Single(session) => vec![session],
        SessionDocument::Multiple { sessions } => sessions,
    };
    Ok(Some(sessions))
}

fn post_json<Request, Response>(url: &str, body: &Request) -> PrayResult<Response>
where
    Request: Serialize,
    Response: for<'de> Deserialize<'de>,
{
    let (host_port, path) = split_http_url(url)?;
    let mut stream = TcpStream::connect((&host_port.0[..], host_port.1))?;
    let body_text =
        serde_json::to_string(body).map_err(|error| PrayError::Manifest(error.to_string()))?;
    write!(
        stream,
        "POST {} HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path,
        host_port.0,
        host_port.1,
        body_text.len(),
        body_text
    )?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    let (status, response_body) = split_http_response(&response)?;
    if !(200..300).contains(&status) {
        return Err(PrayError::Resolution(format!(
            "request to {url} failed with status {status}: {response_body}"
        )));
    }
    serde_json::from_str(response_body).map_err(|error| PrayError::Parse {
        kind: "auth response",
        message: error.to_string(),
    })
}

fn split_http_url(url: &str) -> PrayResult<((String, u16), String)> {
    let url = url
        .strip_prefix("http://")
        .ok_or_else(|| PrayError::Unsupported("only http:// URLs are supported".to_string()))?;
    let (host_port, path) = url.split_once('/').unwrap_or((url, ""));
    let (host, port) = host_port
        .split_once(':')
        .ok_or_else(|| PrayError::Unsupported("URL must include a port".to_string()))?;
    let port = port
        .parse::<u16>()
        .map_err(|error| PrayError::Unsupported(error.to_string()))?;
    Ok(((host.to_string(), port), format!("/{}", path)))
}

fn split_http_response(response: &str) -> PrayResult<(u16, &str)> {
    let (header, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| PrayError::Resolution("invalid HTTP response".to_string()))?;
    let status = header
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .ok_or_else(|| PrayError::Resolution("missing HTTP status".to_string()))?;
    Ok((status, body))
}

fn trim_trailing_slash(value: &str) -> String {
    value.trim_end_matches('/').to_string()
}

fn ssh_agent_sign(public_key: &str, message: &[u8]) -> PrayResult<String> {
    let agent_socket = std::env::var("SSH_AUTH_SOCK")
        .map_err(|_| PrayError::Unsupported("SSH_AUTH_SOCK is not set".to_string()))?;
    let mut stream = std::os::unix::net::UnixStream::connect(agent_socket)?;
    let (_, raw_key_bytes) = parse_ssh_ed25519_public_key(public_key)?;
    let mut public_key_blob = Vec::new();
    write_ssh_string(&mut public_key_blob, b"ssh-ed25519");
    write_ssh_string(&mut public_key_blob, &raw_key_bytes);
    let mut payload = Vec::new();
    write_ssh_string(&mut payload, &public_key_blob);
    write_ssh_string(&mut payload, message);
    write_u32(&mut payload, 0)?;
    write_ssh_message(&mut stream, 13, &payload)?;
    let (message_type, response) = read_ssh_message(&mut stream)?;
    if message_type != 14 {
        return Err(PrayError::Resolution(format!(
            "ssh agent returned unexpected message type: {message_type}"
        )));
    }
    let signature_blob = read_ssh_string_bytes(&response)?;
    parse_ssh_signature_blob(signature_blob)
}

fn parse_ssh_ed25519_public_key(public_key: &str) -> PrayResult<(String, [u8; 32])> {
    let mut fields = public_key.split_whitespace();
    let algorithm = fields.next().ok_or_else(|| {
        PrayError::Unsupported("public key must include an algorithm".to_string())
    })?;
    if algorithm != "ssh-ed25519" {
        return Err(PrayError::Unsupported(format!(
            "unsupported public key algorithm: {algorithm}"
        )));
    }
    let key_value = fields
        .next()
        .ok_or_else(|| PrayError::Unsupported("public key must include key bytes".to_string()))?;
    let blob = STANDARD
        .decode(key_value.as_bytes())
        .map_err(|error| PrayError::Parse {
            kind: "public key",
            message: error.to_string(),
        })?;
    let mut cursor = blob.as_slice();
    let blob_algorithm = read_ssh_string(&mut cursor)?;
    if blob_algorithm != b"ssh-ed25519" {
        return Err(PrayError::Parse {
            kind: "public key",
            message: "ed25519 public key blob must start with ssh-ed25519".to_string(),
        });
    }
    let key_bytes = read_ssh_string(&mut cursor)?;
    let key_bytes: [u8; 32] = key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| PrayError::Parse {
            kind: "public key",
            message: "ed25519 public key must be 32 bytes".to_string(),
        })?;
    Ok((format!("ssh-ed25519 {key_value}"), key_bytes))
}

fn parse_ssh_signature_blob(signature_blob: Vec<u8>) -> PrayResult<String> {
    let mut cursor = &signature_blob[..];
    let algorithm = read_ssh_string(&mut cursor)?;
    if algorithm != b"ssh-ed25519" {
        return Err(PrayError::Unsupported(format!(
            "unsupported ssh signature algorithm: {}",
            String::from_utf8_lossy(&algorithm)
        )));
    }
    let signature = read_ssh_string(&mut cursor)?;
    Ok(STANDARD.encode(signature))
}

fn write_ssh_message(
    stream: &mut std::os::unix::net::UnixStream,
    message_type: u8,
    payload: &[u8],
) -> PrayResult<()> {
    let mut buffer = Vec::new();
    buffer.push(message_type);
    buffer.extend_from_slice(payload);
    write_u32(stream, buffer.len() as u32)?;
    stream.write_all(&buffer)?;
    Ok(())
}

fn read_ssh_message(stream: &mut std::os::unix::net::UnixStream) -> PrayResult<(u8, Vec<u8>)> {
    let length = read_u32(stream)? as usize;
    let mut buffer = vec![0u8; length];
    stream.read_exact(&mut buffer)?;
    let message_type = buffer
        .first()
        .copied()
        .ok_or_else(|| PrayError::Resolution("empty ssh agent response".to_string()))?;
    Ok((message_type, buffer[1..].to_vec()))
}

fn write_ssh_string(buffer: &mut Vec<u8>, bytes: &[u8]) {
    buffer.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buffer.extend_from_slice(bytes);
}

fn read_ssh_string(cursor: &mut &[u8]) -> PrayResult<Vec<u8>> {
    let length = read_u32_from_slice(cursor)? as usize;
    if cursor.len() < length {
        return Err(PrayError::Resolution(
            "truncated ssh agent response".to_string(),
        ));
    }
    let (value, rest) = cursor.split_at(length);
    *cursor = rest;
    Ok(value.to_vec())
}

fn read_ssh_string_bytes(buffer: &[u8]) -> PrayResult<Vec<u8>> {
    let mut cursor = buffer;
    read_ssh_string(&mut cursor)
}

fn write_u32<T: Write>(writer: &mut T, value: u32) -> PrayResult<()> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

fn read_u32<T: Read>(reader: &mut T) -> PrayResult<u32> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;
    Ok(u32::from_be_bytes(buffer))
}

fn read_u32_from_slice(cursor: &mut &[u8]) -> PrayResult<u32> {
    if cursor.len() < 4 {
        return Err(PrayError::Resolution("truncated ssh field".to_string()));
    }
    let (length_bytes, rest) = cursor.split_at(4);
    *cursor = rest;
    Ok(u32::from_be_bytes(
        length_bytes.try_into().expect("length bytes"),
    ))
}
