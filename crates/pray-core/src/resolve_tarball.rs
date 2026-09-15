use crate::hashing::sha256_prefixed;
use crate::package_archive::unpack_praypkg;
use crate::paths::{find_prayspec_file, remove_path_if_exists};
use crate::resolve_context::ResolveOptions;
use crate::resource_limits::MAX_ARCHIVE_TOTAL_BYTES;
use crate::{PrayError, PrayResult};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn resolve_tarball_package_root(
    project_root: &Path,
    tarball: &str,
    options: &ResolveOptions,
) -> PrayResult<PathBuf> {
    let artifact_bytes = read_tarball_bytes(project_root, tarball, options)?;
    if artifact_bytes.len() as u64 > MAX_ARCHIVE_TOTAL_BYTES {
        return Err(PrayError::Integrity(format!(
            "package archive exceeds {MAX_ARCHIVE_TOTAL_BYTES} bytes"
        )));
    }
    let cache_key = sha256_prefixed(&artifact_bytes)
        .trim_start_matches("sha256:")
        .chars()
        .take(16)
        .collect::<String>();
    let cache_directory = project_root.join(".pray/cache/tarball").join(cache_key);
    if find_prayspec_file(&cache_directory).is_ok() {
        return Ok(cache_directory);
    }
    let staging_directory = cache_directory.with_extension("staging");
    remove_path_if_exists(&staging_directory)?;
    fs::create_dir_all(&staging_directory)?;
    if let Err(error) = unpack_praypkg(&artifact_bytes, &staging_directory) {
        let _ = remove_path_if_exists(&staging_directory);
        return Err(error);
    }
    if let Err(error) = find_prayspec_file(&staging_directory) {
        let _ = remove_path_if_exists(&staging_directory);
        return Err(error);
    }
    if let Some(parent) = cache_directory.parent() {
        fs::create_dir_all(parent)?;
    }
    remove_path_if_exists(&cache_directory)?;
    fs::rename(&staging_directory, &cache_directory).map_err(|error| {
        let _ = remove_path_if_exists(&staging_directory);
        PrayError::from(error)
    })?;
    Ok(cache_directory)
}

fn read_tarball_bytes(
    project_root: &Path,
    tarball: &str,
    options: &ResolveOptions,
) -> PrayResult<Vec<u8>> {
    if tarball.starts_with("http://") || tarball.starts_with("https://") {
        if options.offline {
            return Err(PrayError::Resolution(format!(
                "tarball {tarball} is not cached locally and offline mode is enabled"
            )));
        }
        return crate::registry_http::http_get(tarball);
    }
    let path = local_tarball_path(project_root, tarball);
    fs::read(&path).map_err(|error| {
        PrayError::Resolution(format!("tarball missing at {}: {error}", path.display()))
    })
}

fn local_tarball_path(project_root: &Path, tarball: &str) -> PathBuf {
    let path = tarball.strip_prefix("file://").unwrap_or(tarball);
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}
