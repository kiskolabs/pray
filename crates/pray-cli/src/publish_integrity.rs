use crate::materialize::find_prayspec_file;
use crate::torrent_manifest_path;
use pray_core::distribution::RegistryDistributionSettings;
use pray_core::hashing::sha256_prefixed;
use pray_core::package_integrity::verify_package_signature;
use pray_core::paths::normalize_package_relative_path;
use pray_core::registry::RegistryPackageVersion;
use pray_core::resolve::ResolvedPackage;
use pray_core::resource_limits::{
    MAX_ARCHIVE_ENTRIES, MAX_ARCHIVE_ENTRY_BYTES, MAX_ARCHIVE_TOTAL_BYTES,
};
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Path;

pub(crate) fn stored_package_artifact(
    root: &Path,
    artifact_path: &str,
    package: &ResolvedPackage,
    existing: &RegistryPackageVersion,
    distribution: &RegistryDistributionSettings,
) -> Option<Vec<u8>> {
    if existing.artifact != artifact_path
        || existing.tree_hash.as_deref() != Some(package.tree_hash.as_str())
    {
        return None;
    }
    let Ok(artifact_bytes) = fs::read(root.join(artifact_path)) else {
        return None;
    };
    let artifact_hash = sha256_prefixed(&artifact_bytes);
    let descriptor_ok =
        !distribution.allows_torrent() || root.join(torrent_manifest_path(artifact_path)).is_file();
    (existing.artifact_hash.as_deref() == Some(artifact_hash.as_str())
        && stored_prayspec_matches(&artifact_bytes, &package.root)
        && descriptor_ok)
        .then_some(artifact_bytes)
}

/// A stored row is current when its own recorded signature still authenticates the
/// stored artifact. The check deliberately ignores who is publishing now: a row signed
/// by another publisher is still a valid attestation of identical bytes, and rewriting it
/// would restate authorship without republishing anything. An unsigned row is treated as
/// not current so that publish repairs it.
pub(crate) fn stored_publish_matches(
    artifact_bytes: &[u8],
    package: &ResolvedPackage,
    existing: &RegistryPackageVersion,
) -> bool {
    existing.signature.is_some()
        && verify_package_signature(
            &package.declaration.name,
            &existing.version,
            existing,
            artifact_bytes,
            &package.tree_hash,
        )
        .is_ok()
}

fn stored_prayspec_matches(artifact_bytes: &[u8], package_root: &Path) -> bool {
    if artifact_bytes.len() as u64 > MAX_ARCHIVE_TOTAL_BYTES {
        return false;
    }
    let Ok(current_path) = find_prayspec_file(package_root) else {
        return false;
    };
    let Some(current_name) = current_path.file_name() else {
        return false;
    };
    let Ok(current_bytes) = fs::read(&current_path) else {
        return false;
    };
    let Ok(decoder) = zstd::stream::read::Decoder::new(artifact_bytes) else {
        return false;
    };
    let mut archive = tar::Archive::new(decoder);
    let Ok(entries) = archive.entries() else {
        return false;
    };
    let mut entry_count = 0usize;
    let mut total_bytes = 0u64;
    let mut paths = BTreeSet::new();
    let mut prayspec_matches = false;
    for entry in entries {
        let Ok(mut entry) = entry else {
            return false;
        };
        let entry_type = entry.header().entry_type();
        if entry_type.is_dir() {
            continue;
        }
        if !entry_type.is_file() {
            return false;
        }
        let Ok(path) = entry.path() else {
            return false;
        };
        let Ok(normalized) = normalize_package_relative_path(&path) else {
            return false;
        };
        if !paths.insert(normalized.clone()) {
            return false;
        }
        entry_count += 1;
        let size = entry.header().size().unwrap_or(0);
        total_bytes = total_bytes.saturating_add(size);
        if entry_count > MAX_ARCHIVE_ENTRIES
            || size > MAX_ARCHIVE_ENTRY_BYTES
            || total_bytes > MAX_ARCHIVE_TOTAL_BYTES
        {
            return false;
        }
        if normalized != Path::new(current_name) {
            continue;
        }
        let mut stored_bytes = Vec::new();
        let Ok(copied) = (&mut entry)
            .take(MAX_ARCHIVE_ENTRY_BYTES.saturating_add(1))
            .read_to_end(&mut stored_bytes)
        else {
            return false;
        };
        if copied as u64 > MAX_ARCHIVE_ENTRY_BYTES {
            return false;
        }
        prayspec_matches = stored_bytes == current_bytes;
    }
    prayspec_matches
}
