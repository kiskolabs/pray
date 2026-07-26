use crate::server_auth::{
    auth_passkey_challenge_response, auth_passkey_enroll_response, auth_passkey_login_response,
    auth_register_response, auth_session_response, auth_ssh_key_challenge_response,
    auth_ssh_key_enroll_response, auth_ssh_key_login_response, auth_verify_response,
};
use crate::server_federation::{
    federation_discovery_response, federation_index_response_since, federation_package_response,
    federation_push_response,
};
use crate::server_html::{html_package_response, html_root_response};
use crate::server_http::{
    decode_rpc_base64_body, http_response_to_rpc, http_to_rpc_request, rpc_response_to_http,
    strip_query, write_response,
};

pub(crate) use crate::server_http::{response_with_status, Response};
use pray_core::derived_metadata::derive_registry_derived_metadata_from_archive_bytes;
use pray_core::push_auth::authorize_distribution_push;
use pray_core::registry::{
    ConfessionSubmission, RegistryIndex, RegistryPackageMetadata, RegistryPackageVersion,
};
use pray_core::ssh_rpc::{RpcRequest, RpcResponse, SSH_RPC_SPEC};
use pray_core::{PrayError, PrayResult};
use pray_transport::{PackageMetadata as TransportPackageMetadata, PackageVersion, PeerInfo};
use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::thread;

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

pub fn run_server(root: PathBuf, host: String, port: u16, allow_open_push: bool) -> PrayResult<()> {
    let listener = TcpListener::bind((host.as_str(), port))?;
    println!("Serving {} on http://{}:{}", root.display(), host, port);
    let auth = ServeAuth::http(host, allow_open_push);
    for connection in listener.incoming() {
        let stream = connection?;
        let root = root.clone();
        let auth = auth.clone();
        thread::spawn(move || {
            if let Err(error) = handle_connection(root, auth, stream) {
                eprintln!("serve error: {error}");
            }
        });
    }
    Ok(())
}

fn handle_connection(root: PathBuf, auth: ServeAuth, mut stream: TcpStream) -> PrayResult<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }
    let request_line = request_line.trim_end_matches(['\r', '\n']);
    if request_line.is_empty() {
        return Ok(());
    }
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| PrayError::Resolution("missing HTTP method".to_string()))?;
    let path = parts
        .next()
        .ok_or_else(|| PrayError::Resolution("missing HTTP path".to_string()))?;

    let mut content_length = 0usize;
    loop {
        let mut header_line = String::new();
        reader.read_line(&mut header_line)?;
        let trimmed = header_line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value
                    .trim()
                    .parse::<usize>()
                    .map_err(|error| PrayError::Resolution(error.to_string()))?;
            }
        }
    }

    let mut body = vec![0; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    let response = dispatch_http_request(&root, &auth, method, path, &body)?;

    write_response(
        &mut stream,
        response.status,
        &response.content_type,
        response.body,
    )?;
    Ok(())
}

pub(crate) fn dispatch_http_request(
    root: &Path,
    auth: &ServeAuth,
    method: &str,
    path: &str,
    body: &[u8],
) -> PrayResult<Response> {
    if let Some(rpc_request) = http_to_rpc_request(method, path, body)? {
        let rpc_response = match handle_rpc(root, auth, &rpc_request) {
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

pub(crate) fn ensure_derived_metadata(
    root: &Path,
    metadata: &mut RegistryPackageMetadata,
) -> PrayResult<()> {
    for version in &mut metadata.versions {
        if version.derived_metadata.is_some() {
            continue;
        }
        let artifact_path = root.join(&version.artifact);
        let artifact_bytes = fs::read(&artifact_path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                PrayError::Resolution(format!(
                    "artifact not found for derived metadata: {}",
                    version.artifact
                ))
            } else {
                PrayError::from(error)
            }
        })?;
        version.derived_metadata = Some(derive_registry_derived_metadata_from_archive_bytes(
            &artifact_bytes,
        )?);
    }
    Ok(())
}

pub(crate) fn read_registry_package_metadata(
    root: &Path,
    package_name: &str,
) -> PrayResult<RegistryPackageMetadata> {
    let metadata_path = registry_metadata_path(root, package_name);
    let metadata_text = fs::read_to_string(&metadata_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PrayError::Resolution(format!("package metadata not found: {}", package_name))
        } else {
            PrayError::from(error)
        }
    })?;
    let metadata: RegistryPackageMetadata =
        serde_json::from_str(&metadata_text).map_err(|error| PrayError::Parse {
            kind: "registry metadata",
            message: error.to_string(),
        })?;
    if metadata.name != package_name {
        return Err(PrayError::Resolution(format!(
            "registry metadata name mismatch: expected {}, found {}",
            package_name, metadata.name
        )));
    }
    Ok(metadata)
}

