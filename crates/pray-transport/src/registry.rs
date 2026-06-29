#[cfg(feature = "p2p")]
use crate::federation::FederationTransportFactory;
use crate::http::HttpTransportFactory;
#[cfg(feature = "p2p")]
use crate::p2p::P2PTransportFactory;
#[cfg(feature = "torrent")]
use crate::torrent::TorrentTransportFactory;
use crate::types::*;

use std::collections::HashMap;
use std::sync::Arc;

/// Global registry of available transports
pub struct TransportRegistry {
    factories: HashMap<String, Arc<dyn TransportAdapterFactory>>,
}

impl TransportRegistry {
    /// Create a new registry with default transports
    pub fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
        };

        // Register built-in transports
        registry.register("http", Arc::new(HttpTransportFactory));

        #[cfg(feature = "p2p")]
        {
            registry.register(
                "federation",
                Arc::new(FederationTransportFactory::default()),
            );
            registry.register("p2p", Arc::new(P2PTransportFactory));
        }

        #[cfg(feature = "torrent")]
        registry.register("torrent", Arc::new(TorrentTransportFactory));

        registry
    }

    /// Register a transport factory
    pub fn register(&mut self, name: &str, factory: Arc<dyn TransportAdapterFactory>) {
        self.factories.insert(name.to_string(), factory);
    }

    /// Create a transport adapter from peer config
    pub fn create(&self, config: &PeerConfig) -> Result<Box<dyn TransportAdapter>> {
        let factory = self.factories.get(&config.transport).ok_or_else(|| {
            TransportError::NotCapable(format!("Unknown transport: {}", config.transport))
        })?;

        factory.create(config)
    }

    /// List available transport names
    pub fn available_transports(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }

    /// Check if a transport is available
    pub fn has_transport(&self, name: &str) -> bool {
        self.factories.contains_key(name)
    }
}

impl Default for TransportRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = TransportRegistry::new();

        // HTTP should always be available
        assert!(registry.has_transport("http"));

        // Federation and P2P availability depend on features
        #[cfg(feature = "p2p")]
        {
            assert!(registry.has_transport("federation"));
            assert!(registry.has_transport("p2p"));
        }

        #[cfg(not(feature = "p2p"))]
        {
            assert!(!registry.has_transport("federation"));
            assert!(!registry.has_transport("p2p"));
        }

        #[cfg(feature = "torrent")]
        assert!(registry.has_transport("torrent"));

        #[cfg(not(feature = "torrent"))]
        assert!(!registry.has_transport("torrent"));
    }

    #[test]
    fn test_list_transports() {
        let registry = TransportRegistry::new();
        let transports = registry.available_transports();

        assert!(transports.contains(&"http".to_string()));
    }

    #[test]
    fn test_create_federation_and_p2p_transports_when_enabled() {
        #[cfg(feature = "p2p")]
        {
            let registry = TransportRegistry::new();
            let federation_peer = PeerConfig {
                name: "peer-a".to_string(),
                transport: "federation".to_string(),
                url: Some("https://peer-a.example".to_string()),
                trust: TrustLevel::Full,
                direction: SyncDirection::Pull,
                config: serde_json::json!({}),
            };

            let p2p_peer = PeerConfig {
                transport: "p2p".to_string(),
                ..federation_peer.clone()
            };

            let federation_transport = registry
                .create(&federation_peer)
                .expect("federation transport should exist");
            let p2p_transport = registry
                .create(&p2p_peer)
                .expect("p2p transport should exist");

            assert_eq!(federation_transport.name(), "federation");
            assert_eq!(p2p_transport.name(), "p2p");
        }
    }

    #[test]
    fn test_torrent_transport_is_available_when_enabled() {
        #[cfg(feature = "torrent")]
        {
            let registry = TransportRegistry::new();
            let torrent_peer = PeerConfig {
                name: "torrent-peer".to_string(),
                transport: "torrent".to_string(),
                url: None,
                trust: TrustLevel::Full,
                direction: SyncDirection::Pull,
                config: serde_json::json!({}),
            };

            let transport = registry
                .create(&torrent_peer)
                .expect("torrent transport should exist");

            assert_eq!(transport.name(), "torrent");
        }
    }
}
