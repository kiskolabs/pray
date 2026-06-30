#[path = "install_network_support.rs"]
mod support;

use serde_json::Value;
use std::fs;

use support::{
    create_add_fixture, find_free_port, run_pray, spawn_server, temporary_directory,
    wait_for_server,
};

#[test]
fn sync_crawls_discovered_peers_and_persists_them() {
    let workspace = temporary_directory("pray-sync-crawl");
    let source_repo = workspace.join("source");
    let seed_root = workspace.join("seed");
    let upstream_root = workspace.join("upstream");
    let downstream_root = workspace.join("downstream");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&seed_root).expect("seed workspace");
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

    fs::create_dir_all(seed_root.join("v1")).expect("seed v1 workspace");
    fs::write(
        seed_root.join("v1/index.json"),
        r#"{
            "spec": "prayfile-distribution-1",
            "packages": []
        }"#,
    )
    .expect("write seed index");

    let seed_port = find_free_port();
    let upstream_port = find_free_port();
    let upstream_url = format!("http://127.0.0.1:{upstream_port}");
    let seed_url = format!("http://127.0.0.1:{seed_port}");
    fs::write(
        seed_root.join("v1/peers.json"),
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
    .expect("write seed peers");
    fs::create_dir_all(downstream_root.join("v1")).expect("downstream v1 workspace");
    fs::write(
        downstream_root.join("v1/peers.json"),
        format!(
            r#"[
                {{
                    "name": "seed",
                    "url": "{seed_url}",
                    "public": true
                }}
            ]"#
        ),
    )
    .expect("write downstream peer list");

    let mut seed_server = spawn_server(&seed_root, seed_port);
    let mut upstream_server = spawn_server(&upstream_root, upstream_port);
    wait_for_server(seed_port);
    wait_for_server(upstream_port);

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
    assert!(stdout.contains("Synchronized 1 package(s) from 2 peer(s); learned 2 peer(s)"));

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
    assert_eq!(synced_peers.as_array().expect("peer array").len(), 2);
    assert_eq!(synced_peers[0]["url"], seed_url);
    assert_eq!(synced_peers[1]["url"], upstream_url);

    let _ = seed_server.kill();
    let _ = seed_server.wait();
    let _ = upstream_server.kill();
    let _ = upstream_server.wait();
}
