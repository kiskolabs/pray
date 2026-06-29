use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::SigningKey;
use pray_core::auth::RegistryAuthStore;
use pray_core::trust::EmailConfirmationPolicy;
use serde_json::Value;
use std::fs;

use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[test]
fn login_recovers_after_auth_server_restart_and_persists_session() {
    let workspace = temporary_directory("pray-login-recovery");
    let auth_root = workspace.join("auth");
    let client_repo = workspace.join("client");
    fs::create_dir_all(&auth_root).expect("auth workspace");
    fs::create_dir_all(&client_repo).expect("client workspace");
    write_auth_registry_fixture(&auth_root);

    let store = RegistryAuthStore::open(&auth_root).expect("open auth store");
    let email = "login-recovery@example.com";
    let signing_key = signing_key_from_seed(31);
    let public_key = ssh_public_key_text(&signing_key);

    verify_email_registration(&store, email);
    store
        .enroll_passkey(email, "login-recovery-passkey", &public_key, Some("laptop"))
        .expect("enroll passkey");

    let private_key_path =
        write_private_key_file(&client_repo, "login-recovery-passkey.bin", &signing_key);
    let port = find_free_port();
    let server_url = format!("http://127.0.0.1:{port}");
    let session_path = client_repo.join(".pray/session.json");

    let failed_login = run_pray(
        &client_repo,
        &[
            "login",
            "--server",
            &server_url,
            "--email",
            email,
            "--credential-id",
            "login-recovery-passkey",
            "--passkey-key",
            private_key_path.to_str().expect("private key path"),
        ],
    );
    assert!(!failed_login.status.success());
    assert!(matches!(failed_login.status.code(), Some(1) | Some(3)));
    let failed_stderr = String::from_utf8_lossy(&failed_login.stderr);
    assert!(
        failed_stderr.contains("Connection refused")
            || failed_stderr.contains("timed out")
            || failed_stderr.contains("Network error")
    );
    assert!(!session_path.exists());

    let mut server = spawn_server(&auth_root, port);
    wait_for_server(port);

    let recovered_login = run_pray(
        &client_repo,
        &[
            "login",
            "--server",
            &server_url,
            "--email",
            email,
            "--credential-id",
            "login-recovery-passkey",
            "--passkey-key",
            private_key_path.to_str().expect("private key path"),
        ],
    );
    assert!(
        recovered_login.status.success(),
        "login failed after restart: {}",
        String::from_utf8_lossy(&recovered_login.stderr)
    );

    let session_text = fs::read_to_string(&session_path).expect("session file");
    let session_json: Value = serde_json::from_str(&session_text).expect("session json");
    assert_eq!(session_json["email"], email);
    assert_eq!(session_json["server_url"], server_url);
    assert_eq!(session_json["kind"], "passkey");
    assert!(session_json["token"]
        .as_str()
        .expect("session token")
        .starts_with("sha256:"));

    let _ = server.kill();
    let _ = server.wait();
}

