use crate::server::ServeAuth;
use crate::server_auth::{
    auth_passkey_challenge_response, auth_passkey_enroll_response, auth_passkey_login_response,
    auth_register_response, auth_session_response, auth_ssh_key_challenge_response,
    auth_ssh_key_enroll_response, auth_ssh_key_login_response, auth_verify_response,
};
use crate::server_federation::{
    federation_discovery_response, federation_index_response_since, federation_package_response,
    federation_push_response,
};
use crate::server_http::{decode_rpc_base64_body, http_response_to_rpc, response_with_status};
use crate::server_static::{artifact_upload_response, confession_response, static_file_response};
use pray_core::ssh_rpc::{RpcRequest, RpcResponse, SSH_RPC_SPEC};
use pray_core::{PrayError, PrayResult};
use std::path::Path;

pub fn handle_rpc(root: &Path, auth: &ServeAuth, request: &RpcRequest) -> PrayResult<RpcResponse> {
    if request.spec != SSH_RPC_SPEC {
        return Ok(RpcResponse::error(
            &request.id,
            400,
            format!("unsupported rpc spec: {}", request.spec),
        ));
    }

    let response = match request.method.as_str() {
        "federation.discovery" => federation_discovery_response(root)?,
        "sync.index" => {
            let since = request
                .params
                .get("since")
                .and_then(|value| value.as_i64())
                .map(|value| value as u64);
            federation_index_response_since(root, since)?
        }
        "sync.package" => {
            let package_name = request
                .params
                .get("name")
                .and_then(|value| value.as_str())
                .ok_or_else(|| PrayError::Resolution("sync.package requires name".to_string()))?;
            federation_package_response(root, &format!("/v1/sync/package/{package_name}"))?
        }
        "sync.push" => {
            let metadata = request
                .params
                .get("metadata")
                .ok_or_else(|| PrayError::Resolution("sync.push requires metadata".to_string()))?;
            federation_push_response(
                root,
                auth,
                &serde_json::to_vec(metadata)
                    .map_err(|error| PrayError::Manifest(error.to_string()))?,
            )?
        }
        "artifact.get" => {
            let path = request
                .params
                .get("path")
                .and_then(|value| value.as_str())
                .ok_or_else(|| PrayError::Resolution("artifact.get requires path".to_string()))?;
            static_file_response(root, &format!("/{path}"))?
        }
        "artifact.put" => {
            let path = request
                .params
                .get("path")
                .and_then(|value| value.as_str())
                .ok_or_else(|| PrayError::Resolution("artifact.put requires path".to_string()))?;
            let body = decode_rpc_base64_body(request.params.get("body"))?;
            artifact_upload_response(root, auth, &format!("/{path}"), &body)?
        }
        "confession.submit" => confession_response(
            root,
            &serde_json::to_vec(request.params.get("confession").ok_or_else(|| {
                PrayError::Resolution("confession.submit requires confession".to_string())
            })?)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
        )?,
        "auth.register" => auth_register_response(
            root,
            &serde_json::to_vec(request.params.get("request").ok_or_else(|| {
                PrayError::Resolution("auth.register requires request".to_string())
            })?)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
        )?,
        "auth.verify" => auth_verify_response(
            root,
            &serde_json::to_vec(request.params.get("request").ok_or_else(|| {
                PrayError::Resolution("auth.verify requires request".to_string())
            })?)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
        )?,
        "auth.session" => auth_session_response(
            root,
            &serde_json::to_vec(request.params.get("request").ok_or_else(|| {
                PrayError::Resolution("auth.session requires request".to_string())
            })?)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
        )?,
        "auth.passkeys.challenge" => auth_passkey_challenge_response(
            root,
            &serde_json::to_vec(request.params.get("request").ok_or_else(|| {
                PrayError::Resolution("auth.passkeys.challenge requires request".to_string())
            })?)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
        )?,
        "auth.passkeys.login" => auth_passkey_login_response(
            root,
            &serde_json::to_vec(request.params.get("request").ok_or_else(|| {
                PrayError::Resolution("auth.passkeys.login requires request".to_string())
            })?)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
        )?,
        "auth.passkeys.enroll" => auth_passkey_enroll_response(
            root,
            &serde_json::to_vec(request.params.get("request").ok_or_else(|| {
                PrayError::Resolution("auth.passkeys.enroll requires request".to_string())
            })?)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
        )?,
        "auth.ssh_keys.challenge" => auth_ssh_key_challenge_response(
            root,
            &serde_json::to_vec(request.params.get("request").ok_or_else(|| {
                PrayError::Resolution("auth.ssh_keys.challenge requires request".to_string())
            })?)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
        )?,
        "auth.ssh_keys.login" => auth_ssh_key_login_response(
            root,
            &serde_json::to_vec(request.params.get("request").ok_or_else(|| {
                PrayError::Resolution("auth.ssh_keys.login requires request".to_string())
            })?)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
        )?,
        "auth.ssh_keys.enroll" => auth_ssh_key_enroll_response(
            root,
            &serde_json::to_vec(request.params.get("request").ok_or_else(|| {
                PrayError::Resolution("auth.ssh_keys.enroll requires request".to_string())
            })?)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
        )?,
        _ => response_with_status(405, "text/plain", b"method not allowed".to_vec()),
    };

    Ok(http_response_to_rpc(&request.id, response))
}
