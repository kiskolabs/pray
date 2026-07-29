use crate::registry_http::{http_put_with_headers, join_url};
use crate::registry_ssh::upload_registry_artifact_ssh;
use crate::{PrayError, PrayResult};

pub fn upload_registry_artifact(
    source_url: &str,
    artifact_path: &str,
    bytes: &[u8],
) -> PrayResult<()> {
    upload_registry_artifact_with_authorization(source_url, artifact_path, bytes, None)
}

pub fn upload_registry_artifact_with_authorization(
    source_url: &str,
    artifact_path: &str,
    bytes: &[u8],
    authorization: Option<&str>,
) -> PrayResult<()> {
    if crate::ssh_client::is_pray_ssh_url(source_url) {
        return upload_registry_artifact_ssh(source_url, artifact_path, bytes);
    }
    let endpoint = join_url(source_url, artifact_path);
    let mut header_pairs = Vec::new();
    if let Some(authorization) = authorization {
        header_pairs.push(("Authorization", authorization));
    }
    let response =
        http_put_with_headers(&endpoint, "application/octet-stream", bytes, &header_pairs)?;
    if response.status / 100 != 2 {
        return Err(PrayError::Resolution(format!(
            "artifact upload failed with HTTP {}",
            response.status
        )));
    }
    Ok(())
}

pub fn publish_authorization_header() -> Option<String> {
    std::env::var("PRAY_PUBLISH_TOKEN")
        .ok()
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
        .map(|token| format!("Bearer {token}"))
}
