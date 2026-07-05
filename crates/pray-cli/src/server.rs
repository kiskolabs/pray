use pray_core::auth::{
    AuthPasskeyChallengeRequest, AuthPasskeyChallengeResponse, AuthPasskeyEnrollmentRequest,
    AuthPasskeyEnrollmentResponse, AuthPasskeyLoginRequest, AuthPasskeyLoginResponse,
    AuthRegistrationRequest, AuthRegistrationResponse, AuthSessionKind, AuthSessionRequest,
    AuthSessionResponse, AuthSshKeyChallengeRequest, AuthSshKeyChallengeResponse,
    AuthSshKeyEnrollmentRequest, AuthSshKeyEnrollmentResponse, AuthSshKeyLoginRequest,
    AuthSshKeyLoginResponse, AuthVerificationRequest, AuthVerificationResponse, RegistryAuthStore,
};
use pray_core::derived_metadata::derive_registry_derived_metadata_from_archive_bytes;
use pray_core::registry::{
    registry_package_signing_identity, ConfessionSubmission, RegistryIndex,
    RegistryPackageMetadata, RegistryPackageVersion,
};
use pray_core::ssh_publishers::authorize_ssh_push;
use pray_core::ssh_rpc::{RpcRequest, RpcResponse, SSH_RPC_SPEC};
use pray_core::trust::read_registry_trust_settings;
use pray_core::{PrayError, PrayResult};
use pray_transport::{
    FederationInfo, IndexResponse, OriginInfo, PackageMetadata as TransportPackageMetadata,
    PackageSummary, PackageVersion, PeerInfo, PublisherInfo, ServerInfo, SignatureInfo,
    SyncEndpoints,
};
use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::thread;

pub fn run_server(root: PathBuf, host: String, port: u16) -> PrayResult<()> {
    let listener = TcpListener::bind((host.as_str(), port))?;
    println!("Serving {} on http://{}:{}", root.display(), host, port);
    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                let root = root.clone();
                thread::spawn(move || {
                    if let Err(error) = handle_connection(root, stream) {
                        eprintln!("serve error: {error}");
                    }
                });
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn handle_connection(root: PathBuf, mut stream: TcpStream) -> PrayResult<()> {
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

    let response = dispatch_http_request(&root, method, path, &body)?;

    write_response(
        &mut stream,
        response.status,
        &response.content_type,
        response.body,
    )?;
    Ok(())
}

fn dispatch_http_request(
    root: &Path,
    method: &str,
    path: &str,
    body: &[u8],
) -> PrayResult<Response> {
    if let Some(rpc_request) = http_to_rpc_request(method, path, body)? {
        let rpc_response = match handle_rpc(root, &rpc_request) {
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

fn http_to_rpc_request(method: &str, path: &str, body: &[u8]) -> PrayResult<Option<RpcRequest>> {
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

fn rpc_response_to_http(response: &RpcResponse) -> Response {
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

struct Response {
    status: u16,
    content_type: String,
    body: Vec<u8>,
}

fn html_root_response(root: &Path) -> PrayResult<Response> {
    let index = read_registry_index(root)?;
    let trust = read_registry_trust_settings(root)?;
    let mut list_items = String::new();
    for package in index.packages {
        list_items.push_str(&format!(
            "<li><a href=\"/packages/{path}\">{package}</a></li>",
            path = html_escape(&package)
        ));
    }
    let body = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Pray distribution point</title></head><body><h1>Pray distribution point</h1><p>Spec: {spec}</p><p>Email confirmation: {email}</p><p>Passkeys: {passkeys}</p><p>SSH keys: {ssh_keys}</p><p>SSH-agent signing: {ssh_agent}</p><ul>{packages}</ul></body></html>",
        spec = html_escape(&index.spec),
        email = trust.email_confirmation_label(),
        passkeys = trust.passkeys_label(),
        ssh_keys = trust.ssh_keys_label(),
        ssh_agent = trust.ssh_agent_label(),
        packages = list_items,
    );
    Ok(Response {
        status: 200,
        content_type: "text/html; charset=utf-8".to_string(),
        body: body.into_bytes(),
    })
}

fn federation_discovery_response(root: &Path) -> PrayResult<Response> {
    let discovery = FederationInfo {
        spec: "pray-federation-v1".to_string(),
        server: ServerInfo {
            name: "pray".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: vec!["static_registry".to_string(), "federation".to_string()],
        },
        sync: SyncEndpoints {
            index_url: "/v1/sync/index".to_string(),
            package_url: "/v1/sync/package/{name}".to_string(),
            artifact_url: "/v1/artifacts/{package}/{version}/{artifact}".to_string(),
            since_param: "since".to_string(),
        },
        peers: read_known_peers(root)?,
    };
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body: serde_json::to_vec_pretty(&discovery)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    })
}

fn federation_index_response_since(root: &Path, since: Option<u64>) -> PrayResult<Response> {
    let index = read_registry_index(root)?;
    let mut packages = Vec::new();
    let mut sync_version = 0u64;

    for package_name in index.packages {
        let Ok(metadata) = read_registry_package_metadata(root, &package_name) else {
            continue;
        };
        let updated_at = latest_publish_timestamp(&metadata)
            .map(|timestamp| timestamp.to_string())
            .unwrap_or_else(|| "0".to_string());
        let updated_at_value = updated_at.parse::<u64>().unwrap_or(0);
        sync_version = sync_version.max(updated_at_value);
        if since.is_some_and(|since| updated_at_value <= since) {
            continue;
        }
        packages.push(PackageSummary {
            name: package_name.clone(),
            updated_at,
            url: format!("/v1/sync/package/{package_name}"),
        });
    }

    let body = IndexResponse {
        spec: "prayfile-distribution-1".to_string(),
        sync_version: sync_version as i64,
        packages,
    };

    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body: serde_json::to_vec_pretty(&body)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    })
}

fn federation_package_response(root: &Path, path: &str) -> PrayResult<Response> {
    let package_name = path.trim_start_matches("/v1/sync/package/");
    let metadata = read_registry_package_metadata(root, package_name)?;
    let body = transport_package_metadata(&metadata);
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body: serde_json::to_vec_pretty(&body)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    })
}