pub(crate) fn write_registry_package_metadata(
    path: &Path,
    metadata: &RegistryPackageMetadata,
) -> PrayResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(metadata)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    )?;
    Ok(())
}

pub(crate) fn registry_metadata_path(root: &Path, package_name: &str) -> PathBuf {
    root.join("v1/packages")
        .join(package_name)
        .with_extension("json")
}

pub(crate) fn read_known_peers(root: &Path) -> PrayResult<Vec<PeerInfo>> {
    let path = root.join("v1/peers.json");
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let peers: Vec<PeerInfo> = serde_json::from_str(&text).map_err(|error| PrayError::Parse {
        kind: "peer list",
        message: error.to_string(),
    })?;
    for peer in &peers {
        if peer.name.trim().is_empty() || peer.url.trim().is_empty() {
            return Err(PrayError::Resolution(
                "peer list contains an entry with an empty name or url".to_string(),
            ));
        }
    }
    Ok(peers)
}

pub(crate) fn latest_publish_timestamp(metadata: &RegistryPackageMetadata) -> Option<u64> {
    metadata
        .versions
        .iter()
        .filter_map(|version| version.published_at.as_deref())
        .filter_map(|published_at| published_at.parse::<u64>().ok())
        .max()
}

pub(crate) fn registry_package_metadata_from_transport(
    metadata: &TransportPackageMetadata,
) -> PrayResult<RegistryPackageMetadata> {
    if metadata.name.trim().is_empty() {
        return Err(PrayError::Resolution(
            "federation package metadata missing package name".to_string(),
        ));
    }

    let mut seen_versions = BTreeSet::new();
    let mut versions = Vec::new();
    for version in &metadata.versions {
        let registry_version = registry_package_version_from_transport(version)?;
        if !seen_versions.insert(registry_version.version.clone()) {
            return Err(PrayError::Resolution(format!(
                "duplicate package version in federation payload: {} {}",
                metadata.name, registry_version.version
            )));
        }
        versions.push(registry_version);
    }

    Ok(RegistryPackageMetadata {
        name: metadata.name.clone(),
        versions,
    })
}

pub(crate) fn registry_package_version_from_transport(
    version: &PackageVersion,
) -> PrayResult<RegistryPackageVersion> {
    if version.version.trim().is_empty() {
        return Err(PrayError::Resolution(
            "federation package version missing version string".to_string(),
        ));
    }
    if version.artifact.trim().is_empty() {
        return Err(PrayError::Resolution(format!(
            "federation package version {} missing artifact path",
            version.version
        )));
    }

    let signer = version
        .publisher
        .as_ref()
        .and_then(|publisher| {
            if publisher.id.trim().is_empty() {
                None
            } else {
                Some(publisher.id.clone())
            }
        })
        .or_else(|| {
            version
                .signature
                .as_ref()
                .map(|signature| signature.public_key.clone())
        })
        .filter(|signer| !signer.trim().is_empty());
    let signer_fingerprint = version
        .publisher
        .as_ref()
        .map(|publisher| publisher.key_fingerprint.clone())
        .filter(|fingerprint| !fingerprint.trim().is_empty());
    let signature = version
        .signature
        .as_ref()
        .map(|signature| signature.signature.clone())
        .filter(|signature| !signature.trim().is_empty());
    let published_at = if version.published_at.trim().is_empty() {
        None
    } else {
        Some(version.published_at.clone())
    };

    Ok(RegistryPackageVersion {
        version: version.version.clone(),
        artifact: version.artifact.clone(),
        artifact_hash: empty_string_to_none(&version.artifact_hash),
        tree_hash: empty_string_to_none(&version.tree_hash),
        yanked: version.yanked,
        targets: version.targets.clone(),
        exports: version.exports.clone(),
        signer,
        signer_fingerprint,
        signer_public_key: version
            .signature
            .as_ref()
            .map(|value| value.public_key.clone())
            .and_then(|value| empty_string_to_none(&value)),
        published_at,
        signature,
        derived_metadata: version.derived_metadata.clone(),
    })
}

