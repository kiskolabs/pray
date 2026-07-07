#[path = "install_network_support.rs"]
mod support;

use pray_core::auth::RegistryAuthStore;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

#[path = "install_distribution_point_support.rs"]
mod distribution_support;

use distribution_support::{run_pray_login_ssh_agent, spawn_mock_ssh_agent, write_public_key_file};
use support::{
    create_add_fixture, find_free_port, run_pray, run_pray_login_passkey, signing_key_from_seed,
    ssh_public_key_text, temporary_directory, verify_email_registration, wait_for_server,
    write_private_key_file, write_registry_client_fixture,
};

fn git_cache_repository(consumer_repo: &std::path::Path) -> PathBuf {
    let cache_root = consumer_repo.join(".pray/cache/git");
    let entry = fs::read_dir(&cache_root)
        .expect("cache entries")
        .next()
        .expect("git cache directory")
        .expect("cache entry");
    entry.path()
}

fn assert_success(output: &Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git(directory: &std::path::Path, arguments: &[&str]) -> Output {
    Command::new("git")
        .current_dir(directory)
        .args(arguments)
        .output()
        .expect("run git")
}

#[test]
fn install_can_resolve_packages_from_a_git_distribution_repo() {
    let workspace = temporary_directory("pray-install-git");
    let source_repo = workspace.join("source");
    let distribution_repo = workspace.join("distribution");
    let prayers_root = distribution_repo.join("prayers");
    let consumer_repo = workspace.join("consumer");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&distribution_repo).expect("distribution workspace");
    fs::create_dir_all(&consumer_repo).expect("consumer workspace");

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
            prayers_root.to_str().expect("distribution path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    assert_success(
        &git(&distribution_repo, &["init", "-b", "main"]),
        "git init",
    );
    assert_success(
        &git(&distribution_repo, &["config", "user.name", "Pray Test"]),
        "git user.name",
    );
    assert_success(
        &git(
            &distribution_repo,
            &["config", "user.email", "pray@example.com"],
        ),
        "git user.email",
    );
    assert_success(&git(&distribution_repo, &["add", "-A"]), "git add");
    assert_success(
        &git(
            &distribution_repo,
            &["commit", "-m", "initial distribution"],
        ),
        "git commit",
    );

    fs::write(
        consumer_repo.join("Prayfile"),
        format!(
            r#"
prayfile "1"
source "dist", "git+file://{distribution}"
agent "sample/base", "~> 1.4", source: "dist"
target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
"#,
            distribution = distribution_repo.display()
        ),
    )
    .expect("write consumer Prayfile");

    let install = run_pray(&consumer_repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let lockfile = fs::read_to_string(consumer_repo.join("Prayfile.lock")).expect("lockfile");
    assert!(lockfile.contains("sample/base"));
    assert!(
        lockfile.contains("revision ="),
        "git source revision should be recorded in lockfile:\n{lockfile}"
    );
    assert!(consumer_repo.join("INSTRUCTIONS.md").is_file());
}

#[test]
fn install_keeps_locked_git_revision_when_distribution_moves_forward() {
    let workspace = temporary_directory("pray-install-git-pin");
    let source_repo = workspace.join("source");
    let distribution_repo = workspace.join("distribution");
    let prayers_root = distribution_repo.join("prayers");
    let consumer_repo = workspace.join("consumer");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&distribution_repo).expect("distribution workspace");
    fs::create_dir_all(&consumer_repo).expect("consumer workspace");

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
            prayers_root.to_str().expect("distribution path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    assert_success(
        &git(&distribution_repo, &["init", "-b", "main"]),
        "git init",
    );
    assert_success(
        &git(&distribution_repo, &["config", "user.name", "Pray Test"]),
        "git user.name",
    );
    assert_success(
        &git(
            &distribution_repo,
            &["config", "user.email", "pray@example.com"],
        ),
        "git user.email",
    );
    assert_success(&git(&distribution_repo, &["add", "-A"]), "git add");
    assert_success(
        &git(
            &distribution_repo,
            &["commit", "-m", "initial distribution"],
        ),
        "git commit",
    );

    let initial_commit =
        String::from_utf8_lossy(&git(&distribution_repo, &["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();

    fs::write(
        consumer_repo.join("Prayfile"),
        format!(
            r#"
prayfile "1"
source "dist", "git+file://{distribution}"
agent "sample/base", "~> 1.4", source: "dist"
target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
"#,
            distribution = distribution_repo.display()
        ),
    )
    .expect("write consumer Prayfile");

    let install = run_pray(&consumer_repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let lockfile = fs::read_to_string(consumer_repo.join("Prayfile.lock")).expect("lockfile");
    assert!(
        lockfile.contains(&initial_commit),
        "lockfile should pin initial git revision:\n{lockfile}"
    );

    fs::write(
        distribution_repo.join("prayers/marker.txt"),
        "distribution moved forward",
    )
    .expect("write marker");
    assert_success(&git(&distribution_repo, &["add", "-A"]), "git add forward");
    assert_success(
        &git(
            &distribution_repo,
            &["commit", "-m", "forward distribution"],
        ),
        "git commit forward",
    );

    let locked_install = run_pray(&consumer_repo, &["install", "--locked"]);
    assert!(
        locked_install.status.success(),
        "locked install failed: {}",
        String::from_utf8_lossy(&locked_install.stderr)
    );

    let lockfile_after = fs::read_to_string(consumer_repo.join("Prayfile.lock")).expect("lockfile");
    assert!(
        lockfile_after.contains(&initial_commit),
        "locked install should keep the pinned revision:\n{lockfile_after}"
    );

    let cache_head = String::from_utf8_lossy(
        &git(
            &git_cache_repository(&consumer_repo),
            &["rev-parse", "HEAD"],
        )
        .stdout,
    )
    .trim()
    .to_string();
    assert_eq!(
        cache_head, initial_commit,
        "git cache should remain on the locked revision after distribution moved forward"
    );
}

#[test]
fn update_advances_git_source_and_republished_artifact_at_same_version() {
    let workspace = temporary_directory("pray-update-git-republish");
    let source_repo = workspace.join("source");
    let distribution_repo = workspace.join("distribution");
    let prayers_root = distribution_repo.join("prayers");
    let consumer_repo = workspace.join("consumer");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&distribution_repo).expect("distribution workspace");
    fs::create_dir_all(&consumer_repo).expect("consumer workspace");

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
            prayers_root.to_str().expect("distribution path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    assert_success(
        &git(&distribution_repo, &["init", "-b", "main"]),
        "git init",
    );
    assert_success(
        &git(&distribution_repo, &["config", "user.name", "Pray Test"]),
        "git user.name",
    );
    assert_success(
        &git(
            &distribution_repo,
            &["config", "user.email", "pray@example.com"],
        ),
        "git user.email",
    );
    assert_success(&git(&distribution_repo, &["add", "-A"]), "git add");
    assert_success(
        &git(
            &distribution_repo,
            &["commit", "-m", "initial distribution"],
        ),
        "git commit",
    );

    let initial_commit =
        String::from_utf8_lossy(&git(&distribution_repo, &["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();

    fs::write(
        consumer_repo.join("Prayfile"),
        format!(
            r#"
prayfile "1"
source "dist", "git+file://{distribution}"
agent "sample/base", "~> 1.4", source: "dist"
target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
"#,
            distribution = distribution_repo.display()
        ),
    )
    .expect("write consumer Prayfile");

    let install = run_pray(&consumer_repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let initial_lockfile =
        fs::read_to_string(consumer_repo.join("Prayfile.lock")).expect("lockfile");
    assert!(initial_lockfile.contains(&initial_commit));
    let initial_instructions =
        fs::read_to_string(consumer_repo.join("INSTRUCTIONS.md")).expect("instructions");
    assert!(initial_instructions.contains("Testing guidance"));

    fs::write(
        source_repo.join("packages/base/exports/testing-basics.md"),
        "Updated testing guidance after republish\n",
    )
    .expect("rewrite export");
    let republish = run_pray(
        &source_repo,
        &[
            "publish",
            "--root",
            prayers_root.to_str().expect("distribution path"),
        ],
    );
    assert!(
        republish.status.success(),
        "republish failed: {}",
        String::from_utf8_lossy(&republish.stderr)
    );
    assert_success(
        &git(&distribution_repo, &["add", "-A"]),
        "git add republish",
    );
    assert_success(
        &git(
            &distribution_repo,
            &["commit", "-m", "republish same version"],
        ),
        "git commit republish",
    );

    let updated_commit =
        String::from_utf8_lossy(&git(&distribution_repo, &["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();
    assert_ne!(initial_commit, updated_commit);

    let update = run_pray(&consumer_repo, &["update"]);
    assert!(
        update.status.success(),
        "update failed: {}",
        String::from_utf8_lossy(&update.stderr)
    );
    let stdout = String::from_utf8_lossy(&update.stdout);
    assert!(
        stdout.contains("Refreshed package sample/base"),
        "update summary should report republished content:\n{stdout}"
    );
    assert!(stdout.contains("Updated source dist revision"));

    let updated_lockfile =
        fs::read_to_string(consumer_repo.join("Prayfile.lock")).expect("lockfile");
    assert!(
        updated_lockfile.contains(&updated_commit),
        "lockfile should advance git source revision:\n{updated_lockfile}"
    );
    assert!(
        !updated_lockfile.contains(&initial_commit),
        "lockfile should not keep the old git revision:\n{updated_lockfile}"
    );

    let updated_instructions =
        fs::read_to_string(consumer_repo.join("INSTRUCTIONS.md")).expect("instructions");
    assert!(updated_instructions.contains("Updated testing guidance after republish"));
    assert!(!updated_instructions.contains("Testing guidance\n"));
}

#[test]
fn remote_preview_reports_republished_artifact_without_mutating_lockfile() {
    let workspace = temporary_directory("pray-remote-preview");
    let source_repo = workspace.join("source");
    let distribution_repo = workspace.join("distribution");
    let prayers_root = distribution_repo.join("prayers");
    let consumer_repo = workspace.join("consumer");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&distribution_repo).expect("distribution workspace");
    fs::create_dir_all(&consumer_repo).expect("consumer workspace");

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
            prayers_root.to_str().expect("distribution path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    assert_success(
        &git(&distribution_repo, &["init", "-b", "main"]),
        "git init",
    );
    assert_success(
        &git(&distribution_repo, &["config", "user.name", "Pray Test"]),
        "git user.name",
    );
    assert_success(
        &git(
            &distribution_repo,
            &["config", "user.email", "pray@example.com"],
        ),
        "git user.email",
    );
    assert_success(&git(&distribution_repo, &["add", "-A"]), "git add");
    assert_success(
        &git(
            &distribution_repo,
            &["commit", "-m", "initial distribution"],
        ),
        "git commit",
    );

    let initial_commit =
        String::from_utf8_lossy(&git(&distribution_repo, &["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();

    fs::write(
        consumer_repo.join("Prayfile"),
        format!(
            r#"
prayfile "1"
source "dist", "git+file://{distribution}"
agent "sample/base", "~> 1.4", source: "dist"
target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
"#,
            distribution = distribution_repo.display()
        ),
    )
    .expect("write consumer Prayfile");

    let install = run_pray(&consumer_repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let initial_lockfile =
        fs::read_to_string(consumer_repo.join("Prayfile.lock")).expect("lockfile");
    assert!(initial_lockfile.contains(&initial_commit));

    fs::write(
        source_repo.join("packages/base/exports/testing-basics.md"),
        "Updated testing guidance after republish\n",
    )
    .expect("rewrite export");
    let republish = run_pray(
        &source_repo,
        &[
            "publish",
            "--root",
            prayers_root.to_str().expect("distribution path"),
        ],
    );
    assert!(
        republish.status.success(),
        "republish failed: {}",
        String::from_utf8_lossy(&republish.stderr)
    );
    assert_success(
        &git(&distribution_repo, &["add", "-A"]),
        "git add republish",
    );
    assert_success(
        &git(
            &distribution_repo,
            &["commit", "-m", "republish same version"],
        ),
        "git commit republish",
    );

    let outdated_remote = run_pray(&consumer_repo, &["outdated", "--remote"]);
    assert!(
        outdated_remote.status.success(),
        "outdated --remote failed: {}",
        String::from_utf8_lossy(&outdated_remote.stderr)
    );
    let outdated_stdout = String::from_utf8_lossy(&outdated_remote.stdout);
    assert!(
        outdated_stdout.contains("Remote update preview"),
        "outdated --remote should preview remote changes:\n{outdated_stdout}"
    );
    assert!(
        outdated_stdout.contains("Refreshed package sample/base")
            || outdated_stdout.contains("Updated source dist revision"),
        "outdated --remote should report upstream drift:\n{outdated_stdout}"
    );

    let dry_run = run_pray(&consumer_repo, &["update", "--dry-run"]);
    assert!(
        dry_run.status.success(),
        "update --dry-run failed: {}",
        String::from_utf8_lossy(&dry_run.stderr)
    );
    let dry_run_stdout = String::from_utf8_lossy(&dry_run.stdout);
    assert!(
        dry_run_stdout.contains("Remote update preview"),
        "update --dry-run should preview remote changes:\n{dry_run_stdout}"
    );

    let lockfile_after_preview =
        fs::read_to_string(consumer_repo.join("Prayfile.lock")).expect("lockfile");
    assert_eq!(
        initial_lockfile, lockfile_after_preview,
        "remote preview commands must not mutate Prayfile.lock"
    );

    let instructions_after_preview =
        fs::read_to_string(consumer_repo.join("INSTRUCTIONS.md")).expect("instructions");
    assert!(
        !instructions_after_preview.contains("Updated testing guidance after republish"),
        "remote preview must not apply rendered output"
    );
}

#[test]
fn install_reuses_git_cache_on_second_run() {
    let workspace = temporary_directory("pray-install-git-cache");
    let source_repo = workspace.join("source");
    let distribution_repo = workspace.join("distribution");
    let prayers_root = distribution_repo.join("prayers");
    let consumer_repo = workspace.join("consumer");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&distribution_repo).expect("distribution workspace");
    fs::create_dir_all(&consumer_repo).expect("consumer workspace");

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
            prayers_root.to_str().expect("distribution path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    assert_success(
        &git(&distribution_repo, &["init", "-b", "main"]),
        "git init",
    );
    assert_success(
        &git(&distribution_repo, &["config", "user.name", "Pray Test"]),
        "git user.name",
    );
    assert_success(
        &git(
            &distribution_repo,
            &["config", "user.email", "pray@example.com"],
        ),
        "git user.email",
    );
    assert_success(&git(&distribution_repo, &["add", "-A"]), "git add");
    assert_success(
        &git(
            &distribution_repo,
            &["commit", "-m", "initial distribution"],
        ),
        "git commit",
    );

    fs::write(
        consumer_repo.join("Prayfile"),
        format!(
            r#"
prayfile "1"
source "dist", "git+file://{distribution}"
agent "sample/base", "~> 1.4", source: "dist"
target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
"#,
            distribution = distribution_repo.display()
        ),
    )
    .expect("write consumer Prayfile");

    let first_install = run_pray(&consumer_repo, &["install"]);
    assert!(
        first_install.status.success(),
        "first install failed: {}",
        String::from_utf8_lossy(&first_install.stderr)
    );

    let second_install = run_pray(&consumer_repo, &["install", "--locked"]);
    assert!(
        second_install.status.success(),
        "second install failed: {}",
        String::from_utf8_lossy(&second_install.stderr)
    );
}

#[test]
fn publish_serve_install_and_confess_end_to_end_with_web_surface() {
    let workspace = temporary_directory("pray-publish-e2e");
    let source_repo = workspace.join("source");
    let registry_root = workspace.join("registry");
    let registry_root_mirror = workspace.join("registry-mirror");
    let client_a = workspace.join("client-a");
    let client_b = workspace.join("client-b");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&registry_root).expect("registry workspace");
    fs::create_dir_all(&registry_root_mirror).expect("registry mirror workspace");
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
            "email_confirmation": "required",
            "passkeys_enabled": true,
            "ssh_keys_enabled": true,
            "ssh_agent_signing_enabled": true
        }"#,
    )
    .expect("write trust settings");
    fs::create_dir_all(registry_root_mirror.join("v1")).expect("registry mirror v1 workspace");
    fs::write(
        registry_root_mirror.join("v1/index.json"),
        r#"{
            "spec": "prayfile-distribution-1",
            "packages": []
        }"#,
    )
    .expect("write mirror index");
    fs::write(
        registry_root_mirror.join("v1/trust.json"),
        r#"{
            "email_confirmation": "required",
            "passkeys_enabled": true,
            "ssh_keys_enabled": true,
            "ssh_agent_signing_enabled": true
        }"#,
    )
    .expect("write mirror trust settings");
    fs::create_dir_all(&client_a).expect("client A workspace");
    fs::create_dir_all(&client_b).expect("client B workspace");

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

    let auth_store = RegistryAuthStore::open(&registry_root).expect("open auth store");
    let publisher_email = "sample-agent-packages@example.com";
    let client_a_email = "client-a@example.com";
    let client_b_email = "client-b@example.com";

    let publisher_key = signing_key_from_seed(11);
    let client_a_key = signing_key_from_seed(12);
    let client_b_key = signing_key_from_seed(13);
    let publisher_public_key = ssh_public_key_text(&publisher_key);
    let client_a_public_key = ssh_public_key_text(&client_a_key);
    let client_b_public_key = ssh_public_key_text(&client_b_key);

    verify_email_registration(&auth_store, publisher_email);
    verify_email_registration(&auth_store, client_a_email);
    verify_email_registration(&auth_store, client_b_email);

    auth_store
        .enroll_passkey(
            publisher_email,
            "publisher-passkey",
            &publisher_public_key,
            Some("publisher passkey"),
        )
        .expect("enroll publisher passkey");
    auth_store
        .enroll_passkey(
            client_a_email,
            "client-a-passkey",
            &client_a_public_key,
            Some("client A passkey"),
        )
        .expect("enroll client A passkey");
    auth_store
        .enroll_ssh_key(
            client_b_email,
            &client_b_public_key,
            Some("client B workstation"),
        )
        .expect("enroll client B ssh key");

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

    let server_url = format!("http://127.0.0.1:{port}");
    write_registry_client_fixture(&client_a, &server_url);
    write_registry_client_fixture(&client_b, &server_url);

    let publisher_private_key_path =
        write_private_key_file(&source_repo, "publisher-passkey.bin", &publisher_key);
    let client_a_private_key_path =
        write_private_key_file(&client_a, "client-a-passkey.bin", &client_a_key);
    let client_b_public_key_path =
        write_public_key_file(&client_b, "client-b-public-key.pub", &client_b_public_key);

    run_pray_login_passkey(
        &source_repo,
        &server_url,
        publisher_email,
        "publisher-passkey",
        &publisher_private_key_path,
    );
    run_pray_login_passkey(
        &client_a,
        &server_url,
        client_a_email,
        "client-a-passkey",
        &client_a_private_key_path,
    );

    let ssh_agent_socket = PathBuf::from("/tmp/pray-ssh-agent.sock");
    let ssh_agent_handle = spawn_mock_ssh_agent(&ssh_agent_socket, client_b_key);
    run_pray_login_ssh_agent(
        &client_b,
        &server_url,
        client_b_email,
        &client_b_public_key_path,
        &ssh_agent_socket,
    );
    ssh_agent_handle.join().expect("ssh agent finished");

    let publish = run_pray(
        &source_repo,
        &[
            "publish",
            "--root",
            registry_root.to_str().expect("registry path"),
            "--root",
            registry_root_mirror.to_str().expect("registry mirror path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    let index_text =
        fs::read_to_string(registry_root.join("v1/index.json")).expect("registry index");
    assert!(index_text.contains("prayfile-distribution-1"));
    assert!(index_text.contains("sample/base"));

    let mirror_index_text = fs::read_to_string(registry_root_mirror.join("v1/index.json"))
        .expect("registry mirror index");
    assert!(mirror_index_text.contains("prayfile-distribution-1"));
    assert!(mirror_index_text.contains("sample/base"));

    let metadata_text = fs::read_to_string(registry_root.join("v1/packages/sample/base.json"))
        .expect("package metadata");
    let metadata: Value = serde_json::from_str(&metadata_text).expect("package metadata json");
    let version = metadata
        .get("versions")
        .and_then(Value::as_array)
        .and_then(|versions| versions.first())
        .expect("published version");
    let artifact_path = version
        .get("artifact")
        .and_then(Value::as_str)
        .expect("artifact path");
    let signature = version
        .get("signature")
        .and_then(Value::as_str)
        .expect("signature");
    let signer = version
        .get("signer")
        .and_then(Value::as_str)
        .expect("signer");
    assert_eq!(signer, publisher_email);
    assert!(signature.starts_with("sha256:"));
    assert!(registry_root.join(artifact_path).is_file());
    assert!(registry_root_mirror.join(artifact_path).is_file());

    let ruby_script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/distribution_point_smoke.rb");
    let smoke = Command::new("ruby")
        .arg(ruby_script)
        .arg("--pray-bin")
        .arg(env!("CARGO_BIN_EXE_pray"))
        .arg("--server-url")
        .arg(&server_url)
        .arg("--client")
        .arg(&client_a)
        .arg("--client")
        .arg(&client_b)
        .output()
        .expect("run ruby smoke test");

    let _ = server.kill();
    let _ = server.wait();

    assert!(
        smoke.status.success(),
        "ruby smoke test failed: {}",
        String::from_utf8_lossy(&smoke.stderr)
    );
    let stdout = String::from_utf8_lossy(&smoke.stdout);
    assert!(stdout.contains("distribution point smoke test passed"));
}