fn federation_push_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    if std::env::var("PRAY_SERVE_STDIO").is_ok() {
        authorize_ssh_push(root)?;
    }
    let incoming: TransportPackageMetadata =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "federation package metadata",
            message: error.to_string(),
        })?;
    let registry_metadata = registry_package_metadata_from_transport(&incoming)?;
    let mut merged_metadata = merge_registry_package_metadata(root, registry_metadata)?;
    ensure_derived_metadata(root, &mut merged_metadata)?;
    let metadata_path = registry_metadata_path(root, &merged_metadata.name);
    write_registry_package_metadata(&metadata_path, &merged_metadata)?;
    update_registry_index_with_package(root, &merged_metadata.name)?;
    Ok(Response {
        status: 201,
        content_type: "application/json".to_string(),
        body: serde_json::to_vec_pretty(&serde_json::json!({
            "status": "ok",
            "package": merged_metadata.name,
        }))
        .map_err(|error| PrayError::Manifest(error.to_string()))?,
    })
}

fn ensure_derived_metadata(root: &Path, metadata: &mut RegistryPackageMetadata) -> PrayResult<()> {
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

fn read_registry_package_metadata(
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

fn write_registry_package_metadata(
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

fn registry_metadata_path(root: &Path, package_name: &str) -> PathBuf {
    root.join("v1/packages")
        .join(package_name)
        .with_extension("json")
}

fn read_known_peers(root: &Path) -> PrayResult<Vec<PeerInfo>> {
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

fn latest_publish_timestamp(metadata: &RegistryPackageMetadata) -> Option<u64> {
    metadata
        .versions
        .iter()
        .filter_map(|version| version.published_at.as_deref())
        .filter_map(|published_at| published_at.parse::<u64>().ok())
        .max()
}

pub(crate) fn transport_package_metadata(
    metadata: &RegistryPackageMetadata,
) -> TransportPackageMetadata {
    let versions = metadata
        .versions
        .iter()
        .map(transport_package_version)
        .collect();
    TransportPackageMetadata {
        name: metadata.name.clone(),
        versions,
        updated_at: latest_publish_timestamp(metadata)
            .map(|timestamp| timestamp.to_string())
            .unwrap_or_else(|| "0".to_string()),
    }
}

pub(crate) fn transport_package_version(version: &RegistryPackageVersion) -> PackageVersion {
    let published_at = version
        .published_at
        .clone()
        .unwrap_or_else(|| "0".to_string());
    let publisher = match (
        version
            .signer_fingerprint
            .as_deref()
            .filter(|value| pray_core::ssh_identity::looks_like_ssh_fingerprint(value)),
        version.signer.as_deref(),
    ) {
        (Some(fingerprint), Some(label)) => Some(PublisherInfo {
            id: label.to_string(),
            key_fingerprint: pray_core::ssh_identity::normalize_identity(fingerprint),
        }),
        (_, Some(signer)) => Some(PublisherInfo {
            id: signer.to_string(),
            key_fingerprint: signer.to_string(),
        }),
        _ => None,
    };
    let signature = version.signature.as_ref().map(|signature| SignatureInfo {
        algorithm: "sha256".to_string(),
        signature: signature.clone(),
        public_key: registry_package_signing_identity(version).unwrap_or_default(),
    });
    let origin = version
        .published_at
        .as_ref()
        .map(|published_at| OriginInfo {
            server: "local".to_string(),
            first_seen: published_at.clone(),
        });
    PackageVersion {
        version: version.version.clone(),
        artifact: version.artifact.clone(),
        artifact_hash: version.artifact_hash.clone().unwrap_or_default(),
        tree_hash: version.tree_hash.clone().unwrap_or_default(),
        yanked: version.yanked,
        targets: version.targets.clone(),
        exports: version.exports.clone(),
        published_at,
        publisher,
        signature,
        origin,
        derived_metadata: version.derived_metadata.clone(),
    }
}

fn registry_package_metadata_from_transport(
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

fn registry_package_version_from_transport(
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
        published_at,
        signature,
        derived_metadata: version.derived_metadata.clone(),
    })
}

fn merge_registry_package_metadata(
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

fn read_or_create_registry_package_metadata(
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

fn update_registry_index_with_package(root: &Path, package_name: &str) -> PrayResult<()> {
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

fn html_package_response(root: &Path, path: &str) -> PrayResult<Response> {
    let package_name = path.trim_start_matches("/packages/");
    let metadata_path = root
        .join("v1/packages")
        .join(package_name)
        .with_extension("json");
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
    let confessions = read_confessions(root, package_name)?;
    let body = render_package_page(package_name, &metadata, &confessions);
    Ok(Response {
        status: 200,
        content_type: "text/html; charset=utf-8".to_string(),
        body: body.into_bytes(),
    })
}

fn render_package_page(
    package_name: &str,
    metadata: &RegistryPackageMetadata,
    confessions: &[ConfessionSubmission],
) -> String {
    let mut versions = String::new();
    for version in &metadata.versions {
        let mut details = String::new();
        if let Some(signer) = version.signer.as_ref() {
            details.push_str(&format!("<div>Signer: {}</div>", html_escape(signer)));
        }
        if let Some(signature) = version.signature.as_ref() {
            details.push_str(&format!("<div>Signature: {}</div>", html_escape(signature)));
        }
        if let Some(published_at) = version.published_at.as_ref() {
            details.push_str(&format!(
                "<div>Published at: {}</div>",
                html_escape(published_at)
            ));
        }
        versions.push_str(&format!(
            "<li><a href=\"/{artifact}\">{version}</a>{details}</li>",
            artifact = html_escape(&version.artifact),
            version = html_escape(&version.version),
            details = details,
        ));
    }
    let accepted = confessions
        .iter()
        .filter(|entry| entry.status == "accepted")
        .count();
    let rejected = confessions
        .iter()
        .filter(|entry| entry.status == "rejected")
        .count();
    let mut confession_items = String::new();
    for confession in confessions {
        confession_items.push_str(&format!(
            "<li><strong>{}</strong> {}{}</li>",
            html_escape(confession.status.as_str()),
            html_escape(confession.version.as_str()),
            confession
                .note
                .as_ref()
                .map(|note| format!(": {}", html_escape(note)))
                .unwrap_or_default()
        ));
    }
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{package}</title></head><body><h1>{package}</h1><p>Accepted: {accepted}</p><p>Rejected: {rejected}</p><h2>Versions</h2><ul>{versions}</ul><h2>Confessions</h2><ul>{confession_items}</ul></body></html>",
        package = html_escape(package_name),
        accepted = accepted,
        rejected = rejected,
        versions = versions,
        confession_items = confession_items,
    )
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

fn artifact_upload_response(root: &Path, path: &str, body: &[u8]) -> PrayResult<Response> {
    if std::env::var("PRAY_SERVE_STDIO").is_ok() {
        authorize_ssh_push(root)?;
    }
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

fn auth_register_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthRegistrationRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth registration",
            message: error.to_string(),
        })?;
    let trust = read_registry_trust_settings(root)?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthRegistrationResponse =
        store.register_email(&request.email, trust.email_confirmation)?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 201,
        content_type: "application/json".to_string(),
        body,
    })
}

fn auth_verify_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthVerificationRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth verification",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthVerificationResponse = store.verify_email(&request.email, &request.code)?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body,
    })
}

