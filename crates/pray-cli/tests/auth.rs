use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use pray_core::auth::RegistryAuthStore;
use pray_core::trust::EmailConfirmationPolicy;
use pray_core::PrayError;
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
    let signing_key = signing_key_from_seed(7);
    let public_key = ssh_public_key_text(&signing_key);

    let store = RegistryAuthStore::open(&registry_root).expect("open auth store");
    let register = store
        .register_email("alice@example.com", EmailConfirmationPolicy::Required)
        .expect("register");
    assert!(!register.verified);
    let code = register.verification_code.expect("verification code");
    assert_eq!(code.len(), 6);

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

    let passkey_enroll = store
        .enroll_passkey(
            "alice@example.com",
            "credential-1",
            &public_key,
            Some("laptop passkey"),
        )
        .expect("passkey enrollment");
    assert!(passkey_enroll.enrolled);

    let passkey_challenge = fetch_http_post(
        &format!("{base_url}/v1/auth/passkeys/challenge"),
        r#"{"credential_id":"credential-1"}"#,
    );
    assert_eq!(passkey_challenge.status, 200);
    let passkey_challenge_id = extract_json_string(&passkey_challenge.body, "challenge_id");
    let passkey_challenge_value = extract_json_string(&passkey_challenge.body, "challenge");
    let passkey_signature = STANDARD.encode(
        signing_key
            .sign(passkey_challenge_value.as_bytes())
            .to_bytes(),
    );
    let passkey_login = fetch_http_post(
        &format!("{base_url}/v1/auth/passkeys/login"),
        &format!(
            r#"{{"credential_id":"credential-1","challenge_id":"{}","signature":"{}"}}"#,
            passkey_challenge_id, passkey_signature
        ),
    );
    assert_eq!(passkey_login.status, 200);
    assert_eq!(
        extract_json_string(&passkey_login.body, "email"),
        "alice@example.com"
    );
    assert!(extract_json_string(&passkey_login.body, "token").starts_with("sha256:"));

    let ssh_enroll = store
        .enroll_ssh_key("alice@example.com", &public_key, Some("workstation"))
        .expect("ssh enrollment");
    assert!(ssh_enroll.enrolled);

    let ssh_challenge = fetch_http_post(
        &format!("{base_url}/v1/auth/ssh-keys/challenge"),
        &format!(r#"{{"public_key":"{}"}}"#, public_key),
    );
    assert_eq!(ssh_challenge.status, 200);
    let ssh_challenge_id = extract_json_string(&ssh_challenge.body, "challenge_id");
    let ssh_challenge_value = extract_json_string(&ssh_challenge.body, "challenge");
    let ssh_signature =
        STANDARD.encode(signing_key.sign(ssh_challenge_value.as_bytes()).to_bytes());
    let ssh_login = fetch_http_post(
        &format!("{base_url}/v1/auth/ssh-keys/login"),
        &format!(
            r#"{{"public_key":"{}","challenge_id":"{}","signature":"{}"}}"#,
            public_key, ssh_challenge_id, ssh_signature
        ),
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

#[test]
fn rejects_invalid_passkey_and_ssh_signatures() {
    let workspace = temporary_directory("pray-auth-invalid-signature");
    let store = RegistryAuthStore::open(&workspace).expect("open auth store");
    let signing_key = signing_key_from_seed(7);
    let wrong_key = signing_key_from_seed(8);
    let public_key = ssh_public_key_text(&signing_key);

    let registration = store
        .register_email("alice@example.com", EmailConfirmationPolicy::Disabled)
        .expect("register");
    assert!(registration.verified);

    store
        .enroll_passkey(
            "alice@example.com",
            "credential-1",
            &public_key,
            Some("laptop passkey"),
        )
        .expect("passkey enrollment");
    store
        .enroll_ssh_key("alice@example.com", &public_key, Some("workstation"))
        .expect("ssh enrollment");

    let passkey_challenge = store
        .request_passkey_challenge("credential-1")
        .expect("passkey challenge");
    let invalid_passkey_signature = STANDARD.encode(
        wrong_key
            .sign(passkey_challenge.challenge.as_bytes())
            .to_bytes(),
    );
    let passkey_error = store
        .respond_passkey_challenge(
            "credential-1",
            &passkey_challenge.challenge_id,
            &invalid_passkey_signature,
        )
        .expect_err("invalid passkey signature should fail");
    assert!(matches!(passkey_error, PrayError::Verify(_)));

    let ssh_challenge = store
        .request_ssh_key_challenge(&public_key)
        .expect("ssh challenge");
    let invalid_ssh_signature = STANDARD.encode(
        wrong_key
            .sign(ssh_challenge.challenge.as_bytes())
            .to_bytes(),
    );
    let ssh_error = store
        .respond_ssh_key_challenge(
            &public_key,
            &ssh_challenge.challenge_id,
            &invalid_ssh_signature,
        )
        .expect_err("invalid ssh signature should fail");
    assert!(matches!(ssh_error, PrayError::Verify(_)));
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

fn ssh_public_key_text(signing_key: &SigningKey) -> String {
    let mut blob = Vec::new();
    write_ssh_string(&mut blob, b"ssh-ed25519");
    write_ssh_string(&mut blob, &signing_key.verifying_key().to_bytes());
    format!("ssh-ed25519 {}", STANDARD.encode(blob))
}

fn write_ssh_string(buffer: &mut Vec<u8>, bytes: &[u8]) {
    buffer.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buffer.extend_from_slice(bytes);
}

fn signing_key_from_seed(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
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
