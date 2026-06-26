use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[test]
fn exercises_registration_session_passkey_and_ssh_key_over_http() {
    let workspace = temporary_directory("pray-auth-http");
    let registry_root = workspace.join("registry");
    fs::create_dir_all(registry_root.join("v1")).expect("registry dirs");
    fs::write(
        registry_root.join("v1/index.json"),
        r#"{
            "spec": "prayfile-distribution-1",
            "packages": []
        }"#,
    )
    .expect("write index");
    fs::write(
        registry_root.join("v1/trust.json"),
        r#"{
            "email_confirmation": "required",
            "passkeys_enabled": true,
            "ssh_keys_enabled": true,
            "ssh_agent_signing_enabled": true
        }"#,
    )
    .expect("write trust settings");

    let port = find_free_port();
    let mut server = Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "serve",
            "--root",
            registry_root.to_str().expect("registry path"),
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn server");
    wait_for_server(port);

    let base_url = format!("http://127.0.0.1:{port}");

    let register = fetch_http_post(
        &format!("{base_url}/v1/auth/register"),
        r#"{"email":"alice@example.com"}"#,
    );
    assert_eq!(register.status, 201);
    assert!(register.body.contains("\"verified\":false"));
    let code = extract_json_string(&register.body, "verification_code");

    let verify = fetch_http_post(
        &format!("{base_url}/v1/auth/verify"),
        &format!(r#"{{"email":"alice@example.com","code":"{}"}}"#, code),
    );
    assert_eq!(verify.status, 200);
    assert!(verify.body.contains("\"verified\":true"));

    let session = fetch_http_post(
        &format!("{base_url}/v1/auth/session"),
        r#"{"email":"alice@example.com"}"#,
    );
    assert_eq!(session.status, 200);
    let session_token = extract_json_string(&session.body, "token");
    assert_eq!(
        extract_json_string(&session.body, "email"),
        "alice@example.com"
    );
    assert_eq!(extract_json_string(&session.body, "kind"), "email");
    assert!(session_token.starts_with("sha256:"));

    let passkey_enroll = fetch_http_post(
        &format!("{base_url}/v1/auth/passkeys/enroll"),
        r#"{"email":"alice@example.com","credential_id":"credential-1","public_key":"ed25519-public-key","label":"laptop passkey"}"#,
    );
    assert_eq!(passkey_enroll.status, 201);
    assert!(extract_json_bool(&passkey_enroll.body, "enrolled"));

    let passkey_login = fetch_http_post(
        &format!("{base_url}/v1/auth/passkeys/login"),
        r#"{"credential_id":"credential-1"}"#,
    );
    assert_eq!(passkey_login.status, 200);
    assert_eq!(
        extract_json_string(&passkey_login.body, "email"),
        "alice@example.com"
    );
    assert!(extract_json_string(&passkey_login.body, "token").starts_with("sha256:"));

    let ssh_enroll = fetch_http_post(
        &format!("{base_url}/v1/auth/ssh-keys/enroll"),
        r#"{"email":"alice@example.com","public_key":"ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIexample","label":"workstation"}"#,
    );
    assert_eq!(ssh_enroll.status, 201);
    assert!(extract_json_bool(&ssh_enroll.body, "enrolled"));

    let ssh_login = fetch_http_post(
        &format!("{base_url}/v1/auth/ssh-keys/login"),
        r#"{"public_key":"ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIexample"}"#,
    );
    assert_eq!(ssh_login.status, 200);
    assert_eq!(
        extract_json_string(&ssh_login.body, "email"),
        "alice@example.com"
    );
    assert!(extract_json_string(&ssh_login.body, "token").starts_with("sha256:"));

    let _ = server.kill();
    let _ = server.wait();
}

struct HttpResponse {
    status: u16,
    body: String,
}

fn fetch_http_post(url: &str, body: &str) -> HttpResponse {
    let url = url.strip_prefix("http://").expect("http url");
    let (host_port, path) = url.split_once('/').unwrap_or((url, ""));
    let (host, port) = host_port.split_once(':').expect("host and port");
    let mut stream =
        TcpStream::connect((host, port.parse::<u16>().expect("port"))).expect("connect");
    let request_path = format!("/{}", path);
    write!(
        stream,
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        request_path,
        host_port,
        body.len(),
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

fn extract_json_string(text: &str, key: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(text).expect("json body");
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .expect("json string")
        .to_string()
}

fn extract_json_bool(text: &str, key: &str) -> bool {
    let value: serde_json::Value = serde_json::from_str(text).expect("json body");
    value
        .get(key)
        .and_then(serde_json::Value::as_bool)
        .expect("json bool")
}

fn temporary_directory(prefix: &str) -> std::path::PathBuf {
    let unique = format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    );
    let path = std::env::temp_dir().join(unique);
    fs::create_dir_all(&path).expect("temporary directory");
    path
}

fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve port");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);
    port
}

fn wait_for_server(port: u16) {
    for _ in 0..50 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("server did not start on port {port}");
}
