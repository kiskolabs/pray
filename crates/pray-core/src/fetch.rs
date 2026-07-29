//! Bounded HTTP and torrent helpers shared by registry install and transport adapters.

use crate::paths::remove_path_if_exists;
use crate::registry_torrent::{fetch_torrent_artifact_to_path, fetch_torrent_manifest};
use crate::PrayResult;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub use crate::registry_http::{
    http_get, http_get_with_headers, http_post, http_put, join_url, HttpResponse,
};
pub use crate::resource_limits::{MAX_HTTP_RESPONSE_BYTES, MAX_TORRENT_ARTIFACT_BYTES};

/// Download a registry artifact using torrent pieces when a sidecar exists, else bounded HTTP GET.
pub fn download_registry_artifact(
    source_url: &str,
    artifact_relative_path: &str,
) -> PrayResult<Vec<u8>> {
    let artifact_url = join_url(source_url, artifact_relative_path);
    if let Some(manifest) = fetch_torrent_manifest(source_url, artifact_relative_path)? {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let staging = std::env::temp_dir().join(format!("pray-torrent-{stamp}"));
        fs::create_dir_all(&staging)?;
        let destination = staging.join("package.praypkg");
        let result = fetch_torrent_artifact_to_path(
            source_url,
            artifact_relative_path,
            &manifest,
            &destination,
        );
        let bytes = match result {
            Ok(bytes) => bytes,
            Err(error) => {
                let _ = remove_path_if_exists(&staging);
                return Err(error);
            }
        };
        let _ = remove_path_if_exists(&staging);
        return Ok(bytes);
    }
    http_get(&artifact_url)
}
