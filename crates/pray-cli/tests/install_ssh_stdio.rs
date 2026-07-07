#[path = "install_network_support.rs"]
mod support;

use std::env;
use std::fs;
use std::sync::Mutex;
use support::{create_add_fixture, run_pray, temporary_directory};

static STDIO_SSH_ENV_LOCK: Mutex<()> = Mutex::new(());

struct StdioSshTestEnv {
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl StdioSshTestEnv {
    fn activate(distribution_root: &std::path::Path) -> Self {
        let lock = STDIO_SSH_ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        env::set_var(
            "PRAY_TEST_SSH_STDIO_ROOT",
            distribution_root.to_str().expect("distribution path"),
        );
        env::set_var("PRAY_TEST_BINARY", env!("CARGO_BIN_EXE_pray"));
        Self { _lock: lock }
    }
}

impl Drop for StdioSshTestEnv {
    fn drop(&mut self) {
        env::remove_var("PRAY_TEST_SSH_STDIO_ROOT");
        env::remove_var("PRAY_TEST_BINARY");
        env::remove_var("PRAY_SSH_PUBLISHER");
        env::remove_var("PRAY_SSH_USER_FINGERPRINT");
    }
}

fn publish_fixture_to_root(source_repo: &std::path::Path, distribution_root: &std::path::Path) {
    create_add_fixture(source_repo);
    let add = run_pray(
        source_repo,
        &["add", "sample/base", "--path", "packages/base"],
    );
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let publish = run_pray(
        source_repo,
        &[
            "publish",
            "--root",
            distribution_root.to_str().expect("distribution path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );
}

fn write_ssh_publishers_policy(distribution_root: &std::path::Path) {
    fs::create_dir_all(distribution_root.join("v1")).expect("v1 directory");
    fs::write(
        distribution_root.join("v1/ssh_publishers.json"),
        r#"{"publishers":[{"fingerprint":"SHA256:abc","id":"team-ci","push":true}]}"#,
    )
    .expect("ssh publishers policy");
}

#[test]
fn install_can_resolve_packages_from_pray_ssh_stdio_source() {
    let workspace = temporary_directory("pray-install-ssh-source");
    let source_repo = workspace.join("source");
    let distribution_root = workspace.join("distribution");
    let consumer_repo = workspace.join("consumer");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&distribution_root).expect("distribution workspace");
    fs::create_dir_all(&consumer_repo).expect("consumer workspace");

    publish_fixture_to_root(&source_repo, &distribution_root);

    fs::write(
        consumer_repo.join("Prayfile"),
        r#"prayfile "1"
source "team", "pray+ssh://pray@stdio-host"
agent "sample/base", "~> 1.4", source: :team
"#,
    )
    .expect("consumer prayfile");

    let _env = StdioSshTestEnv::activate(&distribution_root);
    let install = run_pray(&consumer_repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );
    let lockfile = fs::read_to_string(consumer_repo.join("Prayfile.lock")).expect("lockfile");
    assert!(lockfile.contains("pray_ssh"));
    assert!(lockfile.contains("pray+ssh://pray@stdio-host"));
    assert!(consumer_repo.join(".pray/cache").exists());
}

#[test]
fn publish_to_pray_ssh_stdio_server_round_trips_install() {
    let workspace = temporary_directory("pray-publish-ssh-stdio");
    let source_repo = workspace.join("source");
    let remote_root = workspace.join("remote");
    let consumer_repo = workspace.join("consumer");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&remote_root).expect("remote workspace");
    fs::create_dir_all(&consumer_repo).expect("consumer workspace");

    publish_fixture_to_root(&source_repo, &source_repo.join("local-dist"));

    let _env = StdioSshTestEnv::activate(&remote_root);
    let publish = run_pray(
        &source_repo,
        &["publish", "--server", "pray+ssh://pray@stdio-host"],
    );
    assert!(
        publish.status.success(),
        "remote publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    fs::write(
        consumer_repo.join("Prayfile"),
        r#"prayfile "1"
source "team", "pray+ssh://pray@stdio-host"
agent "sample/base", "~> 1.4", source: :team
"#,
    )
    .expect("consumer prayfile");

    let install = run_pray(&consumer_repo, &["install"]);
    assert!(
        install.status.success(),
        "install after remote publish failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );
    assert!(remote_root.join("v1/packages/sample/base.json").is_file());
}

#[test]
fn stdio_publish_requires_pray_ssh_publisher_when_policy_configured() {
    let workspace = temporary_directory("pray-ssh-publish-auth");
    let source_repo = workspace.join("source");
    let remote_root = workspace.join("remote");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&remote_root).expect("remote workspace");

    publish_fixture_to_root(&source_repo, &source_repo.join("local-dist"));
    write_ssh_publishers_policy(&remote_root);

    let _env = StdioSshTestEnv::activate(&remote_root);
    let denied = run_pray(
        &source_repo,
        &["publish", "--server", "pray+ssh://pray@stdio-host"],
    );
    assert!(
        !denied.status.success(),
        "publish should fail without SSH user fingerprint: {}",
        String::from_utf8_lossy(&denied.stderr)
    );
    let denied_message = format!(
        "{}{}",
        String::from_utf8_lossy(&denied.stderr),
        String::from_utf8_lossy(&denied.stdout)
    );
    assert!(denied_message.contains("SSH user fingerprint"));

    env::set_var("PRAY_SSH_USER_FINGERPRINT", "SHA256:abc");
    let allowed = run_pray(
        &source_repo,
        &["publish", "--server", "pray+ssh://pray@stdio-host"],
    );
    assert!(
        allowed.status.success(),
        "authorized publish failed: {}",
        String::from_utf8_lossy(&allowed.stderr)
    );
    assert!(remote_root.join("v1/packages/sample/base.json").is_file());
}

#[test]
fn sync_pulls_packages_from_pray_ssh_stdio_peer() {
    let workspace = temporary_directory("pray-sync-ssh-stdio");
    let source_repo = workspace.join("source");
    let remote_root = workspace.join("remote");
    let mirror_root = workspace.join("mirror");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&remote_root).expect("remote workspace");
    fs::create_dir_all(&mirror_root).expect("mirror workspace");

    publish_fixture_to_root(&source_repo, &source_repo.join("local-dist"));

    let _env = StdioSshTestEnv::activate(&remote_root);
    let publish = run_pray(
        &source_repo,
        &["publish", "--server", "pray+ssh://pray@stdio-host"],
    );
    assert!(
        publish.status.success(),
        "remote publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    let sync = run_pray(
        &mirror_root,
        &[
            "sync",
            "--root",
            mirror_root.to_str().expect("mirror path"),
            "--peer",
            "pray+ssh://pray@stdio-host",
        ],
    );
    assert!(
        sync.status.success(),
        "sync failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );
    assert!(mirror_root.join("v1/packages/sample/base.json").is_file());
}

#[test]
fn stdio_server_answers_sync_package_rpc() {
    use pray_core::ssh_rpc::{call_stdio, RpcRequest};
    use serde_json::json;
    use std::io::BufReader;
    use std::process::{Command, Stdio};

    let workspace = temporary_directory("pray-ssh-rpc");
    let source_repo = workspace.join("source");
    let distribution_root = workspace.join("distribution");
    fs::create_dir_all(&source_repo).expect("source workspace");

    publish_fixture_to_root(&source_repo, &distribution_root);

    let _env = StdioSshTestEnv::activate(&distribution_root);

    let mut server = Command::new(env!("CARGO_BIN_EXE_pray"))
        .arg("serve")
        .arg("--stdio")
        .arg("--root")
        .arg(&distribution_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn stdio server");

    let request = RpcRequest::new("1", "sync.package", json!({ "name": "sample/base" }));
    let stdin = server.stdin.as_mut().expect("server stdin");
    let stdout = server.stdout.as_mut().expect("server stdout");
    let mut reader = BufReader::new(stdout);
    let response = call_stdio(&mut reader, stdin, &request).expect("rpc response");
    assert_eq!(response.status, 200);
    assert_eq!(
        response.body.get("name").and_then(|value| value.as_str()),
        Some("sample/base")
    );
}