fn auth_session_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthSessionRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth session",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthSessionResponse =
        store.issue_session(&request.email, AuthSessionKind::Email)?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body,
    })
}

fn auth_passkey_enroll_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthPasskeyEnrollmentRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth passkey enrollment",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthPasskeyEnrollmentResponse = store.enroll_passkey(
        &request.email,
        &request.credential_id,
        &request.public_key,
        request.label.as_deref(),
    )?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 201,
        content_type: "application/json".to_string(),
        body,
    })
}

fn auth_passkey_challenge_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthPasskeyChallengeRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth passkey challenge",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthPasskeyChallengeResponse =
        store.request_passkey_challenge(&request.credential_id)?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body,
    })
}

fn auth_passkey_login_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthPasskeyLoginRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth passkey login",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthPasskeyLoginResponse = store.respond_passkey_challenge(
        &request.credential_id,
        &request.challenge_id,
        &request.signature,
    )?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body,
    })
}

fn auth_ssh_key_challenge_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthSshKeyChallengeRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth ssh key challenge",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthSshKeyChallengeResponse =
        store.request_ssh_key_challenge(&request.public_key)?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body,
    })
}

fn auth_ssh_key_enroll_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthSshKeyEnrollmentRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth ssh key enrollment",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthSshKeyEnrollmentResponse = store.enroll_ssh_key(
        &request.email,
        &request.public_key,
        request.label.as_deref(),
    )?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 201,
        content_type: "application/json".to_string(),
        body,
    })
}

