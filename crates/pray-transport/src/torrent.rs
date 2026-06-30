use crate::http::{HttpConfig, HttpTransport};
use crate::types::*;
use async_trait::async_trait;
use reqwest::{header::RANGE, Client};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use std::time::Duration;

const DEFAULT_PIECE_SIZE: usize = 16 * 1024;
const DEFAULT_METADATA_SUFFIX: &str = ".praytorrent.json";
const TONGUE_TWISTER_SPEC: &str = "pray-torrent-v1";

/// Torrent transport configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentConfig {
    /// Optional tracker endpoints used for announcing content.
    #[serde(default)]
    pub bootstrap_trackers: Vec<String>,

    /// Optional DHT bootstrap nodes used for peer discovery.
    #[serde(default)]
    pub dht_bootstrap_nodes: Vec<String>,

    /// Whether DHT discovery should be attempted.
    #[serde(default)]
    pub enable_dht: bool,

    /// HTTP settings reused for discovery and metadata requests.
    #[serde(default)]
    pub http: HttpConfig,

    /// Artifact piece size used for torrent-style fetching.
    #[serde(default = "default_piece_size")]
    pub piece_size: usize,

    /// Sidecar manifest suffix used to resolve torrent metadata.
    #[serde(default = "default_metadata_suffix")]
    pub metadata_suffix: String,
}

impl Default for TorrentConfig {
    fn default() -> Self {
        Self {
            bootstrap_trackers: Vec::new(),
            dht_bootstrap_nodes: Vec::new(),
            enable_dht: false,
            http: HttpConfig::default(),
            piece_size: default_piece_size(),
            metadata_suffix: default_metadata_suffix(),
        }
    }
}

fn default_piece_size() -> usize {
    DEFAULT_PIECE_SIZE
}

fn default_metadata_suffix() -> String {
    DEFAULT_METADATA_SUFFIX.to_string()
}

/// Torrent-style manifest used for piece-based artifact fetching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TorrentManifest {
    pub spec: String,
    pub name: String,
    pub version: String,
    pub artifact_url: String,
    pub artifact_hash: String,
    pub piece_size: usize,
    pub length: usize,
    pub pieces: Vec<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub trackers: Vec<String>,
}

/// A byte range paired with the expected piece hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PieceRange {
    pub start: usize,
    pub end: usize,
    pub hash: String,
}

/// Torrent transport adapter.
pub struct TorrentTransport {
    config: TorrentConfig,
    http: HttpTransport,
    client: Client,
}

impl TorrentTransport {
    pub fn new(config: TorrentConfig) -> Result<Self> {
        if config.piece_size == 0 {
            return Err(TransportError::InvalidResponse(
                "torrent piece size must be greater than zero".to_string(),
            ));
        }

        let client = build_client(&config.http)?;
        let http = HttpTransport::new(config.http.clone())?;
        Ok(Self {
            config,
            http,
            client,
        })
    }

    pub fn build_manifest(
        name: String,
        version: String,
        artifact_url: String,
        bytes: &[u8],
        piece_size: usize,
        sources: Vec<String>,
        trackers: Vec<String>,
    ) -> TorrentManifest {
        let normalized_piece_size = piece_size.max(1);
        TorrentManifest {
            spec: TONGUE_TWISTER_SPEC.to_string(),
            name,
            version,
            artifact_url,
            artifact_hash: sha256_prefixed(bytes),
            piece_size: normalized_piece_size,
            length: bytes.len(),
            pieces: piece_hashes(bytes, normalized_piece_size),
            sources,
            trackers,
        }
    }

    fn manifest_url(&self, peer: &PeerConfig, artifact: &ArtifactRef) -> Result<String> {
        let artifact_url = resolve_peer_url(peer, &artifact.url)?;
        if artifact_url.ends_with(&self.config.metadata_suffix) {
            return Ok(artifact_url);
        }

        Ok(format!("{}{}", artifact_url, self.config.metadata_suffix))
    }

