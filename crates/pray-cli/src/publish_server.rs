use crate::materialize::build_package_archive_bytes;
use crate::publish_context::{PublishDestLabel, PublishIdentity};
use crate::publish_plan::{report_published, PublishStats};
use crate::publish_ssh::publish_to_ssh_server;
use crate::publish_version::published_registry_package_version;
use crate::registry_ops::{registry_artifact_path, torrent_manifest_bytes, torrent_manifest_path};
use crate::sync_peers::map_transport_error;
use crate::transport_metadata::transport_package_metadata;
use pray_core::distribution::fetch_registry_distribution_settings;
use pray_core::registry::{
    publish_authorization_header, upload_registry_artifact_with_authorization,
    RegistryPackageMetadata,
};
use pray_core::resolve::ResolvedPackage;
use pray_core::PrayResult;
use pray_transport::{PeerConfig, SyncDirection, TransportRegistry, TrustLevel};

pub(crate) fn publish_to_server(
    packages: &[&ResolvedPackage],
    identity: &PublishIdentity<'_>,
    server_url: &str,
    runtime: &tokio::runtime::Runtime,
    label: &PublishDestLabel<'_>,
) -> PrayResult<PublishStats> {
    if pray_core::ssh_client::is_pray_ssh_url(server_url) {
        publish_to_ssh_server(
            packages,
            identity.signer,
            identity.signer_fingerprint,
            identity.published_at,
            identity.signing_key,
            server_url,
        )?;
        return Ok(report_server_published(packages, label));
    }

    let authorization = publish_authorization_header();
    let mut headers = serde_json::Map::new();
    if let Some(authorization) = authorization.as_ref() {
        headers.insert(
            "Authorization".to_string(),
            serde_json::Value::String(authorization.clone()),
        );
    }
    let peer = PeerConfig {
        name: server_url.to_string(),
        transport: "http".to_string(),
        url: Some(server_url.to_string()),
        trust: TrustLevel::Full,
        direction: SyncDirection::Push,
        config: serde_json::json!({ "headers": headers }),
    };
    let registry = TransportRegistry::new();
    let transport = registry.create(&peer).map_err(map_transport_error)?;
    let distribution = fetch_registry_distribution_settings(server_url)?;

    let mut stats = PublishStats {
        published: 0,
        unchanged: 0,
    };
    for package in packages {
        let archive_bytes = build_package_archive_bytes(package)?;
        let artifact_path =
            registry_artifact_path(&package.declaration.name, &package.spec.version);
        upload_registry_artifact_with_authorization(
            server_url,
            &artifact_path,
            &archive_bytes,
            authorization.as_deref(),
        )?;
        if distribution.allows_torrent() {
            upload_registry_artifact_with_authorization(
                server_url,
                &torrent_manifest_path(&artifact_path),
                &torrent_manifest_bytes(
                    package,
                    &artifact_path,
                    &archive_bytes,
                    distribution.bootstrap_trackers.clone(),
                )?,
                authorization.as_deref(),
            )?;
        }

        let metadata = RegistryPackageMetadata {
            name: package.declaration.name.clone(),
            versions: vec![published_registry_package_version(
                package,
                identity.signer,
                identity.signer_fingerprint,
                identity.published_at,
                identity.signing_key,
                &artifact_path,
                &archive_bytes,
            )?],
        };
        let transport_metadata = transport_package_metadata(&metadata);
        runtime
            .block_on(transport.push_package(&peer, &transport_metadata))
            .map_err(map_transport_error)?;
        report_published(package, label.dest_name, label.target, &artifact_path);
        stats.published += 1;
    }

    Ok(stats)
}

fn report_server_published(
    packages: &[&ResolvedPackage],
    label: &PublishDestLabel<'_>,
) -> PublishStats {
    let mut stats = PublishStats {
        published: 0,
        unchanged: 0,
    };
    for package in packages {
        let artifact_path =
            registry_artifact_path(&package.declaration.name, &package.spec.version);
        report_published(package, label.dest_name, label.target, &artifact_path);
        stats.published += 1;
    }
    stats
}
