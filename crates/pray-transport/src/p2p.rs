use crate::types::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "p2p")]
use {
    librqbit::{Session as BitTorrentSession, SessionOptions},
    mainline::{Dht, Id as DhtId},
    sha2::{Digest, Sha256},
    std::collections::HashMap,
};

/// P2P transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// DHT bootstrap nodes
    #[serde(default = "default_bootstrap_nodes")]
    pub bootstrap_nodes: Vec<String>,

    /// Local port for BitTorrent
    #[serde(default = "default_bittorrent_port")]
    pub bittorrent_port: u16,

    /// Local port for DHT
    #[serde(default = "default_dht_port")]
    pub dht_port: u16,

    /// Download directory for packages
    pub download_dir: PathBuf,

    /// Enable seeding after download
    #[serde(default = "default_enable_seeding")]
    pub enable_seeding: bool,

    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_bootstrap_nodes() -> Vec<String> {
    vec![
        "router.bittorrent.com:6881".to_string(),
        "router.utorrent.com:6881".to_string(),
        "dht.transmissionbt.com:6881".to_string(),
    ]
}

fn default_bittorrent_port() -> u16 {
    6881
}

fn default_dht_port() -> u16 {
    6881
}

fn default_enable_seeding() -> bool {
    true
}

fn default_timeout() -> u64 {
    30
}

/// P2P transport adapter
#[cfg(feature = "p2p")]
pub struct P2PTransport {
    config: P2PConfig,
    bittorrent_session: Option<Arc<BitTorrentSession>>,
    dht: Option<Arc<Dht>>,
}

#[cfg(feature = "p2p")]
impl P2PTransport {
    pub fn new(config: P2PConfig) -> Result<Self> {
        Ok(Self {
            config,
            bittorrent_session: None,
            dht: None,
        })
    }

    /// Initialize BitTorrent session
    async fn ensure_bittorrent_session(&mut self) -> Result<Arc<BitTorrentSession>> {
        if let Some(session) = &self.bittorrent_session {
            return Ok(Arc::clone(session));
        }

        let opts = SessionOptions {
            disable_dht: false,
            disable_dht_persistence: false,
            dht_config: None,
            listen_port_range: Some(self.config.bittorrent_port..=self.config.bittorrent_port + 10),
            ..Default::default()
        };

        let session = BitTorrentSession::new_with_opts(
            self.config.download_dir.clone(),
            opts,
        )
        .await
        .map_err(|e| TransportError::Other(anyhow::anyhow!("Failed to create BitTorrent session: {}", e)))?;

        let session = Arc::new(session);
        self.bittorrent_session = Some(Arc::clone(&session));

        Ok(session)
    }

    /// Initialize DHT
    async fn ensure_dht(&mut self) -> Result<Arc<Dht>> {
        if let Some(dht) = &self.dht {
            return Ok(Arc::clone(dht));
        }

        let mut dht = Dht::server()
            .map_err(|e| TransportError::Other(anyhow::anyhow!("Failed to create DHT: {}", e)))?;

        // Bootstrap with known nodes
        for node in &self.config.bootstrap_nodes {
            let _ = dht.add_node(node);
        }

        let dht = Arc::new(dht);
        self.dht = Some(Arc::clone(&dht));

        Ok(dht)
    }

    /// Compute info hash for a package
    fn package_info_hash(package_name: &str, version: &str) -> [u8; 20] {
        let mut hasher = Sha256::new();
        hasher.update(b"pray-package:");
        hasher.update(package_name.as_bytes());
        hasher.update(b":");
        hasher.update(version.as_bytes());

        let hash = hasher.finalize();
        let mut info_hash = [0u8; 20];
        info_hash.copy_from_slice(&hash[..20]);
        info_hash
    }

    /// Announce package to DHT
    async fn announce_package(&mut self, package_name: &str, version: &str) -> Result<()> {
        let dht = self.ensure_dht().await?;
        let info_hash = Self::package_info_hash(package_name, version);

        // Announce to DHT
        dht.announce_peer(info_hash.into(), self.config.bittorrent_port)
            .await
            .map_err(|e| TransportError::Other(anyhow::anyhow!("DHT announce failed: {}", e)))?;

        Ok(())
    }

    /// Discover peers for a package from DHT
    async fn discover_peers(&mut self, package_name: &str, version: &str) -> Result<Vec<String>> {
        let dht = self.ensure_dht().await?;
        let info_hash = Self::package_info_hash(package_name, version);

        // Query DHT for peers
        let peers = dht.get_peers(info_hash.into())
            .await
            .map_err(|e| TransportError::Other(anyhow::anyhow!("DHT lookup failed: {}", e)))?;

        let peer_addrs: Vec<String> = peers
            .values()
            .into_iter()
            .map(|addr| format!("{}:{}", addr.ip(), addr.port()))
            .collect();

        Ok(peer_addrs)
    }

