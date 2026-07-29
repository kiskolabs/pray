use crate::manifest::ManifestPackage;
use crate::package_integrity::require_remote_integrity_fields;
use crate::paths::remove_path_if_exists;
use crate::registry::{
    lockfile_signer_fingerprint, registry_latest_version_label, ConfessionSubmission,
    RegistryPackageMetadata, RegistryPackageResolution,
};
use crate::registry_select::{apply_yank_policy, select_package_version};
use crate::resolve_context::PackageResolutionContext;
use crate::{PrayError, PrayResult};
use std::fs;
use std::path::Path;

pub(crate) fn resolve_ssh_registry_package_root(
    project_root: &Path,
    source_url: &str,
    declaration: &ManifestPackage,
    context: &PackageResolutionContext,
) -> PrayResult<RegistryPackageResolution> {
    use crate::ssh_client::with_pray_ssh_session;
    use serde_json::json;

    with_pray_ssh_session(source_url, |session| {
        let metadata = fetch_ssh_registry_package_metadata(session, &declaration.name)?;
        let registry_latest_version = registry_latest_version_label(&metadata);
        let selected = select_package_version(
            &metadata,
            &declaration.constraint,
            context.preferred_version.as_deref(),
        )?;
        apply_yank_policy(&declaration.name, &selected, context.fail_on_yanked)?;
        let signer_fingerprint = lockfile_signer_fingerprint(&selected);
        require_remote_integrity_fields(&declaration.name, &selected.version, &selected)?;
        crate::client_trust::enforce_require_signed_packages(
            source_url,
            &declaration.name,
            &selected,
        )?;
        crate::client_trust::gate_pray_ssh_publisher_optional(
            source_url,
            signer_fingerprint.as_deref(),
        )?;
        if let Some(vendored_root) = crate::registry_cache::try_vendored_package_root(
            project_root,
            &declaration.name,
            &selected,
        )? {
            return Ok(RegistryPackageResolution {
                root: vendored_root,
                signer_fingerprint,
                registry_latest_version,
            });
        }
        let cache_directory = crate::registry_cache::registry_cache_directory(
            project_root,
            source_url,
            &declaration.name,
            &selected.version,
        );

        if let Some(mut cached) = crate::registry_cache::try_reuse_cached_registry_package(
            &cache_directory,
            &selected,
            signer_fingerprint.clone(),
        )? {
            cached.registry_latest_version = registry_latest_version.clone();
            return Ok(cached);
        }
        if context.offline {
            return Err(crate::registry_cache::offline_package_error(
                &declaration.name,
                &selected.version,
            ));
        }

        if cache_directory.exists() {
            remove_path_if_exists(&cache_directory)?;
        }
        fs::create_dir_all(&cache_directory)?;

        let artifact_bytes = session.call_bytes(
            "artifact.get",
            json!({
                "path": selected.artifact,
            }),
        )?;
        crate::registry_cache::validate_and_unpack_registry_package(
            &cache_directory,
            declaration,
            &selected,
            &artifact_bytes,
        )?;

        Ok(RegistryPackageResolution {
            root: cache_directory,
            signer_fingerprint,
            registry_latest_version,
        })
    })
}

pub(crate) fn fetch_ssh_registry_package_metadata(
    session: &mut crate::ssh_client::SshRpcSession,
    package_name: &str,
) -> PrayResult<RegistryPackageMetadata> {
    use serde_json::json;

    let metadata_path = format!("v1/packages/{package_name}.json");
    let metadata_bytes = session.call_bytes("artifact.get", json!({ "path": metadata_path }))?;
    serde_json::from_slice(&metadata_bytes).map_err(|error| PrayError::Parse {
        kind: "registry metadata",
        message: error.to_string(),
    })
}

pub(crate) fn submit_confession_ssh(
    source_url: &str,
    confession: &ConfessionSubmission,
) -> PrayResult<()> {
    use crate::ssh_client::with_pray_ssh_session;
    use serde_json::json;

    with_pray_ssh_session(source_url, |session| {
        session.call_json(
            "confession.submit",
            json!({
                "confession": confession,
            }),
        )?;
        Ok(())
    })
}

pub(crate) fn upload_registry_artifact_ssh(
    source_url: &str,
    artifact_path: &str,
    bytes: &[u8],
) -> PrayResult<()> {
    use crate::ssh_client::with_pray_ssh_session;
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use serde_json::json;

    with_pray_ssh_session(source_url, |session| {
        session.call_json(
            "artifact.put",
            json!({
                "path": artifact_path,
                "body": STANDARD.encode(bytes),
            }),
        )?;
        Ok(())
    })
}
