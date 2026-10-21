#![allow(dead_code)]

#[path = "support/git_catalog.rs"]
mod git_catalog;

use git_catalog::{git, git_head, pinned_shallow_cache_fixture, PinnedShallowCache};
use pray_core::embed::{resolve_project_with_options, ResolveOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct HostRepository {
    path: PathBuf,
    git_dir: PathBuf,
}

struct HostState {
    origin_url: String,
    refs: String,
    index: Vec<u8>,
}

struct CallerEnv {
    names: Vec<&'static str>,
}

impl Drop for CallerEnv {
    fn drop(&mut self) {
        for name in &self.names {
            std::env::remove_var(name);
        }
    }
}

#[test]
fn install_from_a_hook_with_git_dir_leaves_the_host_repository_alone() {
    assert_host_untouched_with(|host| vec![("GIT_DIR", host.git_dir.clone())]);
}

#[test]
fn install_from_a_hook_with_git_index_file_leaves_the_host_repository_alone() {
    assert_host_untouched_with(|host| vec![("GIT_INDEX_FILE", host.git_dir.join("index"))]);
}

#[test]
fn install_from_a_hook_with_git_work_tree_leaves_the_host_repository_alone() {
    assert_host_untouched_with(|host| {
        vec![
            ("GIT_DIR", host.git_dir.clone()),
            ("GIT_WORK_TREE", host.path.clone()),
            ("GIT_INDEX_FILE", host.git_dir.join("index")),
        ]
    });
}

#[test]
fn update_from_a_hook_with_git_dir_leaves_the_host_head_and_branches_alone() {
    let fixture = pinned_shallow_cache_fixture(false);
    let host = host_repository(&fixture);
    let before = host_state(&host);
    let options = ResolveOptions {
        refresh_source_revisions: true,
        ignore_locked_versions: true,
        ..ResolveOptions::default()
    };

    let result = {
        let _env = set_caller_env(vec![("GIT_DIR", host.git_dir.clone())]);
        resolve_project_with_options(&fixture.root.join("Prayfile"), &options)
    };

    assert_host_state_unchanged(&host, &before);
    let project = result.expect("update should refresh the cache, not the host");
    assert_eq!(
        project.source_revisions.get("used").map(String::as_str),
        Some(git_head(&fixture.origin).as_str())
    );
}

fn assert_host_untouched_with(
    caller_env: impl Fn(&HostRepository) -> Vec<(&'static str, PathBuf)>,
) {
    let fixture = pinned_shallow_cache_fixture(false);
    let host = host_repository(&fixture);
    let before = host_state(&host);

    let result = {
        let _env = set_caller_env(caller_env(&host));
        resolve_project_with_options(&fixture.root.join("Prayfile"), &ResolveOptions::default())
    };

    assert_host_state_unchanged(&host, &before);
    let project = result.expect("install should resolve against the cache, not the host");
    assert_eq!(
        project.source_revisions.get("used").map(String::as_str),
        Some(fixture.pinned.as_str())
    );
}

fn assert_host_state_unchanged(host: &HostRepository, before: &HostState) {
    let after = host_state(host);
    let mut changes = Vec::new();
    if after.origin_url != before.origin_url {
        changes.push(format!(
            "origin URL: {} -> {}",
            before.origin_url, after.origin_url
        ));
    }
    if after.refs != before.refs {
        changes.push(format!(
            "HEAD or refs:\n{}\n->\n{}",
            before.refs, after.refs
        ));
    }
    if after.index != before.index {
        changes.push("index".to_string());
    }
    assert!(
        changes.is_empty(),
        "host repository changed:\n{}",
        changes.join("\n")
    );
}

fn host_repository(fixture: &PinnedShallowCache) -> HostRepository {
    let remote = fixture.root.join("host-remote.git");
    git(
        &fixture.root,
        &[
            "init",
            "--bare",
            "--template=",
            remote.to_str().expect("remote"),
        ],
    );
    let path = fixture.root.join("host");
    fs::create_dir_all(&path).expect("host");
    fs::write(path.join("README.md"), "host\n").expect("host file");
    git(&path, &["init", "--template=", "-b", "main"]);
    git(&path, &["add", "-A"]);
    git(
        &path,
        &[
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "-m",
            "host",
        ],
    );
    git(
        &path,
        &["remote", "add", "origin", remote.to_str().expect("remote")],
    );
    git(&path, &["push", "origin", "main"]);
    git(&path, &["fetch", "origin"]);
    let path = path.canonicalize().expect("canonicalize host");
    HostRepository {
        git_dir: path.join(".git"),
        path,
    }
}

fn host_state(host: &HostRepository) -> HostState {
    HostState {
        origin_url: git_stdout(&host.path, &["remote", "get-url", "origin"]),
        refs: git_stdout(&host.path, &["show-ref", "--head"]),
        index: fs::read(host.git_dir.join("index")).expect("host index"),
    }
}

fn set_caller_env(variables: Vec<(&'static str, PathBuf)>) -> CallerEnv {
    let names = variables.iter().map(|(name, _)| *name).collect();
    for (name, value) in variables {
        std::env::set_var(name, value);
    }
    CallerEnv { names }
}

fn git_stdout(directory: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(directory)
        .args(arguments)
        .output()
        .expect("git");
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