    fn source_urls(
        &self,
        peer: &PeerConfig,
        artifact: &ArtifactRef,
        manifest: &TorrentManifest,
    ) -> Result<Vec<String>> {
        if !manifest.sources.is_empty() {
            return Ok(manifest.sources.clone());
        }

        Ok(vec![resolve_peer_url(peer, &manifest.artifact_url)
            .or_else(|_| resolve_peer_url(peer, &artifact.url))?])
    }

    async fn fetch_manifest(
        &self,
        peer: &PeerConfig,
        artifact: &ArtifactRef,
    ) -> Result<TorrentManifest> {
        let manifest_url = self.manifest_url(peer, artifact)?;
        let response = self
            .client
            .get(&manifest_url)
            .send()
            .await
            .map_err(|error| TransportError::Network(format!("HTTP request failed: {error}")))?;

        if !response.status().is_success() {
            return Err(TransportError::NotFound(format!(
                "Torrent manifest not found: {}",
                manifest_url
            )));
        }

        let manifest: TorrentManifest = response
            .json()
            .await
            .map_err(|error| TransportError::InvalidResponse(format!("Invalid JSON: {error}")))?;

        manifest.validate_for(artifact)?;
        Ok(manifest)
    }

    async fn download_piece(&self, source_urls: &[String], piece: &PieceRange) -> Result<Vec<u8>> {
        let range_header = format!("bytes={}-{}", piece.start, piece.end);

        for source_url in source_urls {
            let response = self
                .client
                .get(source_url)
                .header(RANGE, range_header.as_str())
                .send()
                .await
                .map_err(|error| {
                    TransportError::Network(format!("HTTP request failed: {error}"))
                })?;

            if !(response.status().is_success() || response.status().as_u16() == 206) {
                continue;
            }

            let bytes = response.bytes().await.map_err(|error| {
                TransportError::Network(format!("Failed to read response: {error}"))
            })?;

            if bytes.len() == piece.length() {
                return Ok(bytes.to_vec());
            }
        }

        Err(TransportError::NotFound(format!(
            "Unable to fetch torrent piece {}-{}",
            piece.start, piece.end
        )))
    }

    fn fallback_discovery(&self) -> FederationInfo {
        let mut peers = Vec::new();
        for url in self
            .config
            .bootstrap_trackers
            .iter()
            .chain(self.config.dht_bootstrap_nodes.iter())
        {
            peers.push(PeerInfo {
                name: url.clone(),
                url: url.clone(),
                public: false,
            });
        }

        FederationInfo {
            spec: TONGUE_TWISTER_SPEC.to_string(),
            server: ServerInfo {
                name: "torrent".to_string(),
                version: "1".to_string(),
                capabilities: vec!["piece_fetch".to_string(), "tracker_hint".to_string()],
            },
            sync: SyncEndpoints {
                index_url: "/v1/sync/index".to_string(),
                package_url: "/v1/sync/package".to_string(),
                artifact_url: "/v1/sync/artifact".to_string(),
                since_param: "since".to_string(),
            },
            peers,
        }
    }
}

