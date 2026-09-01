use super::{http_response_to_rpc, response_with_status, rpc_response_to_http};
use crate::server::{dispatch_http_request, handle_rpc, ServeAuth};
use crate::server_federation::federation_discovery_response;
use pray_core::ssh_rpc::{RpcRequest, SSH_RPC_SPEC};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

fn temporary_root(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("pray-http-rpc-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(path.join("v1")).expect("v1 directory");
    fs::write(
        path.join("v1/index.json"),
        r#"{"spec":"prayfile-distribution-1","packages":[]}"#,
    )
    .expect("index");
    path
}

#[test]
fn http_discovery_matches_direct_handler() {
    let root = temporary_root("discovery");
    let direct = federation_discovery_response(&root).expect("direct response");
    let auth = ServeAuth::http("127.0.0.1", false);
    let bridged = dispatch_http_request(
        &root,
        &auth,
        "GET",
        "/.well-known/pray-federation.json",
        &[],
    )
    .expect("bridged response");
    assert_eq!(direct.status, bridged.status);
    assert_eq!(direct.content_type, bridged.content_type);
    let direct_json: serde_json::Value = serde_json::from_slice(&direct.body).expect("direct json");
    let bridged_json: serde_json::Value =
        serde_json::from_slice(&bridged.body).expect("bridged json");
    assert_eq!(direct_json, bridged_json);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn rpc_response_round_trips_through_http_envelope() {
    let response = response_with_status(200, "application/json", br#"{"ok":true}"#.to_vec());
    let rpc = http_response_to_rpc("1", response);
    let http = rpc_response_to_http(&rpc);
    assert_eq!(http.status, 200);
    assert_eq!(http.content_type, "application/json");
    assert_eq!(http.body, br#"{"ok":true}"#);
}

#[test]
fn handle_rpc_and_http_dispatch_share_sync_package_path() {
    let root = temporary_root("sync-package");
    let metadata_path = root.join("v1/packages/sample/base.json");
    fs::create_dir_all(metadata_path.parent().unwrap()).expect("package directory");
    fs::write(
        &metadata_path,
        r#"{"name":"sample/base","versions":[{"version":"1.0.0","artifact":"v1/artifacts/sample/base/1.0.0/package.praypkg"}]}"#,
    )
    .expect("metadata");
    fs::write(
        root.join("v1/index.json"),
        r#"{"spec":"prayfile-distribution-1","packages":["sample/base"]}"#,
    )
    .expect("index");

    let auth = ServeAuth::http("127.0.0.1", false);
    let rpc = handle_rpc(
        &root,
        &auth,
        &RpcRequest::new("1", "sync.package", json!({ "name": "sample/base" })),
    )
    .expect("rpc response");
    assert_eq!(rpc.spec, SSH_RPC_SPEC);
    assert_eq!(rpc.status, 200);

    let http = dispatch_http_request(&root, &auth, "GET", "/v1/sync/package/sample/base", &[])
        .expect("http response");
    assert_eq!(http.status, 200);
    assert_eq!(rpc_response_to_http(&rpc).body, http.body);
    let first = crate::server_http::next_http_request_id();
    let second = crate::server_http::next_http_request_id();
    assert_ne!(first, second);
    let _ = fs::remove_dir_all(&root);
}