#[test]
fn login_supports_multiple_auth_origins() {
    let workspace = temporary_directory("pray-login-multi-origin");
    let client_repo = workspace.join("client");
    let auth_root_a = workspace.join("auth-a");
    let auth_root_b = workspace.join("auth-b");
    fs::create_dir_all(&client_repo).expect("client workspace");
    fs::create_dir_all(&auth_root_a).expect("auth root A workspace");
    fs::create_dir_all(&auth_root_b).expect("auth root B workspace");
    write_auth_registry_fixture(&auth_root_a);
    write_auth_registry_fixture(&auth_root_b);

    let auth_store_a = RegistryAuthStore::open(&auth_root_a).expect("open auth store A");
    let auth_store_b = RegistryAuthStore::open(&auth_root_b).expect("open auth store B");
    let login_email = "multi-origin@example.com";
    let signing_key = signing_key_from_seed(21);
    let public_key = ssh_public_key_text(&signing_key);

    verify_email_registration(&auth_store_a, login_email);
    verify_email_registration(&auth_store_b, login_email);
    auth_store_a
        .enroll_passkey(
            login_email,
            "multi-origin-passkey",
            &public_key,
            Some("auth origin A"),
        )
        .expect("enroll passkey on auth origin A");
    auth_store_b
        .enroll_passkey(
            login_email,
            "multi-origin-passkey",
            &public_key,
            Some("auth origin B"),
        )
        .expect("enroll passkey on auth origin B");

    let port_a = find_free_port();
    let port_b = find_free_port();
    let mut server_a = spawn_server(&auth_root_a, port_a);
    let mut server_b = spawn_server(&auth_root_b, port_b);
    wait_for_server(port_a);
    wait_for_server(port_b);

    let server_url_a = format!("http://127.0.0.1:{port_a}");
    let server_url_b = format!("http://127.0.0.1:{port_b}");
    let private_key_path =
        write_private_key_file(&client_repo, "multi-origin-passkey.bin", &signing_key);

    let login = Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "login",
            "--server",
            &server_url_a,
            "--server",
            &server_url_b,
            "--email",
            login_email,
            "--credential-id",
            "multi-origin-passkey",
            "--passkey-key",
            private_key_path.to_str().expect("private key path"),
        ])
        .current_dir(&client_repo)
        .output()
        .expect("run multi-origin login");
    assert!(
        login.status.success(),
        "multi-origin login failed: {}",
        String::from_utf8_lossy(&login.stderr)
    );

    let session_text =
        fs::read_to_string(client_repo.join(".pray/session.json")).expect("session file");
    let session_json: Value = serde_json::from_str(&session_text).expect("session json");
    let server_urls = session_server_urls(&session_json);
    assert!(server_urls.contains(&server_url_a));
    assert!(server_urls.contains(&server_url_b));

    let _ = server_a.kill();
    let _ = server_a.wait();
    let _ = server_b.kill();
    let _ = server_b.wait();
}

fn run_pray(repo: &std::path::Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_pray"))
        .args(arguments)
        .current_dir(repo)
        .output()
        .expect("run pray command")
}

fn write_auth_registry_fixture(root: &std::path::Path) {
    fs::create_dir_all(root.join("v1")).expect("auth root directories");
    fs::write(
        root.join("v1/index.json"),
        r#"{
            "spec": "prayfile-distribution-1",
            "packages": []
        }"#,
    )
    .expect("write auth index");
    fs::write(
        root.join("v1/trust.json"),
        r#"{
            "email_confirmation": "required",
            "passkeys_enabled": true,
            "ssh_keys_enabled": true,
            "ssh_agent_signing_enabled": true
        }"#,
    )
    .expect("write auth trust settings");
}

fn verify_email_registration(store: &RegistryAuthStore, email: &str) {
    let registration = store
        .register_email(email, EmailConfirmationPolicy::Required)
        .expect("register email");
    let verification_code = registration
        .verification_code
        .as_deref()
        .expect("verification code");
    store
        .verify_email(email, verification_code)
        .expect("verify email");
}

fn spawn_server(root: &std::path::Path, port: u16) -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "serve",
            "--root",
            root.to_str().expect("auth root path"),
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn server")
}

fn write_private_key_file(
    repo: &std::path::Path,
    filename: &str,
    signing_key: &SigningKey,
) -> std::path::PathBuf {
    let path = repo.join(filename);
    fs::write(&path, signing_key.to_bytes()).expect("write private key file");
    path
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

fn session_server_urls(session_json: &Value) -> Vec<String> {
    if let Some(sessions) = session_json.get("sessions").and_then(Value::as_array) {
        return sessions
            .iter()
            .filter_map(|session| {
                session
                    .get("server_url")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
            .collect();
    }

    session_json
        .get("server_url")
        .and_then(Value::as_str)
        .map(|server_url| vec![server_url.to_string()])
        .unwrap_or_default()
}
