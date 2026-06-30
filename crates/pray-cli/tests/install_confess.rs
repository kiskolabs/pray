#[path = "install_network_support.rs"]
mod support;

use pray_core::auth::RegistryAuthStore;
use pray_core::lockfile::read_lockfile;
use serde_json::Value;
use std::fs;

use support::{
    create_add_fixture, find_free_port, run_pray, signing_key_from_seed, spawn_server,
    ssh_public_key_text, temporary_directory, verify_email_registration, wait_for_server,
    write_private_key_file,
};

#[test]
fn confess_recovers_after_server_restart_and_persists_submission() {
    let workspace = temporary_directory("pray-confess-recovery");
    let source_repo = workspace.join("source");
    let registry_root = workspace.join("registry");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&registry_root).expect("registry workspace");
    fs::create_dir_all(registry_root.join("v1")).expect("registry v1 workspace");
    fs::write(
        registry_root.join("v1/index.json"),
        r#"{
            "spec": "prayfile-distribution-1",
            "packages": []
        }"#,
    )
    .expect("write registry index");
    fs::write(
        registry_root.join("v1/trust.json"),
        r#"{
            "email_confirmation": "disabled",
            "passkeys_enabled": false,
            "ssh_keys_enabled": false,
            "ssh_agent_signing_enabled": false
        }"#,
    )
    .expect("write trust settings");

    create_add_fixture(&source_repo);
    let add = run_pray(
        &source_repo,
        &["add", "sample/base", "--path", "packages/base"],
    );
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    let publish = run_pray(
        &source_repo,
        &[
            "publish",
            "--root",
            registry_root.to_str().expect("registry path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    let port = find_free_port();
    let server_url = format!("http://127.0.0.1:{port}");
    let manifest_path = source_repo.join("Prayfile");
    fs::write(
        &manifest_path,
        format!(
            r#"
prayfile "1"
source "registry", "{server_url}"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", source: "registry"
render mode: :managed, conflict: :fail, churn: :minimal
"#,
        ),
    )
    .expect("manifest with source");

    let initial_confession_path = registry_root.join("v1/confessions.jsonl");
    assert!(!initial_confession_path.exists());

    let failed_confess = run_pray(
        &source_repo,
        &[
            "confess",
            "sample/base",
            "--version",
            "1.4.3",
            "--accepted",
            "--note",
            "server down",
        ],
    );
    assert!(!failed_confess.status.success());
    assert!(matches!(failed_confess.status.code(), Some(1) | Some(3)));
    let failed_stderr = String::from_utf8_lossy(&failed_confess.stderr);
    assert!(
        failed_stderr.contains("Connection refused")
            || failed_stderr.contains("timed out")
            || failed_stderr.contains("Network error")
            || failed_stderr.contains("resolution error")
    );
    assert!(!initial_confession_path.exists());

    let mut server = spawn_server(&registry_root, port);
    wait_for_server(port);

    let recovered_confess = run_pray(
        &source_repo,
        &[
            "confess",
            "sample/base",
            "--version",
            "1.4.3",
            "--accepted",
            "--note",
            "server back",
        ],
    );
    assert!(
        recovered_confess.status.success(),
        "confess failed after restart: {}",
        String::from_utf8_lossy(&recovered_confess.stderr)
    );

    let confession_text = fs::read_to_string(&initial_confession_path).expect("confession log");
    let confession_line = confession_text.lines().next().expect("confession line");
    let confession: Value = serde_json::from_str(confession_line).expect("confession json");
    assert_eq!(confession["package"], "sample/base");
    assert_eq!(confession["version"], "1.4.3");
    assert_eq!(confession["status"], "accepted");
    assert_eq!(confession["note"], "server back");

    let install = run_pray(&source_repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed before from-lock confession: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let lockfile = read_lockfile(&source_repo.join("Prayfile.lock")).expect("lockfile");
    let span_id = lockfile
        .managed_span
        .first()
        .expect("managed span in lockfile")
        .id
        .clone();

    let from_lock_confess = run_pray(
        &source_repo,
        &[
            "confess",
            "--from-lock",
            &span_id,
            "--accepted",
            "--note",
            "from lock",
        ],
    );
    assert!(
        from_lock_confess.status.success(),
        "from-lock confess failed: {}",
        String::from_utf8_lossy(&from_lock_confess.stderr)
    );

    let confession_text = fs::read_to_string(&initial_confession_path).expect("confession log");
    let confession_line = confession_text.lines().last().expect("confession line");
    let confession: Value = serde_json::from_str(confession_line).expect("confession json");
    assert_eq!(confession["package"], "sample/base");
    assert_eq!(confession["version"], "1.4.3");
    assert_eq!(confession["status"], "accepted");
    assert_eq!(confession["note"], "from lock");

    let _ = server.kill();
    let _ = server.wait();
}
