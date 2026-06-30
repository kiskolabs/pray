use crate::derived_metadata::RegistryDerivedMetadata;
use crate::hashing::sha256_prefixed;
use crate::manifest::ManifestPackage;
use crate::package_spec::parse_package_spec;
use crate::{PrayError, PrayResult};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

const TORRENT_MANIFEST_SPEC: &str = "pray-torrent-v1";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryIndex {
    pub spec: String,
    #[serde(default)]
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryPackageMetadata {
    pub name: String,
    #[serde(default)]
    pub versions: Vec<RegistryPackageVersion>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryPackageVersion {
    pub version: String,
    pub artifact: String,
    #[serde(default)]
    pub artifact_hash: Option<String>,
    #[serde(default)]
    pub tree_hash: Option<String>,
    #[serde(default)]
    pub yanked: bool,
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default)]
    pub exports: Vec<String>,
    #[serde(default)]
    pub signer: Option<String>,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived_metadata: Option<RegistryDerivedMetadata>,
}

impl RegistryPackageVersion {
    pub fn same_identity(&self, other: &Self) -> bool {
        self.version == other.version
            && self.artifact == other.artifact
            && self.artifact_hash == other.artifact_hash
            && self.tree_hash == other.tree_hash
            && self.yanked == other.yanked
            && self.targets == other.targets
            && self.exports == other.exports
            && self.signer == other.signer
            && self.published_at == other.published_at
            && self.signature == other.signature
    }

