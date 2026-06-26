use crate::http::HttpTransportFactory;
#[cfg(feature = "p2p")]
use crate::p2p::P2PTransportFactory;
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
        registry.register("p2p", Arc::new(P2PTransportFactory));

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

        // P2P availability depends on features
        #[cfg(feature = "p2p")]
        assert!(registry.has_transport("p2p"));

        #[cfg(not(feature = "p2p"))]
        assert!(!registry.has_transport("p2p"));
    }

    #[test]
    fn test_list_transports() {
        let registry = TransportRegistry::new();
        let transports = registry.available_transports();

        assert!(transports.contains(&"http".to_string()));
    }
}
