use crate::federation::{FederationConfig, FederationTransport};
use crate::types::*;
use async_trait::async_trait;

/// P2P transport config is currently the same as federation transport config.
pub type P2PConfig = FederationConfig;

/// P2P transport adapter.
pub struct P2PTransport {
    inner: FederationTransport,
}

impl P2PTransport {
    pub fn new(config: P2PConfig) -> Result<Self> {
        Ok(Self {
            inner: FederationTransport::new("p2p", config)?,
        })
    }
}

#[async_trait]
impl TransportAdapter for P2PTransport {
    fn capabilities(&self) -> TransportCapabilities {
        self.inner.capabilities()
    }

    fn name(&self) -> &str {
        "p2p"
    }

    async fn fetch_discovery(&self, peer: &PeerConfig) -> Result<FederationInfo> {
        self.inner.fetch_discovery(peer).await
    }

    async fn fetch_index(&self, peer: &PeerConfig, since: Option<i64>) -> Result<IndexResponse> {
        self.inner.fetch_index(peer, since).await
    }

    async fn fetch_package(&self, peer: &PeerConfig, name: &str) -> Result<PackageMetadata> {
        self.inner.fetch_package(peer, name).await
    }

    async fn fetch_artifact(&self, peer: &PeerConfig, artifact: &ArtifactRef) -> Result<Vec<u8>> {
        self.inner.fetch_artifact(peer, artifact).await
    }

    async fn push_package(&self, peer: &PeerConfig, metadata: &PackageMetadata) -> Result<()> {
        self.inner.push_package(peer, metadata).await
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
