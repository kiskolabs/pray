use pray_core::ssh_rpc::{RpcRequest, RpcResponse, SSH_RPC_SPEC};
use pray_core::{PrayError, PrayResult};
use std::io::Write;
use std::net::TcpStream;

pub(crate) struct Response {
    pub(crate) status: u16,
    pub(crate) content_type: String,
    pub(crate) body: Vec<u8>,
}

pub(crate) fn response_with_status(status: u16, content_type: &str, body: Vec<u8>) -> Response {
    Response {
        status,
        content_type: content_type.to_string(),
        body,
    }
}

pub(crate) fn strip_query(path: &str) -> &str {
    path.split_once('?').map(|(path, _)| path).unwrap_or(path)
}

pub(crate) fn query_parameter(path: &str, name: &str) -> Option<String> {
    let query = path.split_once('?')?.1;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key == name {
            return Some(value.to_string());
        }
    }
    None
}

pub(crate) fn http_to_rpc_request(method: &str, path: &str, body: &[u8]) -> PrayResult<Option<RpcRequest>> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use serde_json::json;

    let request_path = strip_query(path);
    let request_id = "http";

    let request = match (method, request_path) {
        ("GET", "/.well-known/pray-federation.json") => {
            RpcRequest::new(request_id, "federation.discovery", json!({}))
        }
        ("GET", "/v1/sync/index") => {
            let mut params = json!({});
            if let Some(since) =
                query_parameter(path, "since").and_then(|value| value.parse::<i64>().ok())
            {
                params["since"] = json!(since);
            }
            RpcRequest::new(request_id, "sync.index", params)
        }
        ("GET", path) if path.starts_with("/v1/sync/package/") => {
            let package_name = path.trim_start_matches("/v1/sync/package/");
            RpcRequest::new(request_id, "sync.package", json!({ "name": package_name }))
        }
        ("POST", "/v1/sync/push") => {
            let metadata: serde_json::Value =
                serde_json::from_slice(body).map_err(|error| PrayError::Parse {
                    kind: "federation package metadata",
                    message: error.to_string(),
                })?;
            RpcRequest::new(request_id, "sync.push", json!({ "metadata": metadata }))
        }
        ("PUT", path) if path.starts_with("/v1/artifacts/") => RpcRequest::new(
            request_id,
            "artifact.put",
            json!({
                "path": path.trim_start_matches('/'),
                "body": STANDARD.encode(body),
            }),
        ),
        ("POST", "/v1/confessions") => {
            rpc_request_with_json_field(request_id, "confession.submit", "confession", body)?
        }
        ("POST", "/v1/auth/register") => {
            rpc_request_with_json_field(request_id, "auth.register", "request", body)?
        }
        ("POST", "/v1/auth/verify") => {
            rpc_request_with_json_field(request_id, "auth.verify", "request", body)?
        }
        ("POST", "/v1/auth/session") => {
            rpc_request_with_json_field(request_id, "auth.session", "request", body)?
        }
        ("POST", "/v1/auth/passkeys/challenge") => {
            rpc_request_with_json_field(request_id, "auth.passkeys.challenge", "request", body)?
        }
        ("POST", "/v1/auth/passkeys/login") => {
            rpc_request_with_json_field(request_id, "auth.passkeys.login", "request", body)?
        }
        ("POST", "/v1/auth/passkeys/enroll") => {
            rpc_request_with_json_field(request_id, "auth.passkeys.enroll", "request", body)?
        }
        ("POST", "/v1/auth/ssh-keys/challenge") => {
            rpc_request_with_json_field(request_id, "auth.ssh_keys.challenge", "request", body)?
        }
        ("POST", "/v1/auth/ssh-keys/login") => {
            rpc_request_with_json_field(request_id, "auth.ssh_keys.login", "request", body)?
        }
        ("POST", "/v1/auth/ssh-keys/enroll") => {
            rpc_request_with_json_field(request_id, "auth.ssh_keys.enroll", "request", body)?
        }
        ("GET", "/") => return Ok(None),
        ("GET", path) if path.starts_with("/packages/") => return Ok(None),
        ("GET", path) => RpcRequest::new(
            request_id,
            "artifact.get",
            json!({ "path": path.trim_start_matches('/') }),
        ),
        _ => return Ok(None),
    };

    Ok(Some(request))
}