fn auth_ssh_key_login_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthSshKeyLoginRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth ssh key login",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthSshKeyLoginResponse = store.respond_ssh_key_challenge(
        &request.public_key,
        &request.challenge_id,
        &request.signature,
    )?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body,
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

fn read_registry_index(root: &Path) -> PrayResult<RegistryIndex> {
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

fn read_confessions(root: &Path, package_name: &str) -> PrayResult<Vec<ConfessionSubmission>> {
    let path = root.join("v1/confessions.jsonl");
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(Vec::new());
    };
    let mut confessions = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let confession: ConfessionSubmission =
            serde_json::from_str(line).map_err(|error| PrayError::Parse {
                kind: "confession",
                message: error.to_string(),
            })?;
        if confession.package == package_name {
            confessions.push(confession);
        }
    }
    Ok(confessions)
}

fn write_response(
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

fn response_with_status(status: u16, content_type: &str, body: Vec<u8>) -> Response {
    Response {
        status,
        content_type: content_type.to_string(),
        body,
    }
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

fn strip_query(path: &str) -> &str {
    path.split_once('?').map(|(path, _)| path).unwrap_or(path)
}

fn query_parameter(path: &str, name: &str) -> Option<String> {
    let query = path.split_once('?')?.1;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key == name {
            return Some(value.to_string());
        }
    }
    None
}

fn sanitize_request_path(path: &str) -> PrayResult<PathBuf> {
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

fn content_type_for_path(path: &Path) -> String {
    match path.extension().and_then(|value| value.to_str()) {
        Some("json") => "application/json".to_string(),
        Some("jsonl") => "application/x-ndjson".to_string(),
        Some("md") | Some("txt") => "text/plain; charset=utf-8".to_string(),
        Some("html") => "text/html; charset=utf-8".to_string(),
        Some("praypkg") => "application/octet-stream".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn handle_rpc(root: &Path, request: &RpcRequest) -> PrayResult<RpcResponse> {
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
            artifact_upload_response(root, &format!("/{path}"), &body)?
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

fn decode_rpc_base64_body(value: Option<&serde_json::Value>) -> PrayResult<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let encoded = value
        .and_then(|value| value.as_str())
        .ok_or_else(|| PrayError::Resolution("artifact.put requires base64 body".to_string()))?;
    STANDARD.decode(encoded).map_err(|error| {
        PrayError::Resolution(format!("artifact.put body base64 decode failed: {error}"))
    })
}

fn http_response_to_rpc(id: &str, response: Response) -> RpcResponse {
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
    } else if response
        .content_type
        .starts_with("application/octet-stream")
        || response.content_type.starts_with("text/")
    {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        RpcResponse {
            spec: SSH_RPC_SPEC.to_string(),
            id: id.to_string(),
            status: response.status,
            content_type: response.content_type,
            body_encoding: Some("base64".to_string()),
            body: serde_json::Value::String(STANDARD.encode(&response.body)),
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

#[cfg(test)]
mod http_rpc_bridge_tests {
    use super::{
        dispatch_http_request, federation_discovery_response, handle_rpc, http_response_to_rpc,
        rpc_response_to_http,
    };
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
        let bridged = dispatch_http_request(&root, "GET", "/.well-known/pray-federation.json", &[])
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
            super::response_with_status(200, "application/json", br#"{"ok":true}"#.to_vec());
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

        let rpc = handle_rpc(
            &root,
            &RpcRequest::new("1", "sync.package", json!({ "name": "sample/base" })),
        )
        .expect("rpc response");
        assert_eq!(rpc.spec, SSH_RPC_SPEC);
        assert_eq!(rpc.status, 200);

        let http = dispatch_http_request(&root, "GET", "/v1/sync/package/sample/base", &[])
            .expect("http response");
        assert_eq!(http.status, 200);
        assert_eq!(rpc_response_to_http(&rpc).body, http.body);
        let _ = fs::remove_dir_all(&root);
    }
}
