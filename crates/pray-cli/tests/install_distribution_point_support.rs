use ed25519_dalek::Signer;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::thread;

pub fn spawn_mock_ssh_agent(
    socket_path: &Path,
    signing_key: ed25519_dalek::SigningKey,
) -> thread::JoinHandle<()> {
    if socket_path.exists() {
        fs::remove_file(socket_path).expect("remove stale ssh agent socket");
    }
    let listener =
        std::os::unix::net::UnixListener::bind(socket_path).expect("bind mock ssh agent");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept ssh agent connection");
        let (message_type, payload) =
            read_ssh_message(&mut stream).expect("read ssh agent request");
        assert_eq!(message_type, 13);
        let mut cursor = payload.as_slice();
        let _public_key_blob = read_ssh_string(&mut cursor).expect("read public key blob");
        let message = read_ssh_string(&mut cursor).expect("read message");
        let _flags = read_u32(&mut cursor).expect("read flags");
        let signature = signing_key.sign(&message).to_bytes();
        let mut signature_blob = Vec::new();
        write_ssh_string(&mut signature_blob, b"ssh-ed25519");
        write_ssh_string(&mut signature_blob, &signature);
        let mut response_payload = Vec::new();
        write_ssh_string(&mut response_payload, &signature_blob);
        write_ssh_message(&mut stream, 14, &response_payload).expect("write ssh agent response");
    })
}

pub fn run_pray_login_ssh_agent(
    repo: &Path,
    server_url: &str,
    email: &str,
    public_key_path: &Path,
    ssh_auth_sock: &Path,
) {
    let login = Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "login",
            "--server",
            server_url,
            "--email",
            email,
            "--public-key",
            public_key_path.to_str().expect("public key path"),
            "--ssh-agent",
        ])
        .current_dir(repo)
        .env("SSH_AUTH_SOCK", ssh_auth_sock)
        .output()
        .expect("run ssh-agent login");
    assert!(
        login.status.success(),
        "ssh-agent login failed: {}",
        String::from_utf8_lossy(&login.stderr)
    );
}

pub fn write_public_key_file(repo: &Path, filename: &str, public_key: &str) -> std::path::PathBuf {
    let path = repo.join(filename);
    fs::write(&path, format!("{public_key}\n")).expect("write public key file");
    path
}

fn read_ssh_string(cursor: &mut &[u8]) -> std::io::Result<Vec<u8>> {
    let length = read_u32(cursor)? as usize;
    if cursor.len() < length {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "truncated ssh string",
        ));
    }
    let (value, rest) = cursor.split_at(length);
    *cursor = rest;
    Ok(value.to_vec())
}

fn read_u32(cursor: &mut &[u8]) -> std::io::Result<u32> {
    if cursor.len() < 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "truncated ssh field",
        ));
    }
    let (length_bytes, rest) = cursor.split_at(4);
    *cursor = rest;
    Ok(u32::from_be_bytes(
        length_bytes.try_into().expect("length bytes"),
    ))
}

fn read_ssh_message(stream: &mut std::os::unix::net::UnixStream) -> std::io::Result<(u8, Vec<u8>)> {
    let length = read_u32_from_stream(stream)? as usize;
    let mut buffer = vec![0u8; length];
    stream.read_exact(&mut buffer)?;
    let message_type = *buffer
        .first()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "empty response"))?;
    Ok((message_type, buffer[1..].to_vec()))
}

fn write_ssh_message(
    stream: &mut std::os::unix::net::UnixStream,
    message_type: u8,
    payload: &[u8],
) -> std::io::Result<()> {
    let mut buffer = Vec::new();
    buffer.push(message_type);
    buffer.extend_from_slice(payload);
    write_u32_to_stream(stream, buffer.len() as u32)?;
    stream.write_all(&buffer)
}

fn read_u32_from_stream(stream: &mut std::os::unix::net::UnixStream) -> std::io::Result<u32> {
    let mut buffer = [0u8; 4];
    stream.read_exact(&mut buffer)?;
    Ok(u32::from_be_bytes(buffer))
}

fn write_u32_to_stream(
    stream: &mut std::os::unix::net::UnixStream,
    value: u32,
) -> std::io::Result<()> {
    stream.write_all(&value.to_be_bytes())
}

fn write_ssh_string(buffer: &mut Vec<u8>, bytes: &[u8]) {
    buffer.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buffer.extend_from_slice(bytes);
}
