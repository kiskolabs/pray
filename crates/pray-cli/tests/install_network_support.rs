#![allow(dead_code)]

use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::SigningKey;
use pray_core::auth::RegistryAuthStore;
use pray_core::trust::EmailConfirmationPolicy;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn run_pray(repo: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_pray"))
        .args(arguments)
        .current_dir(repo)
        .output()
        .expect("run pray command")
}

pub fn run_pray_login_passkey(
    repo: &Path,
    server_url: &str,
    email: &str,
    credential_id: &str,
    private_key_path: &Path,
) {
    let login = Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "login",
            "--server",
            server_url,
            "--email",
            email,
            "--credential-id",
            credential_id,
            "--passkey-key",
            private_key_path.to_str().expect("private key path"),
        ])
        .current_dir(repo)
        .output()
        .expect("run passkey login");
    assert!(
        login.status.success(),
        "passkey login failed: {}",
        String::from_utf8_lossy(&login.stderr)
    );
}

pub fn write_registry_client_fixture(repo: &Path, server_url: &str) {
    fs::create_dir_all(repo).expect("client repo");
    fs::write(
        repo.join("Prayfile"),
        format!(
            r#"
prayfile "1"
source "default", "{server_url}"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", "~> 1.4", source: "default"
render mode: :managed, conflict: :fail, churn: :minimal
"#,
        ),
    )
    .expect("write client Prayfile");
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

pub fn spawn_server(root: &Path, port: u16) -> Child {
    Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "serve",
            "--root",
            root.to_str().expect("registry path"),
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

pub fn write_private_key_file(repo: &Path, filename: &str, signing_key: &SigningKey) -> PathBuf {
    let path = repo.join(filename);
    fs::write(&path, signing_key.to_bytes()).expect("write private key file");
    path
}

pub fn signing_key_from_seed(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

pub fn ssh_public_key_text(signing_key: &SigningKey) -> String {
    let mut blob = Vec::new();
    write_ssh_string(&mut blob, b"ssh-ed25519");
    write_ssh_string(&mut blob, &signing_key.verifying_key().to_bytes());
    format!("ssh-ed25519 {}", STANDARD.encode(blob))
}

pub fn write_ssh_string(buffer: &mut Vec<u8>, bytes: &[u8]) {
    buffer.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buffer.extend_from_slice(bytes);
}

pub fn find_free_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .expect("bind port")
        .local_addr()
        .expect("local addr")
        .port()
}

pub fn wait_for_server(port: u16) {
    for _ in 0..50 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        sleep(Duration::from_millis(100));
    }
    panic!("server did not start on port {port}");
}

pub fn temporary_directory(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!("{prefix}-{stamp}-{suffix}"));
    fs::create_dir_all(&path).expect("temp dir");
    path
}

pub fn create_add_fixture(repo: &Path) {
    fs::create_dir_all(repo.join("packages/base/exports")).expect("fixture directories");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
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
