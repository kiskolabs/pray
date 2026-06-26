use crate::http::{HttpConfig, HttpTransport};
use crate::types::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// P2P federation configuration.
///
/// This is the first transport-pluggable federation layer: it bootstraps from a
/// manual peer list, shares known peers, and delegates the actual wire protocol
/// to the underlying HTTP transport for now.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// Manually configured peer URLs used when discovery has no explicit URL.
    #[serde(default)]
    pub bootstrap_peers: Vec<String>,

    /// Allow discovered servers to advertise additional peers.
    #[serde(default = "default_allow_peer_sharing")]
    pub allow_peer_sharing: bool,

    /// HTTP settings used for federation calls.
    #[serde(default)]
    pub http: HttpConfig,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            bootstrap_peers: Vec::new(),
            allow_peer_sharing: default_allow_peer_sharing(),
            http: HttpConfig::default(),
        }
    }
}

fn default_allow_peer_sharing() -> bool {
    true
}

/// P2P transport adapter.
pub struct P2PTransport {
    config: P2PConfig,
    http: HttpTransport,
}

impl P2PTransport {
    pub fn new(config: P2PConfig) -> Result<Self> {
        let http = HttpTransport::new(config.http.clone())?;
        Ok(Self { config, http })
    }

    fn candidate_urls(&self, peer: &PeerConfig) -> Vec<String> {
        let mut candidates = Vec::new();

        if let Some(url) = &peer.url {
            candidates.push(url.clone());
        }

        for url in &self.config.bootstrap_peers {
            if !candidates.contains(url) {
                candidates.push(url.clone());
            }
        }

        candidates
    }

    fn with_url(peer: &PeerConfig, url: String) -> PeerConfig {
        let mut peer = peer.clone();
        peer.url = Some(url);
        peer
    }

    fn merge_peer_lists(&self, mut peers: Vec<PeerInfo>) -> Vec<PeerInfo> {
        if !self.config.allow_peer_sharing {
            return peers;
        }

        let mut seen_urls: HashSet<String> = peers.iter().map(|peer| peer.url.clone()).collect();

        for url in &self.config.bootstrap_peers {
            if seen_urls.insert(url.clone()) {
                peers.push(PeerInfo {
                    name: url.clone(),
                    url: url.clone(),
                    public: false,
                });
            }
        }

        peers
    }
}

#[async_trait]
impl TransportAdapter for P2PTransport {
    fn capabilities(&self) -> TransportCapabilities {
        self.http.capabilities()
    }

    fn name(&self) -> &str {
        "p2p"
    }

    async fn fetch_discovery(&self, peer: &PeerConfig) -> Result<FederationInfo> {
        let mut last_error: Option<TransportError> = None;

        for url in self.candidate_urls(peer) {
            let candidate = Self::with_url(peer, url);

            match self.http.fetch_discovery(&candidate).await {
                Ok(mut discovery) => {
                    discovery.peers = self.merge_peer_lists(discovery.peers);
                    return Ok(discovery);
                }
                Err(error) => {
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TransportError::InvalidResponse("No peer URL available for discovery".to_string())
        }))
    }

    async fn fetch_index(&self, peer: &PeerConfig, since: Option<i64>) -> Result<IndexResponse> {
        self.http.fetch_index(peer, since).await
    }

    async fn fetch_package(&self, peer: &PeerConfig, name: &str) -> Result<PackageMetadata> {
        self.http.fetch_package(peer, name).await
    }

    async fn fetch_artifact(&self, peer: &PeerConfig, artifact: &ArtifactRef) -> Result<Vec<u8>> {
        self.http.fetch_artifact(peer, artifact).await
    }

    async fn push_package(&self, peer: &PeerConfig, metadata: &PackageMetadata) -> Result<()> {
        self.http.push_package(peer, metadata).await
    }
}

/// Factory for creating P2P transport adapters.
pub struct P2PTransportFactory;

impl TransportAdapterFactory for P2PTransportFactory {
    fn name(&self) -> &str {
        "p2p"
    }

    fn create(&self, config: &PeerConfig) -> Result<Box<dyn TransportAdapter>> {
        let p2p_config: P2PConfig =
            serde_json::from_value(config.config.clone()).unwrap_or_default();
        let transport = P2PTransport::new(p2p_config)?;
        Ok(Box::new(transport))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_bootstrap_peers_into_discovery_list() {
        let transport = P2PTransport::new(P2PConfig {
            bootstrap_peers: vec![
                "https://seed-one.example".to_string(),
                "https://seed-two.example".to_string(),
            ],
            allow_peer_sharing: true,
            http: HttpConfig::default(),
        })
        .expect("transport should build");

        let peers = transport.merge_peer_lists(vec![PeerInfo {
            name: "primary".to_string(),
            url: "https://primary.example".to_string(),
            public: true,
        }]);

        assert_eq!(peers.len(), 3);
        assert!(peers
            .iter()
            .any(|peer| peer.url == "https://seed-one.example"));
        assert!(peers
            .iter()
            .any(|peer| peer.url == "https://seed-two.example"));
    }

    #[test]
    fn candidate_urls_keep_explicit_peer_first() {
        let transport = P2PTransport::new(P2PConfig {
            bootstrap_peers: vec!["https://bootstrap.example".to_string()],
            allow_peer_sharing: true,
            http: HttpConfig::default(),
        })
        .expect("transport should build");

        let peer = PeerConfig {
            name: "peer-a".to_string(),
            transport: "p2p".to_string(),
            url: Some("https://peer-a.example".to_string()),
            trust: TrustLevel::Full,
            direction: SyncDirection::Bidirectional,
            config: serde_json::json!({}),
        };

        let urls = transport.candidate_urls(&peer);

        assert_eq!(
            urls,
            vec![
                "https://peer-a.example".to_string(),
                "https://bootstrap.example".to_string()
            ]
        );
    }
}
