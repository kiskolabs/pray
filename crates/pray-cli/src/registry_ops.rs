use crate::auth_client::{
    current_signer as current_signer_from_session,
    current_signer_fingerprint as current_signer_fingerprint_from_session,
};
use crate::project_paths::workspace_root;
#[cfg(feature = "auth")]
use pray_core::auth::RegistryAuthStore;
use pray_core::registry::{RegistryIndex, RegistryPackageMetadata};
use pray_core::ssh_identity::active_ssh_user_fingerprint;
use pray_core::{PrayError, PrayResult};
use pray_transport::{TorrentConfig, TorrentTransport};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn current_signer() -> PrayResult<String> {
    let session_root = workspace_root();
    if let Some(email) = current_signer_from_session(&session_root) {
        return Ok(email);
    }

    if let Ok(token) = std::env::var("PRAY_SESSION_TOKEN") {
        #[cfg(feature = "auth")]
        {
            let auth_root = std::env::var("PRAY_AUTH_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|_| session_root.clone());
            if let Ok(store) = RegistryAuthStore::open(&auth_root) {
                if let Ok(Some(session)) = store.resolve_session(&token) {
                    return Ok(session.email);
                }
            }
        }

        #[cfg(not(feature = "auth"))]
        {
            let _ = token;
            return Err(PrayError::Unsupported(
                "this build was compiled without auth support".to_string(),
            ));
        }
    }

    Ok(std::env::var("PRAY_SIGNER")
        .or_else(|_| std::env::var("USER"))
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string()))
}

pub(crate) fn current_signer_fingerprint() -> Option<String> {
    if let Some(fingerprint) = active_ssh_user_fingerprint() {
        return Some(fingerprint);
    }
    current_signer_fingerprint_from_session(&workspace_root())
}

pub(crate) fn current_timestamp() -> PrayResult<String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PrayError::Resolution(error.to_string()))
        .map(|duration| duration.as_secs().to_string())
}

pub(crate) fn load_registry_index(root: &Path) -> PrayResult<RegistryIndex> {
    let path = root.join("v1/index.json");
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(RegistryIndex {
            spec: "prayfile-distribution-1".to_string(),
            packages: Vec::new(),
        });
    };
    let index: RegistryIndex = serde_json::from_str(&text).map_err(|error| PrayError::Parse {
        kind: "registry index",
        message: error.to_string(),
    })?;
    if index.spec != "prayfile-distribution-1" {
        return Err(PrayError::Resolution(format!(
            "unsupported registry index spec: {}",
            index.spec
        )));
    }
    Ok(index)
}

pub(crate) fn load_registry_package_metadata(
    path: &Path,
    package_name: &str,
) -> PrayResult<RegistryPackageMetadata> {
    if path.exists() {
        let text = fs::read_to_string(path)?;
        let metadata: RegistryPackageMetadata =
            serde_json::from_str(&text).map_err(|error| PrayError::Parse {
                kind: "registry metadata",
                message: error.to_string(),
            })?;
        if metadata.name != package_name {
            return Err(PrayError::Resolution(format!(
                "registry metadata name mismatch: expected {}, found {}",
                package_name, metadata.name
            )));
        }
        Ok(metadata)
    } else {
        Ok(RegistryPackageMetadata {
            name: package_name.to_string(),
            versions: Vec::new(),
        })
    }
}

pub(crate) fn write_registry_index(root: &Path, index: &RegistryIndex) -> PrayResult<()> {
    let path = root.join("v1/index.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(index)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    )?;
    Ok(())
}

pub(crate) fn write_registry_package_metadata(
    path: &Path,
    metadata: &RegistryPackageMetadata,
) -> PrayResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(metadata)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    )?;
    Ok(())
}

pub(crate) fn registry_metadata_path(root: &Path, package_name: &str) -> PathBuf {
    root.join("v1/packages")
        .join(package_name)
        .with_extension("json")
}

pub(crate) fn registry_artifact_path(package_name: &str, version: &str) -> String {
    let artifact_name = format!("{}-{}.praypkg", package_name.replace('/', "-"), version);
    format!("v1/artifacts/{package_name}/{version}/{artifact_name}")
}

pub(crate) fn torrent_manifest_path(artifact_path: &str) -> String {
    format!("{artifact_path}.praytorrent.json")
}

pub(crate) fn torrent_manifest_bytes(
    package: &pray_core::resolve::ResolvedPackage,
    artifact_path: &str,
    archive_bytes: &[u8],
) -> PrayResult<Vec<u8>> {
    let torrent_config = TorrentConfig::default();
    let manifest = TorrentTransport::build_manifest(
        package.declaration.name.clone(),
        package.spec.version.clone(),
        artifact_path.to_string(),
        archive_bytes,
        torrent_config.piece_size,
        vec![artifact_path.to_string()],
        torrent_config.bootstrap_trackers,
    );
    serde_json::to_vec_pretty(&manifest).map_err(|error| PrayError::Manifest(error.to_string()))
}

pub(crate) fn write_torrent_manifest(
    root: &Path,
    package: &pray_core::resolve::ResolvedPackage,
    artifact_path: &str,
    archive_bytes: &[u8],
) -> PrayResult<()> {
    let manifest_path = root.join(torrent_manifest_path(artifact_path));
    write_output_bytes(
        &manifest_path,
        &torrent_manifest_bytes(package, artifact_path, archive_bytes)?,
    )
}

pub(crate) fn write_output_bytes(path: &Path, bytes: &[u8]) -> PrayResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}
