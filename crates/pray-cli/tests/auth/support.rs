use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::SigningKey;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

pub(crate) struct HttpResponse {
    pub(crate) status: u16,
    pub(crate) body: String,
}

pub(crate) fn fetch_http_post(url: &str, body: &str) -> HttpResponse {
    fetch_http_post_with_authorization(url, body, None)
}

pub(crate) fn fetch_http_post_with_authorization(
    url: &str,
    body: &str,
    authorization: Option<&str>,
) -> HttpResponse {
    let url = url.strip_prefix("http://").expect("http url");
    let (host_port, path) = url.split_once('/').unwrap_or((url, ""));
    let (host, port) = host_port.split_once(':').expect("host and port");
    let mut stream =
        TcpStream::connect((host, port.parse::<u16>().expect("port"))).expect("connect");
    let request_path = format!("/{}", path);
    let authorization_header = authorization
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    write!(
        stream,
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{}Connection: close\r\n\r\n{}",
        request_path,
        host_port,
        body.len(),
        authorization_header,
        body
    )
    .expect("write request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read response");
    let mut sections = response.splitn(2, "\r\n\r\n");
    let header = sections.next().unwrap_or_default();
    let body = sections.next().unwrap_or_default().to_string();
    let status = header
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .expect("status code");
    HttpResponse { status, body }
}

pub(crate) fn latest_delivery_code(root: &Path, email: &str) -> String {
    let text = fs::read_to_string(root.join(".pray/verification-deliveries.jsonl"))
        .expect("delivery file");
    text.lines()
        .rev()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("jsonl"))
        .find(|value| value["email"].as_str() == Some(email))
        .and_then(|value| value["code"].as_str().map(str::to_string))
        .expect("delivered code")
}

pub(crate) fn extract_json_string(text: &str, key: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(text).expect("json body");
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .expect("json string")
        .to_string()
}

pub(crate) fn ssh_public_key_text(signing_key: &SigningKey) -> String {
    let mut blob = Vec::new();
    write_ssh_string(&mut blob, b"ssh-ed25519");
    write_ssh_string(&mut blob, &signing_key.verifying_key().to_bytes());
    format!("ssh-ed25519 {}", STANDARD.encode(blob))
}

fn write_ssh_string(buffer: &mut Vec<u8>, bytes: &[u8]) {
    buffer.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buffer.extend_from_slice(bytes);
}

pub(crate) fn signing_key_from_seed(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}
