use crate::types::*;
use async_trait::async_trait;
use pray_core::ssh_client::{is_pray_ssh_url, parse_pray_ssh_url, SshRpcSession};
use serde_json::json;

pub struct SshTransport;

impl Default for SshTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl SshTransport {
    pub fn new() -> Self {
        Self
    }

    fn peer_url(peer: &PeerConfig) -> Result<String> {
        peer.url
            .clone()
            .ok_or_else(|| TransportError::InvalidResponse("Missing pray+ssh URL".to_string()))
    }

    fn with_session<T>(
        peer: &PeerConfig,
        operation: impl FnOnce(&mut SshRpcSession) -> Result<T>,
    ) -> Result<T> {
        let url = Self::peer_url(peer)?;
        if !is_pray_ssh_url(&url) {
            return Err(TransportError::InvalidResponse(format!(
                "ssh transport requires pray+ssh:// url, got {url}"
            )));
        }
        let target = parse_pray_ssh_url(&url).map_err(|error| {
            TransportError::InvalidResponse(format!("invalid pray+ssh url: {error}"))
        })?;
        let mut session = SshRpcSession::connect(&target).map_err(|error| {
            TransportError::Network(format!("failed to open ssh rpc session: {error}"))
        })?;
        operation(&mut session)
    }

    fn call_json(
        session: &mut SshRpcSession,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        session
            .call_json(method, params)
            .map_err(|error| TransportError::Network(format!("ssh rpc {method} failed: {error}")))
    }
}

#[async_trait]
impl TransportAdapter for SshTransport {
    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            pull: true,
            push: true,
            streaming: false,
            binary: true,
            max_message_size: Some(16 * 1024 * 1024),
            partial_responses: false,
        }
    }

    fn name(&self) -> &str {
        "ssh"
    }

    async fn fetch_discovery(&self, peer: &PeerConfig) -> Result<FederationInfo> {
        let peer = peer.clone();
        tokio::task::spawn_blocking(move || {
            Self::with_session(&peer, |session| {
                let body = Self::call_json(session, "federation.discovery", json!({}))?;
                serde_json::from_value(body).map_err(|error| {
                    TransportError::InvalidResponse(format!("invalid discovery response: {error}"))
                })
            })
        })
        .await
        .map_err(|error| TransportError::Other(anyhow::anyhow!("ssh task failed: {error}")))?
    }

    async fn fetch_index(&self, peer: &PeerConfig, since: Option<i64>) -> Result<IndexResponse> {
        let peer = peer.clone();
        tokio::task::spawn_blocking(move || {
            Self::with_session(&peer, |session| {
                let mut params = json!({});
                if let Some(since) = since {
                    params["since"] = json!(since);
                }
                let body = Self::call_json(session, "sync.index", params)?;
                serde_json::from_value(body).map_err(|error| {
                    TransportError::InvalidResponse(format!("invalid index response: {error}"))
                })
            })
        })
        .await
        .map_err(|error| TransportError::Other(anyhow::anyhow!("ssh task failed: {error}")))?
    }

    async fn fetch_package(&self, peer: &PeerConfig, name: &str) -> Result<PackageMetadata> {
        let peer = peer.clone();
        let package_name = name.to_string();
        tokio::task::spawn_blocking(move || {
            Self::with_session(&peer, |session| {
                let body =
                    Self::call_json(session, "sync.package", json!({ "name": package_name }))?;
                serde_json::from_value(body).map_err(|error| {
                    TransportError::InvalidResponse(format!("invalid package response: {error}"))
                })
            })
        })
        .await
        .map_err(|error| TransportError::Other(anyhow::anyhow!("ssh task failed: {error}")))?
    }

    async fn fetch_artifact(&self, peer: &PeerConfig, artifact: &ArtifactRef) -> Result<Vec<u8>> {
        let peer = peer.clone();
        let artifact_path = artifact.url.clone();
        tokio::task::spawn_blocking(move || {
            Self::with_session(&peer, |session| {
                session
                    .call_bytes("artifact.get", json!({ "path": artifact_path }))
                    .map_err(|error| {
                        TransportError::Network(format!("ssh artifact.get failed: {error}"))
                    })
            })
        })
        .await
        .map_err(|error| TransportError::Other(anyhow::anyhow!("ssh task failed: {error}")))?
    }

    async fn push_package(&self, peer: &PeerConfig, metadata: &PackageMetadata) -> Result<()> {
        let peer = peer.clone();
        let metadata = metadata.clone();
        tokio::task::spawn_blocking(move || {
            Self::with_session(&peer, |session| {
                Self::call_json(session, "sync.push", json!({ "metadata": metadata }))?;
                Ok(())
            })
        })
        .await
        .map_err(|error| TransportError::Other(anyhow::anyhow!("ssh task failed: {error}")))?
    }
}

pub struct SshTransportFactory;

impl TransportAdapterFactory for SshTransportFactory {
    fn name(&self) -> &str {
        "ssh"
    }

    fn create(&self, _config: &PeerConfig) -> Result<Box<dyn TransportAdapter>> {
        Ok(Box::new(SshTransport::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_transport_declares_push_and_pull() {
        let transport = SshTransport::new();
        let capabilities = transport.capabilities();
        assert!(capabilities.pull);
        assert!(capabilities.push);
        assert_eq!(transport.name(), "ssh");
    }
}
