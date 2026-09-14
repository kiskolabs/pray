use crate::materialize::build_package_archive_bytes;
use crate::publish::published_registry_package_version;
use crate::transport_metadata::transport_package_metadata;
use crate::{registry_artifact_path, torrent_manifest_bytes, torrent_manifest_path};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use pray_core::registry::RegistryPackageMetadata;
use pray_core::resolve::ResolvedProject;
use pray_core::ssh_client::with_pray_ssh_session;
use pray_core::PrayResult;
use serde_json::json;

pub(crate) fn publish_to_ssh_server(
    project: &ResolvedProject,
    signer: &str,
    signer_fingerprint: Option<&str>,
    published_at: u64,
    signing_key: Option<&ed25519_dalek::SigningKey>,
    server_url: &str,
) -> PrayResult<()> {
    with_pray_ssh_session(server_url, |session| {
        for package in &project.packages {
            let archive_bytes = build_package_archive_bytes(package)?;
            let artifact_path =
                registry_artifact_path(&package.declaration.name, &package.spec.version);
            session.call_json(
                "artifact.put",
                json!({
                    "path": artifact_path,
                    "body": STANDARD.encode(&archive_bytes),
                }),
            )?;
            let torrent_path = torrent_manifest_path(&artifact_path);
            session.call_json(
                "artifact.put",
                json!({
                    "path": torrent_path,
                    "body": STANDARD.encode(&torrent_manifest_bytes(
                        package,
                        &artifact_path,
                        &archive_bytes,
                    )?),
                }),
            )?;

            let metadata = RegistryPackageMetadata {
                name: package.declaration.name.clone(),
                versions: vec![published_registry_package_version(
                    package,
                    signer,
                    signer_fingerprint,
                    published_at,
                    signing_key,
                    &artifact_path,
                    &archive_bytes,
                )?],
            };
            session.call_json(
                "sync.push",
                json!({
                    "metadata": transport_package_metadata(&metadata),
                }),
            )?;
        }
        Ok(())
    })
}
