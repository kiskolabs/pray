#[path = "install_network_support.rs"]
mod support;

use serde_json::json;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use support::{
    create_add_fixture, find_free_port, run_pray, spawn_server, temporary_directory,
    wait_for_server,
};

fn run_command(
    program: &str,
    directory: &Path,
    arguments: &[&str],
    environment: &[(&str, &str)],
) -> Output {
    let mut command = Command::new(program);
    command.current_dir(directory).args(arguments);
    for (key, value) in environment {
        command.env(key, value);
    }
    command.output().expect("run command")
}

fn assert_success(output: &Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git(directory: &Path, arguments: &[&str]) -> Output {
    run_command("git", directory, arguments, &[])
}

fn hg(directory: &Path, arguments: &[&str], environment: &[(&str, &str)]) -> Output {
    run_command("hg", directory, arguments, environment)
}

fn pray(directory: &Path, arguments: &[&str], environment: &[(&str, &str)]) -> Output {
    run_command(
        env!("CARGO_BIN_EXE_pray"),
        directory,
        arguments,
        environment,
    )
}

fn write_revision_config(root: &Path, config: serde_json::Value) {
    let v1 = root.join("v1");
    fs::create_dir_all(&v1).expect("revision config directory");
    fs::write(
        v1.join("revision.json"),
        serde_json::to_string_pretty(&config).expect("serialize revision config"),
    )
    .expect("write revision config");
}

#[test]
fn publish_records_git_revision_and_pushes_to_remote() {
    let workspace = temporary_directory("pray-revision-git");
    let repository = workspace.join("repo");
    let remote = workspace.join("remote.git");
    fs::create_dir_all(&repository).expect("repository workspace");

    assert_success(
        &git(
            &workspace,
            &["init", "--bare", remote.to_str().expect("remote path")],
        ),
        "git init --bare",
    );
    assert_success(&git(&repository, &["init", "-b", "main"]), "git init");
    assert_success(
        &git(&repository, &["config", "user.name", "Pray Test"]),
        "git user.name",
    );
    assert_success(
        &git(&repository, &["config", "user.email", "pray@example.com"]),
        "git user.email",
    );
    assert_success(
        &git(
            &repository,
            &[
                "remote",
                "add",
                "origin",
                remote.to_str().expect("remote path"),
            ],
        ),
        "git remote add",
    );

    create_add_fixture(&repository);
    write_revision_config(
        &repository,
        json!({
            "backend": "git",
            "remote": "origin",
            "push": true,
            "commit_message": "publish revision for git"
        }),
    );

    let add = run_pray(
        &repository,
        &["add", "sample/base", "--path", "packages/base"],
    );
    assert_success(&add, "pray add");

    let publish = pray(
        &repository,
        &[
            "publish",
            "--root",
            repository.to_str().expect("repository path"),
        ],
        &[],
    );
    assert_success(&publish, "pray publish");

    let local_commit = git(&repository, &["rev-parse", "HEAD"]);
    assert_success(&local_commit, "git rev-parse HEAD");
    let remote_commit = git(&remote, &["rev-parse", "refs/heads/main"]);
    assert_success(&remote_commit, "git rev-parse remote branch");

    assert_eq!(
        String::from_utf8_lossy(&local_commit.stdout).trim(),
        String::from_utf8_lossy(&remote_commit.stdout).trim(),
    );

    let commit_message = git(&repository, &["log", "-1", "--pretty=%s"]);
    assert_success(&commit_message, "git log");
    assert_eq!(
        String::from_utf8_lossy(&commit_message.stdout).trim(),
        "publish revision for git"
    );
}

#[test]
fn publish_rejects_git_push_when_remote_has_advanced() {
    let workspace = temporary_directory("pray-revision-git-conflict");
    let repository = workspace.join("repo");
    let remote = workspace.join("remote.git");
    let competing_clone = workspace.join("competing");
    fs::create_dir_all(&repository).expect("repository workspace");

    assert_success(
        &git(
            &workspace,
            &["init", "--bare", remote.to_str().expect("remote path")],
        ),
        "git init --bare",
    );
    assert_success(&git(&repository, &["init", "-b", "main"]), "git init");
    assert_success(
        &git(&repository, &["config", "user.name", "Pray Test"]),
        "git user.name",
    );
    assert_success(
        &git(&repository, &["config", "user.email", "pray@example.com"]),
        "git user.email",
    );
    assert_success(
        &git(
            &repository,
            &[
                "remote",
                "add",
                "origin",
                remote.to_str().expect("remote path"),
            ],
        ),
        "git remote add",
    );

    create_add_fixture(&repository);
    write_revision_config(
        &repository,
        json!({
            "backend": "git",
            "remote": "origin",
            "push": true
        }),
    );

    let add = run_pray(
        &repository,
        &["add", "sample/base", "--path", "packages/base"],
    );
    assert_success(&add, "pray add");

    let publish = pray(
        &repository,
        &[
            "publish",
            "--root",
            repository.to_str().expect("repository path"),
        ],
        &[],
    );
    assert_success(&publish, "pray publish");
    assert_success(
        &git(&remote, &["symbolic-ref", "HEAD", "refs/heads/main"]),
        "bare default branch",
    );

    assert_success(
        &git(
            &workspace,
            &[
                "clone",
                remote.to_str().expect("remote path"),
                competing_clone.to_str().expect("competing path"),
            ],
        ),
        "git clone",
    );
    assert_success(
        &git(&competing_clone, &["config", "user.name", "Pray Test"]),
        "competing user.name",
    );
    assert_success(
        &git(
            &competing_clone,
            &["config", "user.email", "pray@example.com"],
        ),
        "competing user.email",
    );
    fs::write(
        competing_clone.join("remote-note.txt"),
        "remote moved ahead\n",
    )
    .expect("write competing commit");
    assert_success(
        &git(&competing_clone, &["add", "remote-note.txt"]),
        "competing add",
    );
    assert_success(
        &git(&competing_clone, &["commit", "-m", "advance remote"]),
        "competing commit",
    );
    assert_success(
        &git(
            &competing_clone,
            &["push", "origin", "HEAD:refs/heads/main"],
        ),
        "competing push",
    );

    fs::write(
        repository.join("packages/base/README.md"),
        "package readme updated\n",
    )
    .expect("update package readme");

    let failed_publish = pray(
        &repository,
        &[
            "publish",
            "--root",
            repository.to_str().expect("repository path"),
        ],
        &[],
    );
    assert!(!failed_publish.status.success());
    let stderr = String::from_utf8_lossy(&failed_publish.stderr);
    assert!(
        stderr.contains("rejected")
            || stderr.contains("non-fast-forward")
            || stderr.contains("fetch and retry")
            || stderr.contains("remote branch moved")
    );
}

#[test]
fn publish_records_hg_revision_without_configured_backend() {
    let workspace = temporary_directory("pray-revision-hg");
    let repository = workspace.join("repo");
    fs::create_dir_all(&repository).expect("repository workspace");

    assert_success(&hg(&repository, &["init"], &[]), "hg init");
    fs::write(
        repository.join(".hg/hgrc"),
        "[ui]\nusername = Pray Test <pray@example.com>\n",
    )
    .expect("write hg hgrc");

    create_add_fixture(&repository);
    let add = run_pray(
        &repository,
        &["add", "sample/base", "--path", "packages/base"],
    );
    assert_success(&add, "pray add");

    let publish = pray(
        &repository,
        &[
            "publish",
            "--root",
            repository.to_str().expect("repository path"),
        ],
        &[("HGUSER", "Pray Test <pray@example.com>")],
    );
    assert_success(&publish, "pray publish");

    let commit_message = hg(
        &repository,
        &["log", "-r", ".", "--template", "{desc}"],
        &[],
    );
    assert_success(&commit_message, "hg log");
    assert_eq!(
        String::from_utf8_lossy(&commit_message.stdout).trim(),
        "pray publish: update distribution state"
    );
}

#[test]
fn sync_records_custom_revision_commands_after_successful_sync() {
    let workspace = temporary_directory("pray-revision-sync");
    let source = workspace.join("source");
    let upstream = workspace.join("upstream");
    let downstream = workspace.join("downstream");
    fs::create_dir_all(&source).expect("source workspace");
    fs::create_dir_all(&upstream).expect("upstream workspace");
    fs::create_dir_all(&downstream).expect("downstream workspace");

    create_add_fixture(&source);
    let add = run_pray(&source, &["add", "sample/base", "--path", "packages/base"]);
    assert_success(&add, "pray add");

    let publish = run_pray(
        &source,
        &[
            "publish",
            "--root",
            upstream.to_str().expect("upstream path"),
        ],
    );
    assert_success(&publish, "pray publish");

    let commit_log = workspace.join("commit.log");
    let push_log = workspace.join("push.log");
    let commit_script = workspace.join("commit.sh");
    let push_script = workspace.join("push.sh");
    fs::write(
        &commit_script,
        "#!/bin/sh\nprintf 'commit:%s\\n' \"$PWD\" >> \"$1\"\n",
    )
    .expect("write commit script");
    fs::write(
        &push_script,
        "#!/bin/sh\nprintf 'push:%s\\n' \"$PWD\" >> \"$1\"\n",
    )
    .expect("write push script");
    write_revision_config(
        &downstream,
        json!({
            "backend": "other",
            "push": true,
            "commit_command": {
                "program": "sh",
                "args": [commit_script.to_str().expect("commit script path"), commit_log.to_str().expect("commit log path")]
            },
            "push_command": {
                "program": "sh",
                "args": [push_script.to_str().expect("push script path"), push_log.to_str().expect("push log path")]
            }
        }),
    );

    fs::create_dir_all(downstream.join("v1")).expect("downstream v1 workspace");
    let port = find_free_port();
    let mut server = spawn_server(&upstream, port);
    wait_for_server(port);
    let upstream_url = format!("http://127.0.0.1:{port}");
    fs::write(
        downstream.join("v1/peers.json"),
        format!(
            r#"[
                {{
                    "name": "upstream",
                    "url": "{upstream_url}",
                    "public": true
                }}
            ]"#
        ),
    )
    .expect("write peer list");

    let sync = run_pray(
        &workspace,
        &[
            "sync",
            "--root",
            downstream.to_str().expect("downstream path"),
        ],
    );
    assert_success(&sync, "pray sync");

    let synced_index =
        fs::read_to_string(downstream.join("v1/index.json")).expect("downstream index");
    assert!(synced_index.contains("sample/base"));

    let commit_log_text = fs::read_to_string(&commit_log).expect("commit log");
    let push_log_text = fs::read_to_string(&push_log).expect("push log");
    let downstream_path = downstream
        .canonicalize()
        .unwrap_or_else(|_| downstream.clone());
    assert_eq!(
        commit_log_text.trim(),
        format!("commit:{}", downstream_path.display())
    );
    assert_eq!(
        push_log_text.trim(),
        format!("push:{}", downstream_path.display())
    );

    let _ = server.kill();
    let _ = server.wait();
}
