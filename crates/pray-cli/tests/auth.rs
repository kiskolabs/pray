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

    let register = fetch_http_post(
        &format!("http://127.0.0.1:{port}/v1/auth/register"),
        r#"{"email":"alice@example.com"}"#,
    );
    assert_eq!(register.status, 201);
    assert!(register.body.contains("\"verified\":false"));
    let code = extract_json_string(&register.body, "verification_code");

    let verify = fetch_http_post(
        &format!("http://127.0.0.1:{port}/v1/auth/verify"),
        &format!(r#"{{"email":"alice@example.com","code":"{}"}}"#, code),
    );
    let _ = server.kill();
    let _ = server.wait();

    assert_eq!(verify.status, 200);
    assert!(verify.body.contains("\"verified\":true"));
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
