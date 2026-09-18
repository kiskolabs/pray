use crate::paths::remove_path_if_exists;
use crate::resolve_git::{
    ensure_git_remote_origin, git_head_revision, git_object_exists, global_git_cache_directory,
    global_git_cache_ready, offline_git_source_uncached,
};
use crate::resolve_git_clone::clone_bare_git_db;
use crate::resolve_git_command::run_git_success;
use crate::{PrayError, PrayResult};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn ensure_global_git_db(
    clone_url: &str,
    pinned_revision: Option<&str>,
    refresh: bool,
    offline: bool,
    working_directory: &Path,
) -> PrayResult<(PathBuf, String)> {
    let db = global_git_cache_directory(clone_url)
        .ok_or_else(|| PrayError::Resolution("git object cache is not configured".to_string()))?;
    if !global_git_cache_ready(&db) {
        if offline {
            return Err(offline_git_source_uncached(clone_url));
        }
        if db.exists() {
            remove_path_if_exists(&db)?;
        }
        if let Some(parent) = db.parent() {
            fs::create_dir_all(parent)?;
        }
        let destination = db.to_str().ok_or_else(|| {
            PrayError::Resolution(format!("invalid global git cache path: {:?}", db))
        })?;
        clone_bare_git_db(working_directory, clone_url, destination, false)?;
        ensure_git_remote_origin(&db, clone_url)?;
    }
    if refresh && !offline {
        fetch_origin_tip(&db, clone_url)?;
    }
    let revision = if let Some(revision) = pinned_revision {
        ensure_revision_in_db(&db, clone_url, revision, !offline)?;
        revision.to_string()
    } else {
        git_head_revision(&db)?
    };
    Ok((db, revision))
}

fn ensure_revision_in_db(
    db: &Path,
    clone_url: &str,
    revision: &str,
    allow_fetch: bool,
) -> PrayResult<()> {
    if git_object_exists(db, revision) {
        return Ok(());
    }
    if !allow_fetch {
        return Err(PrayError::Resolution(format!(
            "git source {:?} is locked to revision {revision}, but that commit is not available locally and offline mode is enabled",
            db
        )));
    }
    ensure_git_remote_origin(db, clone_url)?;
    if db.join("shallow").is_file() {
        fetch_unshallow(db)?;
    }
    if git_object_exists(db, revision) {
        return Ok(());
    }
    fetch_revision(db, revision)?;
    if git_object_exists(db, revision) {
        return Ok(());
    }
    Err(PrayError::Resolution(format!(
        "git source {:?} is locked to revision {revision}, but that commit could not be fetched",
        db
    )))
}

fn fetch_origin_tip(db: &Path, clone_url: &str) -> PrayResult<()> {
    ensure_git_remote_origin(db, clone_url)?;
    if run_git_success(
        db,
        &["fetch", "--depth", "1", "--filter=blob:none", "origin"],
    )
    .is_err()
    {
        run_git_success(db, &["fetch", "--depth", "1", "origin"])?;
    }
    run_git_success(db, &["update-ref", "HEAD", "FETCH_HEAD"])
}

fn fetch_unshallow(db: &Path) -> PrayResult<()> {
    if run_git_success(
        db,
        &["fetch", "--unshallow", "--filter=blob:none", "origin"],
    )
    .is_ok()
    {
        return Ok(());
    }
    run_git_success(db, &["fetch", "--unshallow", "origin"])
}

fn fetch_revision(db: &Path, revision: &str) -> PrayResult<()> {
    if run_git_success(db, &["fetch", "--filter=blob:none", "origin", revision]).is_ok() {
        return Ok(());
    }
    run_git_success(db, &["fetch", "origin", revision])
}
