use crate::server_html::{html_package_response, html_root_response};
use crate::server_http::{http_to_rpc_request, rpc_response_to_http, strip_query};
use pray_core::ssh_rpc::RpcResponse;
use pray_core::PrayResult;
use std::path::Path;

pub(crate) use crate::server_http::{response_with_status, Response};
pub(crate) use crate::server_registry::{
    ensure_derived_metadata, latest_publish_timestamp, merge_registry_package_metadata,
    read_known_peers, read_registry_index, read_registry_package_metadata, registry_metadata_path,
    update_registry_index_with_package, write_registry_package_metadata,
};
pub(crate) use crate::transport_metadata::registry_package_metadata_from_transport;
pub use crate::server_rpc::handle_rpc;

#[derive(Debug, Clone)]
pub struct ServeAuth {
    pub bind_host: String,
    pub allow_open_push: bool,
    pub stdio_mode: bool,
}

impl ServeAuth {
    pub fn http(bind_host: impl Into<String>, allow_open_push: bool) -> Self {
        Self {
            bind_host: bind_host.into(),
            allow_open_push,
            stdio_mode: false,
        }
    }

    pub fn stdio() -> Self {
        Self {
            bind_host: "stdio".to_string(),
            allow_open_push: false,
            stdio_mode: true,
        }
    }
}

pub(crate) fn dispatch_http_request(
    root: &Path,
    auth: &ServeAuth,
    method: &str,
    path: &str,
    body: &[u8],
) -> PrayResult<Response> {
    if let Some(rpc_request) = http_to_rpc_request(method, path, body)? {
        let rpc_response = match crate::server_rpc::handle_rpc(root, auth, &rpc_request) {
            Ok(response) => response,
            Err(error) => RpcResponse::error(&rpc_request.id, 500, error.to_string()),
        };
        return Ok(rpc_response_to_http(&rpc_response));
    }

    match (method, strip_query(path)) {
        ("GET", "/") => html_root_response(root),
        ("GET", path) if path.starts_with("/packages/") => html_package_response(root, path),
        _ => Ok(response_with_status(
            405,
            "text/plain",
            b"method not allowed".to_vec(),
        )),
    }
}
