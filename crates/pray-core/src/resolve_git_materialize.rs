use crate::paths::remove_path_if_exists;
use crate::resolve_git::{git_object_exists, is_local_distribution_root};
use crate::resolve_git_clone::catalog_sparse_cones;
use crate::resolve_git_command::run_git_command_with_env;
use crate::resolve_git_lock::acquire_git_cache_lock;
use crate::resolve_git_materialize_check::{
    annotate_git_materialize_failure, require_revision_present_offline,
};
use crate::resolve_git_materialize_markers::{
    catalog_root_markers, offline_git_env, GIT_DIR_MARKER, REVISION_MARKER,
};
use crate::resolve_git_materialize_size::{
    require_blob_within_limit, require_tree_prefix_within_limits,
};
use crate::resolve_git_materialize_unpack::unpack_bounded_catalog_tar;
use crate::{PrayError, PrayResult};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn materialize_catalog_tree(
    git_dir: &Path,
    dest: &Path,
    revision: &str,
    subdir: Option<&str>,
    refresh: bool,
    offline: bool,
) -> PrayResult<()> {
    require_revision_present_offline(git_dir, revision, offline)?;
    let _lock = acquire_git_cache_lock(dest)?;
    if !refresh && catalog_matches(dest, git_dir, revision) {
        return Ok(());
    }
    let staging = staging_catalog_path(dest);
    if staging.exists() {
        remove_path_if_exists(&staging)?;
    }
    fs::create_dir_all(&staging)?;
    let mut unpacked = false;
    for prefix in catalog_sparse_cones(subdir) {
        if unpack_archive_prefix(git_dir, &staging, revision, &prefix, offline)? {
            unpacked = true;
        }
    }
    if !unpacked {
        remove_path_if_exists(&staging)?;
        if offline {
            return Err(PrayError::Resolution(format!(
                "git catalog at revision {revision} is not materializable locally and offline mode is enabled"
            )));
        }
        return Err(PrayError::Resolution(format!(
            "no pray distribution root in git source at revision {revision}"
        )));
    }
    fs::write(staging.join(REVISION_MARKER), revision)?;
    let git_dir_text = git_dir.to_str().ok_or_else(|| {
        PrayError::Resolution(format!("invalid global git cache path: {:?}", git_dir))
    })?;
    fs::write(staging.join(GIT_DIR_MARKER), git_dir_text)?;
    publish_catalog_tree(dest, &staging)
}

pub(crate) fn materialize_git_catalog_file(
    source_root: &Path,
    relative: &Path,
    offline: bool,
) -> PrayResult<()> {
    let full_path = source_root.join(relative);
    if full_path.is_file() {
        return Ok(());
    }
    let Some((git_dir, revision, catalog_root)) = catalog_root_markers(source_root) else {
        return Ok(());
    };
    let _lock = acquire_git_cache_lock(&catalog_root)?;
    if full_path.is_file() {
        return Ok(());
    }
    require_revision_present_offline(&git_dir, &revision, offline)?;
    let Ok(repo_relative) = full_path.strip_prefix(&catalog_root) else {
        return Ok(());
    };
    let Some(repo_relative) = repo_relative.to_str() else {
        return Ok(());
    };
    let object = format!("{revision}:{repo_relative}");
    require_blob_within_limit(&git_dir, &object, offline)
        .map_err(|error| annotate_git_materialize_failure(&git_dir, &revision, error))?;
    let output = run_git_command_with_env(&git_dir, &["show", &object], offline_git_env(offline))
        .map_err(|error| annotate_git_materialize_failure(&git_dir, &revision, error))?;
    if !output.status.success() {
        let error = crate::resolve_git_command::command_error("git show", output);
        if offline {
            return Err(PrayError::Resolution(format!(
                "git catalog blob {repo_relative} at revision {revision} is not available locally and offline mode is enabled"
            )));
        }
        return Err(annotate_git_materialize_failure(&git_dir, &revision, error));
    }
    write_catalog_blob_atomic(&full_path, &output.stdout)
}

fn write_catalog_blob_atomic(full_path: &Path, bytes: &[u8]) -> PrayResult<()> {
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut partial = full_path.as_os_str().to_os_string();
    partial.push(".partial");
    let partial = PathBuf::from(partial);
    fs::write(&partial, bytes).map_err(|error| {
        PrayError::Resolution(format!(
            "failed to write git catalog file {}: {error}",
            full_path.display()
        ))
    })?;
    fs::rename(&partial, full_path).map_err(|error| {
        let _ = remove_path_if_exists(&partial);
        PrayError::Resolution(format!(
            "failed to publish git catalog file {}: {error}",
            full_path.display()
        ))
    })
}

fn catalog_matches(dest: &Path, git_dir: &Path, revision: &str) -> bool {
    let recorded = fs::read_to_string(dest.join(REVISION_MARKER)).unwrap_or_default();
    if recorded.trim() != revision {
        return false;
    }
    let recorded_dir = fs::read_to_string(dest.join(GIT_DIR_MARKER)).unwrap_or_default();
    if Path::new(recorded_dir.trim()) != git_dir {
        return false;
    }
    if !git_object_exists(git_dir, revision) {
        return false;
    }
    is_local_distribution_root(dest)
        || is_local_distribution_root(&dest.join("prayers"))
        || dest.read_dir().ok().is_some_and(|entries| {
            entries
                .flatten()
                .any(|entry| is_local_distribution_root(&entry.path()))
        })
}

fn unpack_archive_prefix(
    git_dir: &Path,
    dest: &Path,
    revision: &str,
    prefix: &str,
    offline: bool,
) -> PrayResult<bool> {
    if !require_tree_prefix_within_limits(git_dir, revision, prefix, offline)? {
        return Ok(false);
    }
    let output = run_git_command_with_env(
        git_dir,
        &["archive", "--format=tar", revision, "--", prefix],
        offline_git_env(offline),
    )?;
    if !output.status.success() || output.stdout.is_empty() {
        return Ok(false);
    }
    unpack_bounded_catalog_tar(&output.stdout, dest)?;
    Ok(true)
}

fn staging_catalog_path(dest: &Path) -> PathBuf {
    let mut staging = dest.as_os_str().to_os_string();
    staging.push(".partial");
    PathBuf::from(staging)
}

fn publish_catalog_tree(dest: &Path, staging: &Path) -> PrayResult<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    if dest.exists() {
        let backup = {
            let mut path = dest.as_os_str().to_os_string();
            path.push(".previous");
            PathBuf::from(path)
        };
        if backup.exists() {
            remove_path_if_exists(&backup)?;
        }
        fs::rename(dest, &backup).map_err(|error| {
            PrayError::Resolution(format!(
                "failed to retire previous git catalog {}: {error}",
                dest.display()
            ))
        })?;
        if let Err(error) = fs::rename(staging, dest) {
            let _ = fs::rename(&backup, dest);
            let _ = remove_path_if_exists(staging);
            return Err(PrayError::Resolution(format!(
                "failed to publish git catalog {}: {error}",
                dest.display()
            )));
        }
        remove_path_if_exists(&backup)?;
        return Ok(());
    }
    fs::rename(staging, dest).map_err(|error| {
        PrayError::Resolution(format!(
            "failed to publish git catalog {}: {error}",
            dest.display()
        ))
    })
}
