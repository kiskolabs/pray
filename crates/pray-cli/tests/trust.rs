use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[test]
fn serves_registry_trust_policy_on_the_root_page() {
    let workspace = temporary_directory("pray-trust");
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
            "email_confirmation": "optional",
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

    let root_page = fetch_http(&format!("http://127.0.0.1:{port}/"));
    let _ = server.kill();
    let _ = server.wait();

    assert!(root_page.contains("Email confirmation: optional"));
    assert!(root_page.contains("Passkeys: enabled"));
    assert!(root_page.contains("SSH keys: enabled"));
    assert!(root_page.contains("SSH-agent signing: enabled"));
}

#[test]
fn checks_an_http_compromised_key_feed() {
    let workspace = temporary_directory("pray-trust-check");
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind feed server");
    let port = listener.local_addr().expect("feed server address").port();
    let feed_body = "[[keys]]\nvalue = \"SHA256:COMPROMISED\"\n";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        feed_body.len(),
        feed_body
    );
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept feed request");
        let mut request = [0_u8; 1024];
        stream.read(&mut request).expect("read feed request");
        stream
            .write_all(response.as_bytes())
            .expect("write feed response");
    });

    let output = Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "trust",
            "check",
            &format!("http://127.0.0.1:{port}/compromised-keys.toml"),
        ])
        .env("PRAY_HOME", &workspace)
        .env("PRAY_NO_VERSION_CHECK", "1")
        .output()
        .expect("run trust check");
    server.join().expect("feed server completes");

    assert!(
        output.status.success(),
        "trust check failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("no compromised trusted signing keys detected"));
}

fn temporary_directory(prefix: &str) -> PathBuf {
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

fn fetch_http(url: &str) -> String {
    let url = url.strip_prefix("http://").expect("http url");
    let (host_port, path) = url.split_once('/').unwrap_or((url, ""));
    let (host, port) = host_port.split_once(':').expect("host and port");
    let mut stream =
        TcpStream::connect((host, port.parse::<u16>().expect("port"))).expect("connect");
    let request_path = format!("/{}", path);
    write!(
        stream,
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        request_path, host_port
    )
    .expect("write request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read response");
    response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body.to_string())
        .unwrap_or(response)
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
