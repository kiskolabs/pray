use crate::http::{HttpConfig, HttpTransport};
use crate::types::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
}

impl TorrentTransport {
    pub fn new(config: TorrentConfig) -> Result<Self> {
        if config.piece_size == 0 {
            return Err(TransportError::InvalidResponse(
                "torrent piece size must be greater than zero".to_string(),
            ));
        }

        let http = HttpTransport::new(config.http.clone())?;
        Ok(Self { config, http })
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
        let base = peer
            .url
            .clone()
            .ok_or_else(|| TransportError::InvalidResponse("Missing URL".to_string()))?;
        let artifact_path = artifact.url.clone();
        let expected_hash = artifact.hash.clone();
        let artifact_name = artifact.name.clone();
        let artifact_version = artifact.version.clone();
        let bytes = tokio::task::spawn_blocking(move || {
            pray_core::fetch::download_registry_artifact(&base, &artifact_path)
                .map_err(|error| TransportError::Network(error.to_string()))
        })
        .await
        .map_err(|error| TransportError::Network(format!("fetch join failed: {error}")))??;
        let computed_hash = sha256_prefixed(&bytes);
        if computed_hash != expected_hash {
            return Err(TransportError::InvalidResponse(format!(
                "artifact hash mismatch for {artifact_name} {artifact_version}"
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
}