#[async_trait]
impl TransportAdapter for TorrentTransport {
    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            pull: true,
            push: true,
            streaming: false,
            binary: true,
            max_message_size: None,
            partial_responses: true,
        }
    }

    fn name(&self) -> &str {
        "torrent"
    }

    async fn fetch_discovery(&self, peer: &PeerConfig) -> Result<FederationInfo> {
        if peer.url.is_none() {
            return Ok(self.fallback_discovery());
        }

        self.http.fetch_discovery(peer).await
    }

    async fn fetch_index(&self, peer: &PeerConfig, since: Option<i64>) -> Result<IndexResponse> {
        self.http.fetch_index(peer, since).await
    }

    async fn fetch_package(&self, peer: &PeerConfig, name: &str) -> Result<PackageMetadata> {
        self.http.fetch_package(peer, name).await
    }

    async fn fetch_artifact(&self, peer: &PeerConfig, artifact: &ArtifactRef) -> Result<Vec<u8>> {
        let manifest = self.fetch_manifest(peer, artifact).await?;
        let source_urls = self.source_urls(peer, artifact, &manifest)?;
        let mut bytes = vec![0u8; manifest.length];

        for piece in manifest.piece_ranges() {
            let piece_bytes = self.download_piece(&source_urls, &piece).await?;
            if sha256_prefixed(&piece_bytes) != piece.hash {
                return Err(TransportError::InvalidResponse(format!(
                    "piece hash mismatch for {}..{}",
                    piece.start, piece.end
                )));
            }
            bytes[piece.start..=piece.end].copy_from_slice(&piece_bytes);
        }

        let computed_hash = sha256_prefixed(&bytes);
        if computed_hash != artifact.hash {
            return Err(TransportError::InvalidResponse(format!(
                "artifact hash mismatch for {} {}",
                artifact.name, artifact.version
            )));
        }

        Ok(bytes)
    }

    async fn push_package(&self, peer: &PeerConfig, metadata: &PackageMetadata) -> Result<()> {
        self.http.push_package(peer, metadata).await
    }
}

/// Factory for creating torrent transport adapters.
pub struct TorrentTransportFactory;

impl TransportAdapterFactory for TorrentTransportFactory {
    fn name(&self) -> &str {
        "torrent"
    }

    fn create(&self, config: &PeerConfig) -> Result<Box<dyn TransportAdapter>> {
        let torrent_config: TorrentConfig =
            serde_json::from_value(config.config.clone()).unwrap_or_default();
        let transport = TorrentTransport::new(torrent_config)?;
        Ok(Box::new(transport))
    }
}

impl TorrentManifest {
    pub fn validate_for(&self, artifact: &ArtifactRef) -> Result<()> {
        if self.spec != TONGUE_TWISTER_SPEC {
            return Err(TransportError::InvalidResponse(format!(
                "unsupported torrent manifest spec: {}",
                self.spec
            )));
        }

        if self.name != artifact.name || self.version != artifact.version {
            return Err(TransportError::InvalidResponse(format!(
                "torrent manifest mismatch for {} {}",
                artifact.name, artifact.version
            )));
        }

        if self.artifact_hash != artifact.hash {
            return Err(TransportError::InvalidResponse(format!(
                "torrent manifest artifact hash mismatch for {} {}",
                artifact.name, artifact.version
            )));
        }

        if self.piece_size == 0 {
            return Err(TransportError::InvalidResponse(
                "torrent manifest piece size must be greater than zero".to_string(),
            ));
        }

        if self.pieces.len() != piece_ranges(self.length, self.piece_size).len() {
            return Err(TransportError::InvalidResponse(
                "torrent manifest piece count does not match length".to_string(),
            ));
        }

        Ok(())
    }

    pub fn piece_ranges(&self) -> Vec<PieceRange> {
        piece_ranges_with_hashes(self.length, self.piece_size, &self.pieces)
    }
}

impl PieceRange {
    pub fn length(&self) -> usize {
        self.end.saturating_sub(self.start) + 1
    }
}

fn build_client(config: &HttpConfig) -> Result<Client> {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(config.timeout_secs))
        .danger_accept_invalid_certs(!config.tls_verify);

    let mut headers = reqwest::header::HeaderMap::new();
    for (key, value) in &config.headers {
        if let (Ok(name), Ok(header_value)) = (
            reqwest::header::HeaderName::try_from(key.as_str()),
            reqwest::header::HeaderValue::from_str(value),
        ) {
            headers.insert(name, header_value);
        }
    }
    builder = builder.default_headers(headers);

    builder.build().map_err(|error| {
        TransportError::Other(anyhow::anyhow!(
            "Failed to create torrent HTTP client: {error}"
        ))
    })
}

