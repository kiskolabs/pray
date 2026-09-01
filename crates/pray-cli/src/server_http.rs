use pray_core::ssh_rpc::{RpcRequest, RpcResponse, SSH_RPC_SPEC};
use pray_core::{PrayError, PrayResult};
use std::io::Write;
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};

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

static HTTP_REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub(crate) fn next_http_request_id() -> String {
    format!(
        "http-{}",
        HTTP_REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}

pub(crate) fn http_to_rpc_request(
    method: &str,
    path: &str,
    body: &[u8],
    request_id: &str,
) -> PrayResult<Option<RpcRequest>> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use serde_json::json;

    let request_path = strip_query(path);

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
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        413 => "Payload Too Large",
        431 => "Request Header Fields Too Large",
        503 => "Service Unavailable",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

#[cfg(test)]
#[path = "server_http_tests.rs"]
mod http_rpc_bridge_tests;
