use crate::server::{
    ensure_derived_metadata, latest_publish_timestamp, merge_registry_package_metadata,
    read_known_peers, read_registry_index, read_registry_package_metadata, registry_metadata_path,
    registry_package_metadata_from_transport, update_registry_index_with_package,
    write_registry_package_metadata, Response, ServeAuth,
};
use crate::transport_metadata::transport_package_metadata;
use pray_core::push_auth::authorize_distribution_push;
use pray_core::{PrayError, PrayResult};
use pray_transport::{
    FederationInfo, IndexResponse, PackageMetadata as TransportPackageMetadata, PackageSummary,
    ServerInfo, SyncEndpoints,
};
use std::path::Path;

pub(crate) fn federation_discovery_response(root: &Path) -> PrayResult<Response> {
    let discovery = FederationInfo {
        spec: "pray-federation-v1".to_string(),
        server: ServerInfo {
            name: "pray".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: vec!["static_registry".to_string(), "federation".to_string()],
        },
        sync: SyncEndpoints {
            index_url: "/v1/sync/index".to_string(),
            package_url: "/v1/sync/package/{name}".to_string(),
            artifact_url: "/v1/artifacts/{package}/{version}/{artifact}".to_string(),
            since_param: "since".to_string(),
        },
        peers: read_known_peers(root)?,
    };
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body: serde_json::to_vec_pretty(&discovery)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    })
}

pub(crate) fn federation_index_response_since(
    root: &Path,
    since: Option<u64>,
) -> PrayResult<Response> {
    let index = read_registry_index(root)?;
    let mut packages = Vec::new();
    let mut sync_version = 0u64;

    for package_name in index.packages {
        let Ok(metadata) = read_registry_package_metadata(root, &package_name) else {
            continue;
        };
        let updated_at = latest_publish_timestamp(&metadata)
            .map(|timestamp| timestamp.to_string())
            .unwrap_or_else(|| "0".to_string());
        let updated_at_value = updated_at.parse::<u64>().unwrap_or(0);
        sync_version = sync_version.max(updated_at_value);
        if since.is_some_and(|since| updated_at_value <= since) {
            continue;
        }
        packages.push(PackageSummary {
            name: package_name.clone(),
            updated_at,
            url: format!("/v1/sync/package/{package_name}"),
        });
    }

    let body = IndexResponse {
        spec: "prayfile-distribution-1".to_string(),
        sync_version: sync_version as i64,
        packages,
    };

    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body: serde_json::to_vec_pretty(&body)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    })
}

pub(crate) fn federation_package_response(root: &Path, path: &str) -> PrayResult<Response> {
    let package_name = path.trim_start_matches("/v1/sync/package/");
    let metadata = read_registry_package_metadata(root, package_name)?;
    let body = transport_package_metadata(&metadata);
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body: serde_json::to_vec_pretty(&body)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    })
}

pub(crate) fn federation_push_response(
    root: &Path,
    auth: &ServeAuth,
    body: &[u8],
) -> PrayResult<Response> {
    authorize_distribution_push(
        root,
        &auth.bind_host,
        auth.allow_open_push,
        auth.stdio_mode,
        auth.authorization.as_deref(),
    )?;
    let incoming: TransportPackageMetadata =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "federation package metadata",
            message: error.to_string(),
        })?;
    let registry_metadata = registry_package_metadata_from_transport(&incoming)?;
    let mut merged_metadata = merge_registry_package_metadata(root, registry_metadata)?;
    ensure_derived_metadata(root, &mut merged_metadata)?;
    let metadata_path = registry_metadata_path(root, &merged_metadata.name);
    write_registry_package_metadata(&metadata_path, &merged_metadata)?;
    update_registry_index_with_package(root, &merged_metadata.name)?;
    Ok(Response {
        status: 201,
        content_type: "application/json".to_string(),
        body: serde_json::to_vec_pretty(&serde_json::json!({
            "status": "ok",
            "package": merged_metadata.name,
        }))
        .map_err(|error| PrayError::Manifest(error.to_string()))?,
    })
}
