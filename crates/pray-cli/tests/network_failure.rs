#[path = "install_network_support.rs"]
mod support;

use std::fs;

use support::{
    create_add_fixture, find_free_port, run_pray, spawn_server, temporary_directory,
    wait_for_server,
};

#[test]
fn install_fails_against_unreachable_registry_host() {
    let repo = temporary_directory("pray-unreachable-registry");
    create_add_fixture(&repo);
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
source "default", "http://127.0.0.1:1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
pray "sample/base", "~> 1.4"
"#,
    )
    .expect("Prayfile");

    let failed = run_pray(&repo, &["install"]);
    assert!(!failed.status.success());
    assert_eq!(failed.status.code(), Some(7));
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(
        stderr.contains("network error")
            || stderr.contains("Connection refused")
            || stderr.contains("timed out")
            || stderr.contains("error sending request"),
        "unexpected stderr: {stderr}"
    );
    assert!(!repo.join("Prayfile.lock").exists());
}

#[test]
fn publish_fails_against_unreachable_http_server() {
    let workspace = temporary_directory("pray-publish-unreachable");
    let source_repo = workspace.join("source");
    fs::create_dir_all(&source_repo).expect("source");
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

    let port = find_free_port();
    let server_url = format!("http://127.0.0.1:{port}");
    let failed = run_pray(&source_repo, &["publish", "--server", &server_url]);
    assert!(!failed.status.success());
    assert!(matches!(failed.status.code(), Some(7) | Some(8)));
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(
        stderr.contains("network error")
            || stderr.contains("Connection refused")
            || stderr.contains("timed out")
            || stderr.contains("error sending request")
            || stderr.contains("unsupported feature"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn sync_fails_when_peer_returns_before_listen() {
    let workspace = temporary_directory("pray-sync-peer-down");
    let downstream_root = workspace.join("downstream");
    fs::create_dir_all(downstream_root.join("v1")).expect("downstream");
    let port = find_free_port();
    let peer_url = format!("http://127.0.0.1:{port}");
    fs::write(
        downstream_root.join("v1/peers.json"),
        format!(
            r#"[
                {{
                    "name": "missing",
                    "url": "{peer_url}",
                    "public": true
                }}
            ]"#
        ),
    )
    .expect("peers");

    let failed = run_pray(
        &workspace,
        &[
            "sync",
            "--root",
            downstream_root.to_str().expect("downstream path"),
        ],
    );
    assert!(!failed.status.success());
    assert_eq!(failed.status.code(), Some(7));
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(
        stderr.contains("network error")
            || stderr.contains("Connection refused")
            || stderr.contains("timed out")
            || stderr.contains("Network error"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn serve_port_is_closed_after_process_stop() {
    let root = temporary_directory("pray-serve-stopped");
    fs::create_dir_all(root.join("v1")).expect("v1");
    fs::write(
        root.join("v1/index.json"),
        r#"{ "spec": "prayfile-distribution-1", "packages": [] }"#,
    )
    .expect("index");
    let port = find_free_port();
    let mut server = spawn_server(&root, port);
    wait_for_server(port);
    let _ = server.kill();
    let _ = server.wait();

    let connect = std::net::TcpStream::connect_timeout(
        &format!("127.0.0.1:{port}").parse().expect("addr"),
        std::time::Duration::from_millis(200),
    );
    assert!(
        connect.is_err(),
        "server port should refuse connections after stop"
    );
}
