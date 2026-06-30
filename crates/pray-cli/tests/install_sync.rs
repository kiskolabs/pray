#[path = "install_network_support.rs"]
mod support;

use serde_json::Value;
use std::fs;

use support::{
    create_add_fixture, find_free_port, run_pray, spawn_server, temporary_directory,
    wait_for_server,
};

#[test]
fn sync_pulls_packages_from_configured_peers() {
    let workspace = temporary_directory("pray-sync");
    let source_repo = workspace.join("source");
    let upstream_root = workspace.join("upstream");
    let downstream_root = workspace.join("downstream");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&upstream_root).expect("upstream workspace");
    fs::create_dir_all(&downstream_root).expect("downstream workspace");

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
            upstream_root.to_str().expect("upstream path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    let port = find_free_port();
    let mut server = spawn_server(&upstream_root, port);
    wait_for_server(port);

    let upstream_url = format!("http://127.0.0.1:{port}");
    fs::create_dir_all(downstream_root.join("v1")).expect("downstream v1 workspace");
    fs::write(
        downstream_root.join("v1/peers.json"),
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
            downstream_root.to_str().expect("downstream path"),
        ],
    );
    assert!(
        sync.status.success(),
        "sync failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );
    let stdout = String::from_utf8_lossy(&sync.stdout);
    assert!(stdout.contains("Synchronized 1 package(s) from 1 peer(s); learned 1 peer(s)"));

    let downstream_index_text =
        fs::read_to_string(downstream_root.join("v1/index.json")).expect("downstream index");
    assert!(downstream_index_text.contains("sample/base"));

    let downstream_metadata_text =
        fs::read_to_string(downstream_root.join("v1/packages/sample/base.json"))
            .expect("downstream package metadata");
    let downstream_metadata: Value =
        serde_json::from_str(&downstream_metadata_text).expect("downstream metadata json");
    assert_eq!(downstream_metadata["name"], "sample/base");
    assert_eq!(
        downstream_metadata["versions"]
            .as_array()
            .expect("versions")
            .len(),
        1
    );
    let downstream_version = downstream_metadata["versions"][0].clone();
    assert_eq!(downstream_version["version"], "1.4.3");
    let artifact_path = downstream_version["artifact"]
        .as_str()
        .expect("artifact path");
    let downstream_artifact =
        fs::read(downstream_root.join(artifact_path)).expect("downstream artifact");

    let upstream_artifact_path = upstream_root.join(artifact_path);
    let upstream_artifact = fs::read(&upstream_artifact_path).expect("upstream artifact");
    assert_eq!(downstream_artifact, upstream_artifact);

    let synced_peers_text =
        fs::read_to_string(downstream_root.join("v1/peers.json")).expect("synced peers file");
    let synced_peers: Value = serde_json::from_str(&synced_peers_text).expect("synced peers json");
    assert_eq!(synced_peers.as_array().expect("peer array").len(), 1);
    assert_eq!(synced_peers[0]["url"], upstream_url);

    let _ = server.kill();
    let _ = server.wait();
}

#[test]
fn sync_recovers_after_peer_restart_and_leaves_no_partial_state() {
    let workspace = temporary_directory("pray-sync-recovery");
    let source_repo = workspace.join("source");
    let upstream_root = workspace.join("upstream");
    let downstream_root = workspace.join("downstream");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&upstream_root).expect("upstream workspace");
    fs::create_dir_all(&downstream_root).expect("downstream workspace");

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
            upstream_root.to_str().expect("upstream path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    let port = find_free_port();
    let upstream_url = format!("http://127.0.0.1:{port}");
    fs::create_dir_all(downstream_root.join("v1")).expect("downstream v1 workspace");
    fs::write(
        downstream_root.join("v1/peers.json"),
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
    .expect("write downstream peer list");
    let initial_peers =
        fs::read_to_string(downstream_root.join("v1/peers.json")).expect("initial peers file");

    let failed_sync = run_pray(
        &workspace,
        &[
            "sync",
            "--root",
            downstream_root.to_str().expect("downstream path"),
        ],
    );
    assert!(!failed_sync.status.success());
    assert!(matches!(failed_sync.status.code(), Some(1) | Some(3)));
    let failed_stderr = String::from_utf8_lossy(&failed_sync.stderr);
    assert!(
        failed_stderr.contains("Network error")
            || failed_stderr.contains("Connection refused")
            || failed_stderr.contains("timed out")
            || failed_stderr.contains("No such file")
            || failed_stderr.contains("resolution error")
    );
    assert_eq!(
        fs::read_to_string(downstream_root.join("v1/peers.json")).expect("peers after failure"),
        initial_peers
    );
    assert!(!downstream_root.join("v1/index.json").exists());
    assert!(!downstream_root.join("v1/packages").exists());

    let mut upstream_server = spawn_server(&upstream_root, port);
    wait_for_server(port);

    let recovered_sync = run_pray(
        &workspace,
        &[
            "sync",
            "--root",
            downstream_root.to_str().expect("downstream path"),
        ],
    );
    assert!(
        recovered_sync.status.success(),
        "recovered sync failed: {}",
        String::from_utf8_lossy(&recovered_sync.stderr)
    );
    let stdout = String::from_utf8_lossy(&recovered_sync.stdout);
    assert!(stdout.contains("Synchronized 1 package(s) from 1 peer(s); learned 1 peer(s)"));
    assert!(downstream_root.join("v1/index.json").exists());
    assert!(downstream_root
        .join("v1/packages/sample/base.json")
        .exists());

    let _ = upstream_server.kill();
    let _ = upstream_server.wait();
}
