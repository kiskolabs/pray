use crate::types::*;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// HTTP transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,

    #[serde(default = "default_tls_verify")]
    pub tls_verify: bool,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout(),
            headers: std::collections::HashMap::new(),
            tls_verify: default_tls_verify(),
        }
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_tls_verify() -> bool {
    true
}

/// HTTP transport adapter
pub struct HttpTransport {
    client: Client,
}

impl HttpTransport {
    pub fn new(config: HttpConfig) -> Result<Self> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .danger_accept_invalid_certs(!config.tls_verify);

        // Add default headers
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in config.headers {
            if let (Ok(k), Ok(v)) = (
                reqwest::header::HeaderName::try_from(key.as_str()),
                reqwest::header::HeaderValue::from_str(&value),
            ) {
                headers.insert(k, v);
            }
        }
        builder = builder.default_headers(headers);

        let client = builder.build().map_err(|e| {
            TransportError::Other(anyhow::anyhow!("Failed to create HTTP client: {}", e))
        })?;

        Ok(Self { client })
    }
}

#[async_trait]
impl TransportAdapter for HttpTransport {
    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            pull: true,
            push: true,
            streaming: true,
            binary: true,
            max_message_size: None,
            partial_responses: true,
        }
    }

    fn name(&self) -> &str {
        "http"
    }

    async fn fetch_discovery(&self, peer: &PeerConfig) -> Result<FederationInfo> {
        let url = peer
            .url
            .as_ref()
            .ok_or_else(|| TransportError::InvalidResponse("Missing URL".to_string()))?;

        let discovery_url = format!(
            "{}/.well-known/pray-federation.json",
            url.trim_end_matches('/')
        );

        let response = self
            .client
            .get(&discovery_url)
            .send()
            .await
            .map_err(|e| TransportError::Network(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(TransportError::Network(format!(
                "HTTP {} {}",
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or("")
            )));
        }

        let info: FederationInfo = response
            .json()
            .await
            .map_err(|e| TransportError::InvalidResponse(format!("Invalid JSON: {}", e)))?;

        Ok(info)
    }

    async fn fetch_index(&self, peer: &PeerConfig, since: Option<i64>) -> Result<IndexResponse> {
        let url = peer
            .url
            .as_ref()
            .ok_or_else(|| TransportError::InvalidResponse("Missing URL".to_string()))?;

        let mut index_url = format!("{}/v1/sync/index", url.trim_end_matches('/'));
        if let Some(ts) = since {
            index_url = format!("{}?since={}", index_url, ts);
        }

        let response = self
            .client
            .get(&index_url)
            .send()
            .await
            .map_err(|e| TransportError::Network(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(TransportError::Network(format!(
                "HTTP {} {}",
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or("")
            )));
        }

        let index: IndexResponse = response
            .json()
            .await
            .map_err(|e| TransportError::InvalidResponse(format!("Invalid JSON: {}", e)))?;

        Ok(index)
    }

    async fn fetch_package(&self, peer: &PeerConfig, name: &str) -> Result<PackageMetadata> {
        let url = peer
            .url
            .as_ref()
            .ok_or_else(|| TransportError::InvalidResponse("Missing URL".to_string()))?;

        let package_url = format!("{}/v1/sync/package/{}", url.trim_end_matches('/'), name);

        let response = self
            .client
            .get(&package_url)
            .send()
            .await
            .map_err(|e| TransportError::Network(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Err(TransportError::NotFound(format!(
                    "Package {} not found",
                    name
                )));
            }
            return Err(TransportError::Network(format!(
                "HTTP {} {}",
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or("")
            )));
        }

        let metadata: PackageMetadata = response
            .json()
            .await
            .map_err(|e| TransportError::InvalidResponse(format!("Invalid JSON: {}", e)))?;

        Ok(metadata)
    }

    async fn fetch_artifact(&self, peer: &PeerConfig, artifact: &ArtifactRef) -> Result<Vec<u8>> {
        let artifact_url =
            if artifact.url.starts_with("http://") || artifact.url.starts_with("https://") {
                artifact.url.clone()
            } else {
                let base = peer
                    .url
                    .as_ref()
                    .ok_or_else(|| TransportError::InvalidResponse("Missing URL".to_string()))?;
                format!(
                    "{}/{}",
                    base.trim_end_matches('/'),
                    artifact.url.trim_start_matches('/')
                )
            };

        let response = self
            .client
            .get(&artifact_url)
            .send()
            .await
            .map_err(|e| TransportError::Network(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Err(TransportError::NotFound(format!(
                    "Artifact not found: {}",
                    artifact.url
                )));
            }
            return Err(TransportError::Network(format!(
                "HTTP {} {}",
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or("")
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| TransportError::Network(format!("Failed to read response: {}", e)))?;

        Ok(bytes.to_vec())
    }

    async fn push_package(&self, peer: &PeerConfig, metadata: &PackageMetadata) -> Result<()> {
        let url = peer
            .url
            .as_ref()
            .ok_or_else(|| TransportError::InvalidResponse("Missing URL".to_string()))?;

        let push_url = format!("{}/v1/sync/push", url.trim_end_matches('/'));

        let response = self
            .client
            .post(&push_url)
            .json(metadata)
            .send()
            .await
            .map_err(|e| TransportError::Network(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(TransportError::Network(format!(
                "HTTP {} {}",
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or("")
            )));
        }

        Ok(())
    }
}

/// Factory for creating HTTP transport adapters
pub struct HttpTransportFactory;

impl TransportAdapterFactory for HttpTransportFactory {
    fn name(&self) -> &str {
        "http"
    }

    fn create(&self, config: &PeerConfig) -> Result<Box<dyn TransportAdapter>> {
        let http_config: HttpConfig =
            serde_json::from_value(config.config.clone()).unwrap_or_default();

        let transport = HttpTransport::new(http_config)?;
        Ok(Box::new(transport))
    }
}
