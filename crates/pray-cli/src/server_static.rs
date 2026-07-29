use crate::server::ServeAuth;
use crate::server_http::Response;
use pray_core::push_auth::authorize_distribution_push;
use pray_core::registry::ConfessionSubmission;
use pray_core::{PrayError, PrayResult};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

pub(crate) fn confession_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let confession: ConfessionSubmission =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "confession",
            message: error.to_string(),
        })?;
    let confession_path = root.join("v1/confessions.jsonl");
    if let Some(parent) = confession_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&confession_path)?;
    let line = serde_json::to_string(&confession)
        .map_err(|error| PrayError::Manifest(error.to_string()))?;
    writeln!(file, "{line}")?;
    let response_body = serde_json::json!({
        "status": "ok",
        "package": confession.package,
        "version": confession.version,
    })
    .to_string();
    Ok(Response {
        status: 201,
        content_type: "application/json".to_string(),
        body: response_body.into_bytes(),
    })
}

pub(crate) fn artifact_upload_response(
    root: &Path,
    auth: &ServeAuth,
    path: &str,
    body: &[u8],
) -> PrayResult<Response> {
    authorize_distribution_push(
        root,
        &auth.bind_host,
        auth.allow_open_push,
        auth.stdio_mode,
        auth.authorization.as_deref(),
    )?;
    let relative_path = sanitize_request_path(path)?;
    let artifact_path = root.join(relative_path);
    if let Some(parent) = artifact_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&artifact_path, body)?;
    Ok(Response {
        status: 201,
        content_type: "application/json".to_string(),
        body: serde_json::json!({
            "status": "ok",
            "artifact": path,
        })
        .to_string()
        .into_bytes(),
    })
}

pub(crate) fn static_file_response(root: &Path, request_path: &str) -> PrayResult<Response> {
    let relative = sanitize_request_path(request_path)?;
    let path = root.join(relative);
    if path.is_dir() {
        return Err(PrayError::Resolution(format!(
            "directory requested as file: {}",
            request_path
        )));
    }
    let body = fs::read(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PrayError::Resolution(format!("not found: {}", request_path))
        } else {
            PrayError::from(error)
        }
    })?;
    let content_type = content_type_for_path(&path);
    Ok(Response {
        status: 200,
        content_type,
        body,
    })
}

pub(crate) fn sanitize_request_path(path: &str) -> PrayResult<PathBuf> {
    let path = path.trim_start_matches('/');
    let mut relative = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            _ => {
                return Err(PrayError::Resolution(format!(
                    "invalid request path: {path}"
                )))
            }
        }
    }
    Ok(relative)
}

pub(crate) fn content_type_for_path(path: &Path) -> String {
    match path.extension().and_then(|value| value.to_str()) {
        Some("json") => "application/json".to_string(),
        Some("jsonl") => "application/x-ndjson".to_string(),
        Some("md") | Some("txt") => "text/plain; charset=utf-8".to_string(),
        Some("html") => "text/html; charset=utf-8".to_string(),
        Some("praypkg") => "application/octet-stream".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}
