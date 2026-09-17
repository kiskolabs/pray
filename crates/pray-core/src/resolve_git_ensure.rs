use crate::paths::remove_path_if_exists;
use crate::resolve_git::{
    apply_sparse_checkout, checkout_git_revision, ensure_git_remote_origin,
    finalize_git_repository, git_head_revision, mirror_git_cache_to_global, refresh_git_worktree,
    refresh_global_from_project, seed_git_cache_from_global,
};
use crate::resolve_git_command::{run_git_command, run_git_success};
use crate::resolve_git_lookup::is_git_checkout;
use crate::resolve_git_paths::{
    git_source_cache_directory, git_source_cache_directory_with_subdir,
};
use crate::{PrayError, PrayResult};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn ensure_git_repository(
    project_root: &Path,
    clone_url: &str,
    refresh: bool,
    pinned_revision: Option<&str>,
    sparse_subdir: Option<&str>,
) -> PrayResult<(PathBuf, String)> {
    let shared = git_source_cache_directory(project_root, clone_url);
    ensure_shared_git_repository(project_root, clone_url, &shared, refresh, pinned_revision)?;
    let checkout = git_source_cache_directory_with_subdir(project_root, clone_url, sparse_subdir);
    if checkout != shared {
        ensure_linked_worktree(&shared, &checkout)?;
        if let Some(revision) = pinned_revision {
            checkout_git_revision(&checkout, clone_url, revision, refresh)?;
        } else if refresh {
            let shared_head = git_head_revision(&shared)?;
            run_git_success(&checkout, &["reset", "--hard", &shared_head])?;
        }
        if let Some(subdir) = sparse_subdir {
            apply_sparse_checkout(&checkout, subdir)?;
        }
        let revision = git_head_revision(&checkout)?;
        return finalize_git_repository(clone_url, &checkout, revision);
    }
    let revision = git_head_revision(&shared)?;
    finalize_git_repository(clone_url, &shared, revision)
}

fn ensure_shared_git_repository(
    project_root: &Path,
    clone_url: &str,
    shared: &Path,
    refresh: bool,
    pinned_revision: Option<&str>,
) -> PrayResult<()> {
    if shared.join(".git").is_dir() {
        if let Some(revision) = pinned_revision {
            checkout_git_revision(shared, clone_url, revision, refresh)?;
        } else if refresh {
            refresh_git_worktree(shared, clone_url)?;
        }
        if refresh {
            let _ = refresh_global_from_project(clone_url, shared);
        }
        return Ok(());
    }
    if shared.exists() {
        remove_path_if_exists(shared)?;
    }
    if let Some(parent) = shared.parent() {
        fs::create_dir_all(parent)?;
    }
    let destination = shared
        .to_str()
        .ok_or_else(|| PrayError::Resolution(format!("invalid git cache path: {:?}", shared)))?;
    let seeded = seed_git_cache_from_global(clone_url, destination, project_root)?;
    if seeded {
        ensure_git_remote_origin(shared, clone_url)?;
    } else {
        run_git_success(
            project_root,
            &["clone", "--depth", "1", clone_url, destination],
        )?;
        let _ = mirror_git_cache_to_global(clone_url, shared);
    }
    if let Some(revision) = pinned_revision {
        checkout_git_revision(shared, clone_url, revision, true)?;
    } else if refresh && seeded {
        refresh_git_worktree(shared, clone_url)?;
    }
    if refresh && seeded {
        let _ = refresh_global_from_project(clone_url, shared);
    }
    Ok(())
}

fn ensure_linked_worktree(shared: &Path, checkout: &Path) -> PrayResult<()> {
    if same_object_store(shared, checkout) {
        return Ok(());
    }
    if checkout.exists() {
        remove_path_if_exists(checkout)?;
    }
    if let Some(parent) = checkout.parent() {
        fs::create_dir_all(parent)?;
    }
    let destination = checkout.to_str().ok_or_else(|| {
        PrayError::Resolution(format!("invalid git worktree path: {:?}", checkout))
    })?;
    run_git_success(shared, &["worktree", "add", "--detach", destination])?;
    Ok(())
}

fn same_object_store(shared: &Path, checkout: &Path) -> bool {
    if !is_git_checkout(checkout) {
        return false;
    }
    match (git_common_dir(shared), git_common_dir(checkout)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn git_common_dir(repository: &Path) -> PrayResult<PathBuf> {
    let output = run_git_command(repository, &["rev-parse", "--git-common-dir"])?;
    if !output.status.success() {
        return Err(PrayError::Resolution(
            "git rev-parse --git-common-dir failed".to_string(),
        ));
    }
    let reported = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let path = Path::new(&reported);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repository.join(path)
    };
    resolved.canonicalize().map_err(|error| {
        PrayError::Resolution(format!("invalid git-common-dir {reported}: {error}"))
    })
}
