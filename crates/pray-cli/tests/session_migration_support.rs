use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::SigningKey;
use pray_core::auth::RegistryAuthStore;
use pray_core::trust::EmailConfirmationPolicy;
use serde_json::Value;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::Duration;

pub fn run_pray(repo: &std::path::Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pray"))
        .args(arguments)
        .current_dir(repo)
        .output()
        .expect("run pray command")
}

pub fn create_publish_fixture(repo: &std::path::Path) {
    fs::create_dir_all(repo.join("packages/base/exports")).expect("package directories");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", path: "packages/base"
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");
    fs::write(
        repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
end
"#,
    )
    .expect("write prayspec");
    fs::write(repo.join("packages/base/README.md"), "package readme\n").expect("write readme");
    fs::write(
        repo.join("packages/base/exports/testing-basics.md"),
        "Testing guidance\n",
    )
    .expect("write export");
}

pub fn published_metadata(root: &std::path::Path) -> Value {
    let text = fs::read_to_string(root.join("v1/packages/sample/base.json")).expect("metadata");
    serde_json::from_str(&text).expect("metadata json")
}

pub fn write_auth_registry_fixture(root: &std::path::Path) {
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

pub fn verify_email_registration(store: &RegistryAuthStore, email: &str) {
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

pub fn spawn_server(root: &std::path::Path, port: u16) -> std::process::Child {
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

pub fn write_private_key_file(
    repo: &std::path::Path,
    filename: &str,
    signing_key: &SigningKey,
) -> std::path::PathBuf {
    let path = repo.join(filename);
    fs::write(&path, signing_key.to_bytes()).expect("write private key file");
    path
}

pub fn ssh_public_key_text(signing_key: &SigningKey) -> String {
    let mut blob = Vec::new();
    write_ssh_string(&mut blob, b"ssh-ed25519");
    write_ssh_string(&mut blob, &signing_key.verifying_key().to_bytes());
    format!("ssh-ed25519 {}", STANDARD.encode(blob))
}

pub fn signing_key_from_seed(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

pub fn session_emails(session_json: &Value) -> Vec<String> {
    if let Some(sessions) = session_json.get("sessions").and_then(Value::as_array) {
        return sessions
            .iter()
            .filter_map(|session| {
                session
                    .get("email")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
            .collect();
    }

    session_json
        .get("email")
        .and_then(Value::as_str)
        .map(|email| vec![email.to_string()])
        .unwrap_or_default()
}

pub fn temporary_directory(prefix: &str) -> std::path::PathBuf {
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

pub fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve port");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);
    port
}

pub fn wait_for_server(port: u16) {
    for _ in 0..50 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("server did not start on port {port}");
}

fn write_ssh_string(buffer: &mut Vec<u8>, bytes: &[u8]) {
    buffer.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buffer.extend_from_slice(bytes);
}