    pub fn merge_annotations_from(&mut self, other: &Self) {
        if self.derived_metadata.is_none() {
            self.derived_metadata = other.derived_metadata.clone();
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfessionSubmission {
    pub package: String,
    pub version: String,
    pub status: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub lockfile: Option<String>,
    #[serde(default)]
    pub distribution_point: Option<String>,
    #[serde(default)]
    pub signer: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct TorrentManifest {
    spec: String,
    name: String,
    version: String,
    artifact_url: String,
    artifact_hash: String,
    piece_size: usize,
    length: usize,
    pieces: Vec<String>,
    #[serde(default)]
    sources: Vec<String>,
    #[serde(default)]
    trackers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TorrentPieceRange {
    start: usize,
    end: usize,
    hash: String,
}

impl TorrentManifest {
    fn validate(&self) -> PrayResult<()> {
        if self.spec != TORRENT_MANIFEST_SPEC {
            return Err(PrayError::Parse {
                kind: "torrent manifest",
                message: format!("unexpected spec: {}", self.spec),
            });
        }
        if self.piece_size == 0 {
            return Err(PrayError::Parse {
                kind: "torrent manifest",
                message: "piece size must be greater than zero".to_string(),
            });
        }
        let expected_piece_count = if self.length == 0 {
            0
        } else {
            self.length.div_ceil(self.piece_size)
        };
        if self.pieces.len() != expected_piece_count {
            return Err(PrayError::Parse {
                kind: "torrent manifest",
                message: format!(
                    "expected {} piece hash(es), found {}",
                    expected_piece_count,
                    self.pieces.len()
                ),
            });
        }
        Ok(())
    }

    fn piece_ranges(&self) -> Vec<TorrentPieceRange> {
        self.pieces
            .iter()
            .enumerate()
            .map(|(index, hash)| {
                let start = index * self.piece_size;
                let end = self
                    .length
                    .saturating_sub(1)
                    .min(start + self.piece_size - 1);
                TorrentPieceRange {
                    start,
                    end,
                    hash: hash.clone(),
                }
            })
            .collect()
    }
}

impl TorrentPieceRange {
    fn length(&self) -> usize {
        self.end.saturating_sub(self.start) + 1
    }
}

pub fn resolve_registry_package_root(
    project_root: &Path,
    source_url: &str,
    declaration: &ManifestPackage,
) -> PrayResult<PathBuf> {
    let metadata = fetch_registry_package_metadata(source_url, &declaration.name)?;
    let selected = select_package_version(&metadata, &declaration.constraint)?;
    let cache_directory = registry_cache_directory(
        project_root,
        source_url,
        &declaration.name,
        &selected.version,
    );

    if find_prayspec_file(&cache_directory).is_ok() {
        return Ok(cache_directory);
    }

    if cache_directory.exists() {
        remove_path_if_exists(&cache_directory)?;
    }
    fs::create_dir_all(&cache_directory)?;

    let artifact_url = join_url(source_url, &selected.artifact);
    let torrent_manifest = fetch_torrent_manifest(source_url, &selected.artifact)?;
    let artifact_bytes = if let Some(manifest) = torrent_manifest {
        fetch_torrent_artifact(source_url, &selected.artifact, &manifest)?
    } else {
        http_get(&artifact_url)?
    };
    validate_and_unpack_registry_package(
        &cache_directory,
        declaration,
        &selected,
        &artifact_bytes,
    )?;

    Ok(cache_directory)
}

pub fn resolve_local_registry_package_root(
    project_root: &Path,
    source_key: &str,
    source_root: &Path,
    declaration: &ManifestPackage,
) -> PrayResult<PathBuf> {
    let metadata_path = source_root.join(format!("v1/packages/{}.json", declaration.name));
    let metadata_text = fs::read_to_string(&metadata_path)?;
    let metadata: RegistryPackageMetadata =
        serde_json::from_str(&metadata_text).map_err(|error| PrayError::Parse {
            kind: "registry metadata",
            message: error.to_string(),
        })?;
    let selected = select_package_version(&metadata, &declaration.constraint)?;
    let cache_identifier = format!(
        "{}:{}:{}:{}",
        source_key,
        declaration.name,
        selected.version,
        selected
            .artifact_hash
            .as_deref()
            .unwrap_or("no-artifact-hash")
    );
    let cache_directory = registry_cache_directory(
        project_root,
        &cache_identifier,
        &declaration.name,
        &selected.version,
    );

    if find_prayspec_file(&cache_directory).is_ok() {
        return Ok(cache_directory);
    }

    if cache_directory.exists() {
        remove_path_if_exists(&cache_directory)?;
    }
    fs::create_dir_all(&cache_directory)?;

    let artifact_bytes = read_local_registry_artifact_bytes(source_root, &selected.artifact)?;
    validate_and_unpack_registry_package(
        &cache_directory,
        declaration,
        &selected,
        &artifact_bytes,
    )?;

    Ok(cache_directory)
}

pub fn registry_artifact_signature(artifact_bytes: &[u8], tree_hash: &str, signer: &str) -> String {
    let mut payload = Vec::with_capacity(artifact_bytes.len() + tree_hash.len() + signer.len() + 2);
    payload.extend_from_slice(artifact_bytes);
    payload.push(0);
    payload.extend_from_slice(tree_hash.as_bytes());
    payload.push(0);
    payload.extend_from_slice(signer.as_bytes());
    sha256_prefixed(&payload)
}

pub fn submit_confession(source_url: &str, confession: &ConfessionSubmission) -> PrayResult<()> {
    let endpoint = join_url(source_url, "v1/confessions");
    let payload =
        serde_json::to_vec(confession).map_err(|error| PrayError::Manifest(error.to_string()))?;
    let response = http_post(&endpoint, "application/json", &payload)?;
    if response.status / 100 != 2 {
        return Err(PrayError::Resolution(format!(
            "confession submission failed with HTTP {}",
            response.status
        )));
    }
    Ok(())
}

pub fn upload_registry_artifact(
    source_url: &str,
    artifact_path: &str,
    bytes: &[u8],
) -> PrayResult<()> {
    let endpoint = join_url(source_url, artifact_path);
    let response = http_put(&endpoint, "application/octet-stream", bytes)?;
    if response.status / 100 != 2 {
        return Err(PrayError::Resolution(format!(
            "artifact upload failed with HTTP {}",
            response.status
        )));
    }
    Ok(())
}

fn fetch_registry_package_metadata(
    source_url: &str,
    package_name: &str,
) -> PrayResult<RegistryPackageMetadata> {
    let url = join_url(source_url, &format!("v1/packages/{}.json", package_name));
    let response = http_get(&url)?;
    serde_json::from_slice(&response).map_err(|error| PrayError::Parse {
        kind: "registry metadata",
        message: error.to_string(),
    })
}

fn fetch_torrent_manifest(
    source_url: &str,
    artifact_path: &str,
) -> PrayResult<Option<TorrentManifest>> {
    let url = join_url(source_url, &format!("{}.praytorrent.json", artifact_path));
    match http_get(&url) {
        Ok(response) => {
            let manifest: TorrentManifest =
                serde_json::from_slice(&response).map_err(|error| PrayError::Parse {
                    kind: "torrent manifest",
                    message: error.to_string(),
                })?;
            manifest.validate()?;
            Ok(Some(manifest))
        }
        Err(PrayError::Resolution(message)) if message.contains("HTTP 404") => Ok(None),
        Err(error) => Err(error),
    }
}

fn fetch_torrent_artifact(
    source_url: &str,
    artifact_path: &str,
    manifest: &TorrentManifest,
) -> PrayResult<Vec<u8>> {
    let artifact_url = if manifest.artifact_url.starts_with("http://")
        || manifest.artifact_url.starts_with("https://")
    {
        manifest.artifact_url.clone()
    } else {
        join_url(source_url, &manifest.artifact_url)
    };

    let sources = if manifest.sources.is_empty() {
        vec![artifact_url]
    } else {
        manifest
            .sources
            .iter()
            .map(|source| {
                if source.starts_with("http://") || source.starts_with("https://") {
                    source.clone()
                } else {
                    join_url(source_url, source)
                }
            })
            .collect()
    };

    let mut bytes = vec![0u8; manifest.length];
    for piece in manifest.piece_ranges() {
        let piece_bytes = download_torrent_piece(&sources, &piece)?;
        if sha256_prefixed(&piece_bytes) != piece.hash {
            return Err(PrayError::Integrity(format!(
                "torrent piece hash mismatch for {artifact_path} {}..{}",
                piece.start, piece.end
            )));
        }
        bytes[piece.start..=piece.end].copy_from_slice(&piece_bytes);
    }

    if sha256_prefixed(&bytes) != manifest.artifact_hash {
        return Err(PrayError::Integrity(format!(
            "torrent artifact hash mismatch for {artifact_path}"
        )));
    }

    Ok(bytes)
}

fn download_torrent_piece(sources: &[String], piece: &TorrentPieceRange) -> PrayResult<Vec<u8>> {
    let range_header = format!("bytes={}-{}", piece.start, piece.end);
    for source in sources {
        match http_get_with_headers(source, &[("Range", &range_header)]) {
            Ok((response, _status)) if response.len() == piece.length() => return Ok(response),
            Ok(_) => continue,
            Err(_) => continue,
        }
    }

    Err(PrayError::Resolution(format!(
        "unable to download torrent piece {}-{}",
        piece.start, piece.end
    )))
}

fn validate_and_unpack_registry_package(
    cache_directory: &Path,
    declaration: &ManifestPackage,
    selected: &RegistryPackageVersion,
    artifact_bytes: &[u8],
) -> PrayResult<()> {
    if let Some(expected_artifact_hash) = selected.artifact_hash.as_deref() {
        let artifact_hash = sha256_prefixed(artifact_bytes);
        if artifact_hash != expected_artifact_hash {
            return Err(PrayError::Integrity(format!(
                "package artifact hash mismatch for {} {}",
                declaration.name, selected.version
            )));
        }
    }
    unpack_praypkg(artifact_bytes, cache_directory)?;

    let spec_path = find_prayspec_file(cache_directory)?;
    let spec_text = fs::read_to_string(&spec_path)?;
    let spec = parse_package_spec(&spec_text)?.canonicalized();

    if spec.name != declaration.name {
        return Err(PrayError::Resolution(format!(
            "package path {:?} declares {:?}, expected {:?}",
            cache_directory, spec.name, declaration.name
        )));
    }
    if spec.version != selected.version {
        return Err(PrayError::Resolution(format!(
            "package {} version {} does not match registry version {}",
            declaration.name, spec.version, selected.version
        )));
    }

    let tree_hash = spec.tree_hash_for_root(cache_directory)?;
    if let Some(expected_tree_hash) = selected.tree_hash.as_deref() {
        if tree_hash != expected_tree_hash {
            return Err(PrayError::Integrity(format!(
                "package tree hash mismatch for {} {}",
                declaration.name, selected.version
            )));
        }
    }

    if let Some(expected_signature) = selected.signature.as_deref() {
        let signer = selected.signer.as_deref().ok_or_else(|| {
            PrayError::Integrity(format!(
                "package signature missing signer for {} {}",
                declaration.name, selected.version
            ))
        })?;
        let actual_signature = registry_artifact_signature(artifact_bytes, &tree_hash, signer);
        if actual_signature != expected_signature {
            return Err(PrayError::Integrity(format!(
                "package signature mismatch for {} {}",
                declaration.name, selected.version
            )));
        }
    }

    Ok(())
}

fn read_local_registry_artifact_bytes(source_root: &Path, artifact: &str) -> PrayResult<Vec<u8>> {
    if artifact.starts_with("http://") || artifact.starts_with("https://") {
        return http_get(artifact);
    }
    if let Some(path) = artifact.strip_prefix("file://") {
        return fs::read(Path::new(path)).map_err(Into::into);
    }
    let artifact_path = Path::new(artifact);
    validate_package_relative_path(artifact_path)?;
    fs::read(source_root.join(artifact_path)).map_err(Into::into)
}

fn select_package_version(
    metadata: &RegistryPackageMetadata,
    constraint: &str,
) -> PrayResult<RegistryPackageVersion> {
    let mut selected: Option<RegistryPackageVersion> = None;
    for version in &metadata.versions {
        if version.yanked {
            continue;
        }
        if !version_satisfies(&version.version, constraint)? {
            continue;
        }
        match &selected {
            Some(existing) if compare_versions(&version.version, &existing.version)? <= 0 => {}
            _ => selected = Some(version.clone()),
        }
    }
    selected.ok_or_else(|| {
        PrayError::Resolution(format!(
            "no registry version for {} satisfies {}",
            metadata.name, constraint
        ))
    })
}

fn version_satisfies(version: &str, constraint: &str) -> PrayResult<bool> {
    if constraint.trim().is_empty() || constraint.trim() == "*" {
        return Ok(true);
    }
    let version =
        Version::parse(version).map_err(|error| PrayError::Resolution(error.to_string()))?;
    let req = if constraint.trim_start().starts_with("~>") {
        VersionReq::parse(&ruby_pessimistic_to_semver(constraint)?)
            .map_err(|error| PrayError::Resolution(error.to_string()))?
    } else {
        VersionReq::parse(constraint.trim())
            .map_err(|error| PrayError::Resolution(error.to_string()))?
    };
    Ok(req.matches(&version))
}

fn compare_versions(left: &str, right: &str) -> PrayResult<i32> {
    let left = Version::parse(left).map_err(|error| PrayError::Resolution(error.to_string()))?;
    let right = Version::parse(right).map_err(|error| PrayError::Resolution(error.to_string()))?;
    Ok(match left.cmp(&right) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    })
}

fn ruby_pessimistic_to_semver(constraint: &str) -> PrayResult<String> {
    let text = constraint.trim().trim_start_matches("~>").trim();
    let parts: Vec<&str> = text.split('.').collect();
    if parts.is_empty() || parts.len() > 3 {
        return Err(PrayError::Resolution(format!(
            "unsupported Ruby pessimistic constraint: {constraint}"
        )));
    }
    let mut numbers = [0u64; 3];
    for (index, part) in parts.iter().enumerate() {
        numbers[index] = part
            .parse::<u64>()
            .map_err(|error| PrayError::Resolution(error.to_string()))?;
    }
    let lower = format!("{}.{}.{}", numbers[0], numbers[1], numbers[2]);
    let upper = match parts.len() {
        1 => format!("{}.0.0", numbers[0] + 1),
        2 => format!("{}.{}.0", numbers[0], numbers[1] + 1),
        _ => format!("{}.{}.0", numbers[0], numbers[1] + 1),
    };
    Ok(format!(">={}, <{}", lower, upper))
}

fn registry_cache_directory(
    project_root: &Path,
    source_url: &str,
    package_name: &str,
    version: &str,
) -> PathBuf {
    let source_key = sha256_prefixed(source_url.as_bytes())
        .trim_start_matches("sha256:")
        .chars()
        .take(16)
        .collect::<String>();
    project_root
        .join(".pray/cache/registry")
        .join(source_key)
        .join(package_name)
        .join(version)
}

fn unpack_praypkg(artifact_bytes: &[u8], output_directory: &Path) -> PrayResult<()> {
    let cursor = std::io::Cursor::new(artifact_bytes);
    let decoder = zstd::stream::read::Decoder::new(cursor)
        .map_err(|error| PrayError::Integrity(error.to_string()))?;
    let mut archive = tar::Archive::new(decoder);
    let mut written_paths = BTreeSet::new();

    for entry in archive
        .entries()
        .map_err(|error| PrayError::Integrity(error.to_string()))?
    {
        let mut entry = entry.map_err(|error| PrayError::Integrity(error.to_string()))?;
        let entry_type = entry.header().entry_type();
        if entry_type.is_dir() {
            continue;
        }
        if entry_type.is_symlink() || entry_type.is_hard_link() || !entry_type.is_file() {
            return Err(PrayError::Integrity(
                "unsupported package archive entry type".to_string(),
            ));
        }
        let path = entry
            .path()
            .map_err(|error| PrayError::Integrity(error.to_string()))?
            .into_owned();
        validate_package_relative_path(&path)?;
        if !written_paths.insert(path.clone()) {
            return Err(PrayError::Integrity(format!(
                "duplicate package archive path: {}",
                path.display()
            )));
        }
        let destination = output_directory.join(&path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut destination_file = fs::File::create(&destination)?;
        std::io::copy(&mut entry, &mut destination_file)
            .map_err(|error| PrayError::Integrity(error.to_string()))?;
    }
    Ok(())
}

fn find_prayspec_file(root: &Path) -> PrayResult<PathBuf> {
    let mut prayspec_files = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("prayspec") {
            prayspec_files.push(path);
        }
    }
    match prayspec_files.len() {
        1 => Ok(prayspec_files.remove(0)),
        0 => Err(PrayError::Resolution(format!(
            "no prayspec file found in {:?}",
            root
        ))),
        _ => Err(PrayError::Resolution(format!(
            "multiple prayspec files found in {:?}",
            root
        ))),
    }
}

fn validate_package_relative_path(path: &Path) -> PrayResult<()> {
    if path.is_absolute() {
        return Err(PrayError::Integrity(format!(
            "package file path must be relative: {}",
            path.display()
        )));
    }
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(PrayError::Integrity(format!(
                "package file path may not traverse upward: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn remove_path_if_exists(path: &Path) -> PrayResult<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => {
            fs::remove_dir_all(path)?;
            Ok(())
        }
        Ok(_) => {
            fs::remove_file(path)?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn join_url(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

struct HttpResponse {
    status: u16,
    #[allow(dead_code)]
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn http_get(url: &str) -> PrayResult<Vec<u8>> {
    let response = http_request("GET", url, None, None, &[])?;
    if response.status / 100 != 2 {
        return Err(PrayError::Resolution(format!(
            "GET {url} failed with HTTP {}",
            response.status
        )));
    }
    Ok(response.body)
}

fn http_get_with_headers(url: &str, headers: &[(&str, &str)]) -> PrayResult<(Vec<u8>, u16)> {
    let response = http_request("GET", url, None, None, headers)?;
    Ok((response.body, response.status))
}

fn http_post(url: &str, content_type: &str, body: &[u8]) -> PrayResult<HttpResponse> {
    http_request("POST", url, Some(content_type), Some(body), &[])
}

fn http_put(url: &str, content_type: &str, body: &[u8]) -> PrayResult<HttpResponse> {
    http_request("PUT", url, Some(content_type), Some(body), &[])
}

fn http_request(
    method: &str,
    url: &str,
    content_type: Option<&str>,
    body: Option<&[u8]>,
    headers: &[(&str, &str)],
) -> PrayResult<HttpResponse> {
    let (host, port, path) = parse_http_url(url)?;
    let mut stream = TcpStream::connect((host.as_str(), port))?;
    let body = body.unwrap_or(&[]);
    let mut request =
        format!("{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n");
    if let Some(content_type) = content_type {
        request.push_str(&format!("Content-Type: {content_type}\r\n"));
    }
    for (name, value) in headers {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    if !body.is_empty() {
        request.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    request.push_str("\r\n");
    stream.write_all(request.as_bytes())?;
    if !body.is_empty() {
        stream.write_all(body)?;
    }
    stream.flush()?;

    let mut response_bytes = Vec::new();
    stream.read_to_end(&mut response_bytes)?;
    parse_http_response(&response_bytes)
}

fn parse_http_url(url: &str) -> PrayResult<(String, u16, String)> {
    let without_scheme = url
        .strip_prefix("http://")
        .ok_or_else(|| PrayError::Unsupported(format!("unsupported URL scheme: {url}")))?;
    let (host_port, path) = if let Some((host_port, path)) = without_scheme.split_once('/') {
        (host_port, format!("/{}", path))
    } else {
        (without_scheme, "/".to_string())
    };
    let (host, port) = if let Some((host, port)) = host_port.rsplit_once(':') {
        (
            host.to_string(),
            port.parse::<u16>()
                .map_err(|error| PrayError::Resolution(error.to_string()))?,
        )
    } else {
        (host_port.to_string(), 80)
    };
    if host.is_empty() {
        return Err(PrayError::Resolution(format!("invalid URL: {url}")));
    }
    Ok((host, port, path))
}

fn parse_http_response(response_bytes: &[u8]) -> PrayResult<HttpResponse> {
    let header_end = response_bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| PrayError::Resolution("malformed HTTP response".to_string()))?;
    let header_text = std::str::from_utf8(&response_bytes[..header_end])
        .map_err(|error| PrayError::Resolution(error.to_string()))?;
    let mut lines = header_text.lines();
    let status_line = lines
        .next()
        .ok_or_else(|| PrayError::Resolution("missing HTTP status line".to_string()))?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| PrayError::Resolution("missing HTTP status code".to_string()))?
        .parse::<u16>()
        .map_err(|error| PrayError::Resolution(error.to_string()))?;
    let mut headers = Vec::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_string(), value.trim().to_string()));
        }
    }
    let body = response_bytes[header_end + 4..].to_vec();
    Ok(HttpResponse {
        status,
        headers,
        body,
    })
}
