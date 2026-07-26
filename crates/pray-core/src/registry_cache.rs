use crate::hashing::sha256_prefixed;
use crate::manifest::ManifestPackage;
use crate::package_integrity::{require_remote_integrity_fields, verify_package_signature};
use crate::package_spec::parse_package_spec;
use crate::paths::{find_prayspec_file, validate_package_relative_path};
use crate::registry::{RegistryPackageResolution, RegistryPackageVersion};
use crate::registry_http::http_get;
use crate::{PrayError, PrayResult};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn registry_cache_matches_selected(
    cache_directory: &Path,
    selected: &RegistryPackageVersion,
) -> PrayResult<bool> {
    let expected_tree_hash = selected.tree_hash.as_deref().ok_or_else(|| {
        PrayError::Integrity("cached package missing tree_hash in registry metadata".to_string())
    })?;
    let spec_path = find_prayspec_file(cache_directory)?;
    let spec_text = fs::read_to_string(&spec_path)?;
    let spec = parse_package_spec(&spec_text)?.canonicalized();
    if spec.version != selected.version {
        return Ok(false);
    }
    let tree_hash = spec.tree_hash_for_root(cache_directory)?;
    if tree_hash != expected_tree_hash {
        return Ok(false);
    }
    if selected
        .signature
        .as_deref()
        .is_some_and(|value| value.starts_with(crate::package_integrity::ED25519_SIGNATURE_PREFIX))
    {
        verify_package_signature(
            &spec.name,
            &selected.version,
            selected,
            &[],
            &tree_hash,
        )?;
    }
    Ok(true)
}

pub(crate) fn try_reuse_cached_registry_package(
    cache_directory: &Path,
    selected: &RegistryPackageVersion,
    signer_fingerprint: Option<String>,
) -> PrayResult<Option<RegistryPackageResolution>> {
    if find_prayspec_file(cache_directory).is_ok()
        && registry_cache_matches_selected(cache_directory, selected)?
    {
        return Ok(Some(RegistryPackageResolution {
            root: cache_directory.to_path_buf(),
            signer_fingerprint,
            registry_latest_version: None,
        }));
    }
    Ok(None)
}

pub(crate) fn validate_and_unpack_registry_package(
    cache_directory: &Path,
    declaration: &ManifestPackage,
    selected: &RegistryPackageVersion,
    artifact_bytes: &[u8],
) -> PrayResult<()> {
    require_remote_integrity_fields(&declaration.name, &selected.version, selected)?;
    let expected_artifact_hash = selected.artifact_hash.as_deref().expect("checked");
    let artifact_hash = sha256_prefixed(artifact_bytes);
    if artifact_hash != expected_artifact_hash {
        return Err(PrayError::Integrity(format!(
            "package artifact hash mismatch for {} {}",
            declaration.name, selected.version
        )));
    }
    unpack_praypkg(artifact_bytes, cache_directory)?;

    let spec_path = find_prayspec_file(cache_directory)?;
    let spec_text = fs::read_to_string(&spec_path)?;
    let spec = parse_package_spec(&spec_text)?.canonicalized();

    if spec.name != declaration.name {
        return Err(PrayError::Resolution(format!(
            "package path {:?} declares {:?}, expected {:?}",
            cache_directory, spec.name, declaration.name
        )));
    }
    if spec.version != selected.version {
        return Err(PrayError::Resolution(format!(
            "package {} version {} does not match registry version {}",
            declaration.name, spec.version, selected.version
        )));
    }

    let tree_hash = spec.tree_hash_for_root(cache_directory)?;
    let expected_tree_hash = selected.tree_hash.as_deref().expect("checked");
    if tree_hash != expected_tree_hash {
        return Err(PrayError::Integrity(format!(
            "package tree hash mismatch for {} {}",
            declaration.name, selected.version
        )));
    }

    verify_package_signature(
        &declaration.name,
        &selected.version,
        selected,
        artifact_bytes,
        &tree_hash,
    )?;

    Ok(())
}

pub(crate) fn registry_cache_directory(
    project_root: &Path,
    source_url: &str,
    package_name: &str,
    version: &str,
) -> PathBuf {
    let source_key = sha256_prefixed(source_url.as_bytes())
        .trim_start_matches("sha256:")
        .chars()
        .take(16)
        .collect::<String>();
    project_root
        .join(".pray/cache/registry")
        .join(source_key)
        .join(package_name)
        .join(version)
}