    /// Download package via BitTorrent
    async fn download_via_bittorrent(
        &mut self,
        magnet_link: &str,
    ) -> Result<PathBuf> {
        let session = self.ensure_bittorrent_session().await?;

        let handle = session.add_torrent(
            magnet_link.parse().map_err(|e| TransportError::InvalidResponse(format!("Invalid magnet link: {}", e)))?,
            None,
        )
        .await
        .map_err(|e| TransportError::Network(format!("Failed to add torrent: {}", e)))?;

        // Wait for download with timeout
        let timeout = Duration::from_secs(self.config.timeout_secs);
        let start = std::time::Instant::now();

        loop {
            let stats = handle.stats();

            if stats.finished {
                break;
            }

            if start.elapsed() > timeout {
                return Err(TransportError::Timeout(
                    format!("Download timed out after {} seconds", self.config.timeout_secs)
                ));
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        // Get downloaded file path
        let info = handle.info();
        let files = info.iter_files();

        if files.is_empty() {
            return Err(TransportError::NotFound("No files in torrent".to_string()));
        }

        // Return path to first file (package artifact)
        Ok(files[0].0.to_path_buf())
    }
}

#[cfg(feature = "p2p")]
#[async_trait]
impl TransportAdapter for P2PTransport {
    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            pull: true,
            push: false, // Push via seeding
            streaming: false,
            binary: true,
            max_message_size: None, // Unlimited
            partial_responses: false,
        }
    }

    fn name(&self) -> &str {
        "p2p"
    }

    async fn fetch_discovery(&self, peer: &PeerConfig) -> Result<FederationInfo> {
        // For P2P, discovery info is embedded in the DHT
        // Return a minimal response indicating P2P transport
        Ok(FederationInfo {
            spec: "pray-federation-v1".to_string(),
            server: ServerInfo {
                name: "p2p-dht".to_string(),
                version: "0.1.0".to_string(),
                capabilities: vec!["p2p".to_string(), "dht".to_string()],
            },
            sync: SyncEndpoints {
                index_url: format!("dht://index"),
                package_url: format!("dht://package/{{name}}"),
                artifact_url: format!("magnet:?xt=urn:btih:{{hash}}"),
                since_param: "since".to_string(),
            },
            peers: vec![],
        })
    }

    async fn fetch_index(
        &self,
        peer: &PeerConfig,
        since: Option<i64>,
    ) -> Result<IndexResponse> {
        // For P2P, we need to query DHT for the index
        // This is a simplified implementation - in production, we'd have a DHT-based index

        // Parse DHT key from peer config
        let dht_key = peer.config.get("dht_index_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TransportError::InvalidResponse("Missing dht_index_key".to_string()))?;

        // TODO: Implement DHT-based index lookup
        // For now, return empty index
        Ok(IndexResponse {
            spec: "prayfile-distribution-1".to_string(),
            sync_version: chrono::Utc::now().timestamp(),
            packages: vec![],
        })
    }

    async fn fetch_package(
        &self,
        peer: &PeerConfig,
        name: &str,
    ) -> Result<PackageMetadata> {
        // Query DHT for package metadata
        // In a real implementation, metadata would be stored in DHT

        Err(TransportError::NotFound(format!("Package {} not found in DHT", name)))
    }

    async fn fetch_artifact(
        &self,
        peer: &PeerConfig,
        artifact: &ArtifactRef,
    ) -> Result<Vec<u8>> {
        // Parse magnet link or info hash from artifact URL
        let magnet_link = if artifact.url.starts_with("magnet:") {
            artifact.url.clone()
        } else {
            // Construct magnet link from hash
            format!("magnet:?xt=urn:btih:{}&dn={}", artifact.hash, artifact.name)
        };

        // Download via BitTorrent
        let mut transport = self.clone();
        let file_path = transport.download_via_bittorrent(&magnet_link).await?;

        // Read file contents
        let contents = tokio::fs::read(&file_path).await?;

        // Verify hash
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let computed_hash = format!("sha256:{}", hex::encode(hasher.finalize()));

        if computed_hash != artifact.hash {
            return Err(TransportError::InvalidResponse(
                format!("Hash mismatch: expected {}, got {}", artifact.hash, computed_hash)
            ));
        }

        Ok(contents)
    }
}

// Clone implementation for P2PTransport
#[cfg(feature = "p2p")]
impl Clone for P2PTransport {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            bittorrent_session: self.bittorrent_session.clone(),
            dht: self.dht.clone(),
        }
    }
}

/// Factory for creating P2P transport adapters
#[cfg(feature = "p2p")]
pub struct P2PTransportFactory;

#[cfg(feature = "p2p")]
impl TransportAdapterFactory for P2PTransportFactory {
    fn name(&self) -> &str {
        "p2p"
    }

    fn create(&self, config: &PeerConfig) -> Result<Box<dyn TransportAdapter>> {
        let p2p_config: P2PConfig = serde_json::from_value(config.config.clone())
            .map_err(|e| TransportError::InvalidResponse(format!("Invalid P2P config: {}", e)))?;

        let transport = P2PTransport::new(p2p_config)?;
        Ok(Box::new(transport))
    }
}

#[cfg(not(feature = "p2p"))]
pub struct P2PTransport;

#[cfg(not(feature = "p2p"))]
impl P2PTransport {
    pub fn new(_config: serde_json::Value) -> Result<Self> {
        Err(TransportError::NotCapable(
            "P2P support not enabled. Rebuild with --features p2p".to_string()
        ))
    }
}
