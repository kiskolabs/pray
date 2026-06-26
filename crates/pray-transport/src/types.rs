use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Core error type for transport operations
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Transport not capable: {0}")]
    NotCapable(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, TransportError>;

/// Transport capabilities declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportCapabilities {
    /// Supports pull-based fetching
    pub pull: bool,

    /// Supports push-based sending
    pub push: bool,

    /// Supports streaming updates
    pub streaming: bool,

    /// Supports binary data
    pub binary: bool,

    /// Maximum message size in bytes (None = unlimited)
    pub max_message_size: Option<usize>,

    /// Supports partial responses
    pub partial_responses: bool,
}

/// Federation discovery information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationInfo {
    pub spec: String,
    pub server: ServerInfo,
    pub sync: SyncEndpoints,
    #[serde(default)]
    pub peers: Vec<PeerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEndpoints {
    pub index_url: String,
    pub package_url: String,
    pub artifact_url: String,
    #[serde(default = "default_since_param")]
    pub since_param: String,
}

fn default_since_param() -> String {
    "since".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub public: bool,
}

/// Index sync response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResponse {
    pub spec: String,
    pub sync_version: i64,
    pub packages: Vec<PackageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSummary {
    pub name: String,
    pub updated_at: String,
    pub url: String,
}

/// Package metadata response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub versions: Vec<PackageVersion>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersion {
    pub version: String,
    pub artifact: String,
    pub artifact_hash: String,
    pub tree_hash: String,
    pub yanked: bool,
    pub targets: Vec<String>,
    pub exports: Vec<String>,
    pub published_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<PublisherInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<SignatureInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<OriginInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherInfo {
    pub id: String,
    pub key_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub algorithm: String,
    pub signature: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginInfo {
    pub server: String,
    pub first_seen: String,
}

/// Reference to an artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub name: String,
    pub version: String,
    pub url: String,
    pub hash: String,
}

/// Peer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConfig {
    pub name: String,
    pub transport: String,
    pub url: Option<String>,
    pub trust: TrustLevel,
    pub direction: SyncDirection,

    /// Transport-specific configuration as JSON
    #[serde(flatten)]
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    Full,
    MetadataOnly,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncDirection {
    Pull,
    Push,
    Bidirectional,
}

impl SyncDirection {
    pub fn needs_pull(&self) -> bool {
        matches!(self, Self::Pull | Self::Bidirectional)
    }

    pub fn needs_push(&self) -> bool {
        matches!(self, Self::Push | Self::Bidirectional)
    }
}

impl fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrustLevel::Full => write!(f, "full"),
            TrustLevel::MetadataOnly => write!(f, "metadata_only"),
            TrustLevel::Disabled => write!(f, "disabled"),
        }
    }
}

impl fmt::Display for SyncDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncDirection::Pull => write!(f, "pull"),
            SyncDirection::Push => write!(f, "push"),
            SyncDirection::Bidirectional => write!(f, "bidirectional"),
        }
    }
}

/// Core trait for transport adapters
#[async_trait]
pub trait TransportAdapter: Send + Sync {
    /// Get transport capabilities
    fn capabilities(&self) -> TransportCapabilities;

    /// Get transport name
    fn name(&self) -> &str;

    /// Fetch discovery information from peer
    async fn fetch_discovery(&self, peer: &PeerConfig) -> Result<FederationInfo>;

    /// Fetch index from peer (optionally since a timestamp)
    async fn fetch_index(&self, peer: &PeerConfig, since: Option<i64>) -> Result<IndexResponse>;

    /// Fetch package metadata from peer
    async fn fetch_package(&self, peer: &PeerConfig, name: &str) -> Result<PackageMetadata>;

    /// Fetch artifact from peer
    async fn fetch_artifact(&self, peer: &PeerConfig, artifact: &ArtifactRef) -> Result<Vec<u8>>;

    /// Send metadata to peer (for push-capable transports)
    async fn push_package(&self, peer: &PeerConfig, metadata: &PackageMetadata) -> Result<()> {
        let _ = (peer, metadata);
        Err(TransportError::NotCapable(
            "Push not supported by this transport".to_string(),
        ))
    }
}

/// Factory for creating transport adapters
pub trait TransportAdapterFactory: Send + Sync {
    fn name(&self) -> &str;
    fn create(&self, config: &PeerConfig) -> Result<Box<dyn TransportAdapter>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_direction_reports_expected_capabilities() {
        assert!(SyncDirection::Pull.needs_pull());
        assert!(!SyncDirection::Pull.needs_push());
        assert!(SyncDirection::Push.needs_push());
        assert!(!SyncDirection::Push.needs_pull());
        assert!(SyncDirection::Bidirectional.needs_pull());
        assert!(SyncDirection::Bidirectional.needs_push());
    }

    #[test]
    fn trust_level_serializes_and_formats_as_expected() {
        let trust = TrustLevel::MetadataOnly;
        let encoded = serde_json::to_string(&trust).expect("should serialize");
        let decoded: TrustLevel = serde_json::from_str(&encoded).expect("should deserialize");

        assert_eq!(encoded, "\"metadata_only\"");
        assert_eq!(decoded, TrustLevel::MetadataOnly);
        assert_eq!(trust.to_string(), "metadata_only");
    }
}
