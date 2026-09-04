use crate::materialize::remove_path_if_exists;
use pray_core::lockfile::read_lockfile;
use pray_core::{PrayError, PrayResult};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(crate) fn clean_unused_registry_cache(project_root: &Path) -> PrayResult<()> {
    let lockfile = read_lockfile(&project_root.join("Prayfile.lock"))?;
    validate_lockfile_for_cleanup(&lockfile)?;
    let registry_root = project_root.join(".pray/cache/registry");
    let retained = lockfile
        .package
        .iter()
        .filter_map(|package| retained_registry_path(project_root, &registry_root, &package.path))
        .collect::<Vec<_>>();

    let metadata = match fs::symlink_metadata(&registry_root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return remove_path_if_exists(&registry_root);
    }

    prune_directory(&registry_root, &retained, false)
}

fn validate_lockfile_for_cleanup(lockfile: &pray_core::lockfile::Lockfile) -> PrayResult<()> {
    validate_sha256_digest("manifest_hash", &lockfile.manifest_hash)?;
    for package in &lockfile.package {
        if package.path.is_empty() {
            return Err(lockfile_parse_error("package path must not be empty"));
        }
        validate_sha256_digest("package tree_hash", &package.tree_hash)?;
        validate_sha256_digest("package artifact_hash", &package.artifact_hash)?;
    }
    Ok(())
}

fn validate_sha256_digest(field: &str, value: &str) -> PrayResult<()> {
    let digest = value.strip_prefix("sha256:").unwrap_or_default();
    if digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(lockfile_parse_error(&format!(
            "{field} must be a sha256 digest"
        )))
    }
}

fn lockfile_parse_error(message: &str) -> PrayError {
    PrayError::Parse {
        kind: "lockfile",
        message: message.to_string(),
    }
}

fn retained_registry_path(
    project_root: &Path,
    registry_root: &Path,
    package_path: &str,
) -> Option<PathBuf> {
    let candidate = Path::new(package_path);
    let absolute = if candidate.is_absolute() {
        lexical_normalize(candidate)
    } else {
        lexical_normalize(&project_root.join(candidate))
    };
    absolute
        .strip_prefix(lexical_normalize(registry_root))
        .ok()
        .map(|relative| registry_root.join(relative))
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn prune_directory(path: &Path, retained: &[PathBuf], remove_when_empty: bool) -> PrayResult<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let protects_entry = retained.iter().any(|kept| kept == &entry_path);
        let leads_to_retained = retained.iter().any(|kept| kept.starts_with(&entry_path));
        if protects_entry {
            continue;
        }

        let metadata = fs::symlink_metadata(&entry_path)?;
        if leads_to_retained && metadata.is_dir() && !metadata.file_type().is_symlink() {
            prune_directory(&entry_path, retained, true)?;
        } else {
            remove_path_if_exists(&entry_path)?;
        }
    }

    if remove_when_empty && fs::read_dir(path)?.next().is_none() {
        fs::remove_dir(path)?;
    }
    Ok(())
}