pub(crate) fn try_vendored_package_root(
    project_root: &Path,
    package_name: &str,
    selected: &RegistryPackageVersion,
) -> PrayResult<Option<PathBuf>> {
    let vendored = project_root
        .join(".pray/vendor")
        .join(package_name.replace('/', "-"))
        .join(&selected.version);
    if find_prayspec_file(&vendored).is_err() {
        return Ok(None);
    }
    let expected_tree_hash = selected.tree_hash.as_deref().ok_or_else(|| {
        PrayError::Integrity(format!(
            "package {package_name} {} is missing tree_hash",
            selected.version
        ))
    })?;
    let spec_path = find_prayspec_file(&vendored)?;
    let spec_text = fs::read_to_string(&spec_path)?;
    let spec = parse_package_spec(&spec_text)?.canonicalized();
    if spec.name != package_name {
        return Err(PrayError::Integrity(format!(
            "vendored package declares {:?}, expected {package_name}",
            spec.name
        )));
    }
    if spec.version != selected.version {
        return Err(PrayError::Integrity(format!(
            "vendored package version {} does not match registry version {}",
            spec.version, selected.version
        )));
    }
    let tree_hash = spec.tree_hash_for_root(&vendored)?;
    if tree_hash != expected_tree_hash {
        return Err(PrayError::Integrity(format!(
            "vendored package tree hash mismatch for {package_name} {}",
            selected.version
        )));
    }
    if selected
        .signature
        .as_deref()
        .is_some_and(|value| value.starts_with(crate::package_integrity::ED25519_SIGNATURE_PREFIX))
    {
        verify_package_signature(package_name, &selected.version, selected, &[], &tree_hash)?;
    }
    Ok(Some(vendored))
}

pub(crate) fn offline_package_error(package_name: &str, version: &str) -> PrayError {
    PrayError::Resolution(format!(
        "package {package_name} {version} is not cached locally and offline mode is enabled"
    ))
}

pub(crate) fn read_local_registry_artifact_bytes(
    source_root: &Path,
    artifact: &str,
) -> PrayResult<Vec<u8>> {
    if artifact.starts_with("http://") || artifact.starts_with("https://") {
        return http_get(artifact);
    }
    if let Some(path) = artifact.strip_prefix("file://") {
        return fs::read(Path::new(path)).map_err(Into::into);
    }
    let artifact_path = Path::new(artifact);
    validate_package_relative_path(artifact_path)?;
    let full_path = source_root.join(artifact_path);
    fs::read(&full_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PrayError::Resolution(format!(
                "package artifact missing at {}. The distribution metadata may be stale or incomplete.",
                full_path.display()
            ))
        } else {
            PrayError::Resolution(format!(
                "failed to read package artifact {}: {error}",
                full_path.display()
            ))
        }
    })
}

pub(crate) fn unpack_praypkg(artifact_bytes: &[u8], output_directory: &Path) -> PrayResult<()> {
    let cursor = std::io::Cursor::new(artifact_bytes);
    let decoder = zstd::stream::read::Decoder::new(cursor)
        .map_err(|error| PrayError::Integrity(error.to_string()))?;
    let mut archive = tar::Archive::new(decoder);
    let mut written_paths = BTreeSet::new();

    for entry in archive
        .entries()
        .map_err(|error| PrayError::Integrity(error.to_string()))?
    {
        let mut entry = entry.map_err(|error| PrayError::Integrity(error.to_string()))?;
        let entry_type = entry.header().entry_type();
        if entry_type.is_dir() {
            continue;
        }
        if entry_type.is_symlink() || entry_type.is_hard_link() || !entry_type.is_file() {
            return Err(PrayError::Integrity(
                "unsupported package archive entry type".to_string(),
            ));
        }
        let path = entry
            .path()
            .map_err(|error| PrayError::Integrity(error.to_string()))?
            .into_owned();
        validate_package_relative_path(&path)?;
        if !written_paths.insert(path.clone()) {
            return Err(PrayError::Integrity(format!(
                "duplicate package archive path: {}",
                path.display()
            )));
        }
        let destination = output_directory.join(&path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut destination_file = fs::File::create(&destination)?;
        std::io::copy(&mut entry, &mut destination_file)
            .map_err(|error| PrayError::Integrity(error.to_string()))?;
    }
    Ok(())
}