fn resolve_peer_url(peer: &PeerConfig, path: &str) -> Result<String> {
    if path.starts_with("http://") || path.starts_with("https://") {
        return Ok(path.to_string());
    }

    let base = peer
        .url
        .as_ref()
        .ok_or_else(|| TransportError::InvalidResponse("Missing URL".to_string()))?;

    Ok(format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    ))
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex_output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        hex_output.push_str(&format!("{byte:02x}"));
    }
    format!("sha256:{hex_output}")
}

fn piece_hashes(bytes: &[u8], piece_size: usize) -> Vec<String> {
    piece_ranges(bytes.len(), piece_size)
        .into_iter()
        .map(|piece| sha256_prefixed(&bytes[piece.start..=piece.end]))
        .collect()
}

fn piece_ranges(length: usize, piece_size: usize) -> Vec<PieceRange> {
    let piece_size = piece_size.max(1);
    let mut ranges = Vec::new();
    let mut start = 0usize;

    while start < length {
        let end = std::cmp::min(start + piece_size, length) - 1;
        ranges.push(PieceRange {
            start,
            end,
            hash: String::new(),
        });
        start = end + 1;
    }

    ranges
}

fn piece_ranges_with_hashes(
    length: usize,
    piece_size: usize,
    hashes: &[String],
) -> Vec<PieceRange> {
    let mut ranges = piece_ranges(length, piece_size);
    for (piece, hash) in ranges.iter_mut().zip(hashes.iter()) {
        piece.hash = hash.clone();
    }
    ranges
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn piece_planning_and_manifest_hashes_are_consistent() {
        let bytes = b"abcdefghij";
        let manifest = TorrentTransport::build_manifest(
            "sample/base".to_string(),
            "1.2.3".to_string(),
            "https://example.test/artifact.bin".to_string(),
            bytes,
            4,
            vec!["https://mirror.example".to_string()],
            vec!["https://tracker.example".to_string()],
        );

        assert_eq!(manifest.spec, TONGUE_TWISTER_SPEC);
        assert_eq!(manifest.length, 10);
        assert_eq!(manifest.pieces.len(), 3);
        assert_eq!(manifest.piece_ranges().len(), 3);
        assert!(manifest
            .pieces
            .iter()
            .all(|piece| piece.starts_with("sha256:")));
    }

    #[tokio::test]
    async fn torrent_transport_fetches_artifact_using_piece_ranges() {
        let artifact_bytes = b"abcdefghij".to_vec();
        let artifact_length = artifact_bytes.len();
        let server_artifact_bytes = artifact_bytes.clone();
        let piece_size = 4usize;
        let artifact_url = "/artifact.bin".to_string();
        let server_artifact_url = artifact_url.clone();
        let manifest_path = format!("{}{}", artifact_url, DEFAULT_METADATA_SUFFIX);
        let manifest = TorrentTransport::build_manifest(
            "sample/base".to_string(),
            "1.2.3".to_string(),
            artifact_url.clone(),
            &artifact_bytes,
            piece_size,
            vec![],
            vec![],
        );
        let manifest_json = serde_json::to_vec(&manifest).expect("manifest json");
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let address = listener.local_addr().expect("listener address");
        let expected_requests = manifest.piece_ranges().len() + 1;

        let server_task = tokio::spawn(async move {
            for _ in 0..expected_requests {
                let (mut socket, _) = listener.accept().await.expect("accepted connection");
                let mut request = Vec::new();
                let mut buffer = [0u8; 1024];
                loop {
                    let read = socket.read(&mut buffer).await.expect("read request");
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..read]);
                    if request.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }

                let request_text = String::from_utf8_lossy(&request);
                let first_line = request_text.lines().next().expect("request line");
                let path = first_line.split_whitespace().nth(1).expect("request path");
                let response = if path == manifest_path {
                    http_response(
                        200,
                        "OK",
                        &[("content-type", "application/json")],
                        &manifest_json,
                    )
                } else if path == server_artifact_url {
                    let range = request_text
                        .lines()
                        .find(|line| line.starts_with("Range: ") || line.starts_with("range: "))
                        .expect("range header");
                    let range = range.split_once(':').expect("range header split").1.trim();
                    let (start, end) = parse_range(range).expect("parse range");
                    let body = server_artifact_bytes[start..=end].to_vec();
                    http_response(
                        206,
                        "Partial Content",
                        &[
                            ("content-type", "application/octet-stream"),
                            (
                                "content-range",
                                &format!("bytes {}-{}/{}", start, end, artifact_length),
                            ),
                        ],
                        &body,
                    )
                } else {
                    http_response(
                        404,
                        "Not Found",
                        &[("content-type", "text/plain")],
                        b"not found",
                    )
                };

                socket
                    .write_all(response.as_bytes())
                    .await
                    .expect("write response");
                socket.shutdown().await.expect("shutdown socket");
            }
        });

        let transport = TorrentTransport::new(TorrentConfig {
            piece_size,
            http: HttpConfig {
                timeout_secs: 5,
                headers: std::collections::HashMap::new(),
                tls_verify: true,
            },
            ..TorrentConfig::default()
        })
        .expect("transport");

        let peer = PeerConfig {
            name: "peer-a".to_string(),
            transport: "torrent".to_string(),
            url: Some(format!("http://{}", address)),
            trust: TrustLevel::Full,
            direction: SyncDirection::Pull,
            config: serde_json::json!({}),
        };
        let artifact = ArtifactRef {
            name: "sample/base".to_string(),
            version: "1.2.3".to_string(),
            url: artifact_url,
            hash: sha256_prefixed(&artifact_bytes),
        };

        let fetched = transport
            .fetch_artifact(&peer, &artifact)
            .await
            .expect("fetch artifact");

        assert_eq!(fetched, artifact_bytes);
        server_task.await.expect("server task");
    }

    #[tokio::test]
    async fn torrent_transport_falls_back_to_tracker_peers_when_no_peer_url_exists() {
        let transport = TorrentTransport::new(TorrentConfig {
            bootstrap_trackers: vec!["https://tracker.example".to_string()],
            dht_bootstrap_nodes: vec!["https://dht.example".to_string()],
            enable_dht: true,
            ..TorrentConfig::default()
        })
        .expect("transport");

        let discovery = transport
            .fetch_discovery(&PeerConfig {
                name: "peer-a".to_string(),
                transport: "torrent".to_string(),
                url: None,
                trust: TrustLevel::Full,
                direction: SyncDirection::Pull,
                config: serde_json::json!({}),
            })
            .await
            .expect("fallback discovery");

        assert_eq!(discovery.spec, TONGUE_TWISTER_SPEC);
        assert_eq!(discovery.peers.len(), 2);
    }

    fn http_response(
        status_code: u16,
        reason: &str,
        headers: &[(&str, &str)],
        body: &[u8],
    ) -> String {
        let mut response = format!(
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n",
            status_code,
            reason,
            body.len()
        );
        for (key, value) in headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        response.push_str("\r\n");
        response.push_str(&String::from_utf8_lossy(body));
        response
    }

    fn parse_range(header: &str) -> std::result::Result<(usize, usize), std::io::Error> {
        let range = header
            .strip_prefix("bytes=")
            .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidInput, "missing bytes prefix"))?;
        let (start, end) = range
            .split_once('-')
            .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidInput, "missing range dash"))?;
        Ok((
            start
                .parse()
                .map_err(|_| std::io::Error::new(ErrorKind::InvalidInput, "invalid range start"))?,
            end.parse()
                .map_err(|_| std::io::Error::new(ErrorKind::InvalidInput, "invalid range end"))?,
        ))
    }
}