fn rpc_request_with_json_field(
    request_id: &str,
    method: &str,
    field_name: &str,
    body: &[u8],
) -> PrayResult<RpcRequest> {
    let value: serde_json::Value =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "request body",
            message: error.to_string(),
        })?;
    Ok(RpcRequest::new(
        request_id,
        method,
        serde_json::json!({ field_name: value }),
    ))
}

pub(crate) fn rpc_response_to_http(response: &RpcResponse) -> Response {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    let body = if response.body_encoding.as_deref() == Some("base64") {
        response
            .body
            .as_str()
            .map(|encoded| STANDARD.decode(encoded).unwrap_or_default())
            .unwrap_or_default()
    } else if response.content_type.starts_with("application/json") {
        serde_json::to_vec(&response.body)
            .unwrap_or_else(|_| response.body.to_string().into_bytes())
    } else if let Some(text) = response.body.as_str() {
        text.as_bytes().to_vec()
    } else {
        response.body.to_string().into_bytes()
    };

    Response {
        status: response.status,
        content_type: response.content_type.clone(),
        body,
    }
}

pub(crate) fn http_response_to_rpc(id: &str, response: Response) -> RpcResponse {
    if response.content_type.starts_with("application/json") {
        let body = serde_json::from_slice(&response.body).unwrap_or_else(|_| {
            serde_json::json!({
                "error": String::from_utf8_lossy(&response.body)
            })
        });
        RpcResponse {
            spec: SSH_RPC_SPEC.to_string(),
            id: id.to_string(),
            status: response.status,
            content_type: response.content_type,
            body_encoding: None,
            body,
        }
    } else {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        RpcResponse {
            spec: SSH_RPC_SPEC.to_string(),
            id: id.to_string(),
            status: response.status,
            content_type: response.content_type,
            body_encoding: Some("base64".to_string()),
            body: serde_json::Value::String(STANDARD.encode(&response.body)),
        }
    }
}

pub(crate) fn decode_rpc_base64_body(value: Option<&serde_json::Value>) -> PrayResult<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let encoded = value
        .and_then(|value| value.as_str())
        .ok_or_else(|| PrayError::Resolution("artifact.put requires base64 body".to_string()))?;
    STANDARD.decode(encoded).map_err(|error| {
        PrayError::Resolution(format!("artifact.put body base64 decode failed: {error}"))
    })
}

pub(crate) fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: Vec<u8>,
) -> PrayResult<()> {
    let reason = reason_phrase(status);
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(&body)?;
    stream.flush()?;
    Ok(())
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

#[cfg(test)]
mod http_rpc_bridge_tests {
    use super::{
        http_response_to_rpc, response_with_status, rpc_response_to_http,
    };
    use crate::server::{dispatch_http_request, handle_rpc, ServeAuth};
    use crate::server_federation::federation_discovery_response;
    use pray_core::ssh_rpc::{RpcRequest, SSH_RPC_SPEC};
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    fn temporary_root(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("pray-http-rpc-{name}-{}", std::process::id()));
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
        let bridged =
            dispatch_http_request(&root, &auth, "GET", "/.well-known/pray-federation.json", &[])
                .expect("bridged response");
        assert_eq!(direct.status, bridged.status);
        assert_eq!(direct.content_type, bridged.content_type);
        let direct_json: serde_json::Value =
            serde_json::from_slice(&direct.body).expect("direct json");
        let bridged_json: serde_json::Value =
            serde_json::from_slice(&bridged.body).expect("bridged json");
        assert_eq!(direct_json, bridged_json);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rpc_response_round_trips_through_http_envelope() {
        let response =
            response_with_status(200, "application/json", br#"{"ok":true}"#.to_vec());
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
        let _ = fs::remove_dir_all(&root);
    }
}