pub(crate) fn merge_registry_package_metadata(
    root: &Path,
    incoming: RegistryPackageMetadata,
) -> PrayResult<RegistryPackageMetadata> {
    let mut current = read_or_create_registry_package_metadata(root, &incoming.name)?;
    for incoming_version in incoming.versions {
        match current
            .versions
            .iter()
            .position(|version| version.version == incoming_version.version)
        {
            Some(index) if current.versions[index].same_identity(&incoming_version) => {
                current.versions[index].merge_annotations_from(&incoming_version);
            }
            Some(_) => {
                return Err(PrayError::Resolution(format!(
                    "conflicting package version received for {} {}",
                    incoming.name, incoming_version.version
                )));
            }
            None => current.versions.push(incoming_version),
        }
    }
    Ok(current)
}

pub(crate) fn read_or_create_registry_package_metadata(
    root: &Path,
    package_name: &str,
) -> PrayResult<RegistryPackageMetadata> {
    match read_registry_package_metadata(root, package_name) {
        Ok(metadata) => Ok(metadata),
        Err(PrayError::Resolution(message))
            if message.starts_with("package metadata not found") =>
        {
            Ok(RegistryPackageMetadata {
                name: package_name.to_string(),
                versions: Vec::new(),
            })
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn update_registry_index_with_package(
    root: &Path,
    package_name: &str,
) -> PrayResult<()> {
    let mut index = read_or_create_registry_index(root)?;
    if index.spec.trim().is_empty() {
        index.spec = "prayfile-distribution-1".to_string();
    }
    if !index
        .packages
        .iter()
        .any(|existing| existing == package_name)
    {
        index.packages.push(package_name.to_string());
    }
    write_registry_index(root, &index)
}

fn read_or_create_registry_index(root: &Path) -> PrayResult<RegistryIndex> {
    let index_path = root.join("v1/index.json");
    match fs::read_to_string(&index_path) {
        Ok(index_text) => serde_json::from_str(&index_text).map_err(|error| PrayError::Parse {
            kind: "registry index",
            message: error.to_string(),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(RegistryIndex {
            spec: "prayfile-distribution-1".to_string(),
            packages: Vec::new(),
        }),
        Err(error) => Err(error.into()),
    }
}

fn write_registry_index(root: &Path, index: &RegistryIndex) -> PrayResult<()> {
    let path = root.join("v1/index.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(index)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    )?;
    Ok(())
}

fn empty_string_to_none(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn confession_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
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

fn artifact_upload_response(
    root: &Path,
    auth: &ServeAuth,
    path: &str,
    body: &[u8],
) -> PrayResult<Response> {
    authorize_distribution_push(root, &auth.bind_host, auth.allow_open_push, auth.stdio_mode)?;
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

fn static_file_response(root: &Path, request_path: &str) -> PrayResult<Response> {
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

pub(crate) fn read_registry_index(root: &Path) -> PrayResult<RegistryIndex> {
    let index_path = root.join("v1/index.json");
    let index_text = fs::read_to_string(&index_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PrayError::Resolution("missing registry index".to_string())
        } else {
            PrayError::from(error)
        }
    })?;
    serde_json::from_str(&index_text).map_err(|error| PrayError::Parse {
        kind: "registry index",
        message: error.to_string(),
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
