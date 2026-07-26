use crate::client_trust::{effective_trust_home, gate_git_source};
use crate::hashing::sha256_prefixed;
use crate::paths::remove_path_if_exists;
use crate::resolve_git_command::{command_error, run_git_command, run_git_success};
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
    let git_cache_directory = git_source_cache_directory(project_root, clone_url);

    if git_cache_directory.join(".git").is_dir() {
        if refresh {
            refresh_global_git_cache(clone_url)?;
        }
        if let Some(revision) = pinned_revision {
            checkout_git_revision(&git_cache_directory, clone_url, revision, refresh)?;
        } else if refresh {
            refresh_git_worktree(&git_cache_directory, clone_url)?;
        }
        if let Some(subdir) = sparse_subdir {
            apply_sparse_checkout(&git_cache_directory, subdir)?;
        }
        let revision = git_head_revision(&git_cache_directory)?;
        return finalize_git_repository(clone_url, &git_cache_directory, revision);
    }

    if git_cache_directory.exists() {
        remove_path_if_exists(&git_cache_directory)?;
    }
    if let Some(parent) = git_cache_directory.parent() {
        fs::create_dir_all(parent)?;
    }
    let destination = git_cache_directory.to_str().ok_or_else(|| {
        PrayError::Resolution(format!("invalid git cache path: {:?}", git_cache_directory))
    })?;
    if seed_git_cache_from_global(clone_url, destination, project_root)? {
        ensure_git_remote_origin(&git_cache_directory, clone_url)?;
    } else {
        run_git_success(
            project_root,
            &["clone", "--depth", "1", clone_url, destination],
        )?;
        let _ = mirror_git_cache_to_global(clone_url, &git_cache_directory);
    }
    if let Some(revision) = pinned_revision {
        checkout_git_revision(&git_cache_directory, clone_url, revision, true)?;
    }
    if let Some(subdir) = sparse_subdir {
        apply_sparse_checkout(&git_cache_directory, subdir)?;
    }
    let revision = git_head_revision(&git_cache_directory)?;
    finalize_git_repository(clone_url, &git_cache_directory, revision)
}

pub(crate) fn global_cache_root() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("PRAY_CACHE") {
        return Some(PathBuf::from(path));
    }
    if let Ok(home) = std::env::var("PRAY_HOME") {
        return Some(PathBuf::from(home).join("cache"));
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache").join("pray"))
}

pub(crate) fn global_git_cache_directory(clone_url: &str) -> Option<PathBuf> {
    global_cache_root().map(|root| root.join("git").join(cache_key(clone_url)))
}

pub(crate) fn global_git_cache_ready(global_cache: &Path) -> bool {
    global_cache.join(".git").is_dir() || global_cache.join("HEAD").is_file()
}

pub(crate) fn seed_git_cache_from_global(
    clone_url: &str,
    destination: &str,
    working_directory: &Path,
) -> PrayResult<bool> {
    let Some(global_cache) = global_git_cache_directory(clone_url) else {
        return Ok(false);
    };
    if !global_git_cache_ready(&global_cache) {
        return Ok(false);
    }
    let global_path = global_cache.to_str().ok_or_else(|| {
        PrayError::Resolution(format!("invalid global git cache path: {:?}", global_cache))
    })?;
    run_git_success(
        working_directory,
        &["clone", "--depth", "1", "--quiet", global_path, destination],
    )?;
    Ok(true)
}

pub(crate) fn mirror_git_cache_to_global(clone_url: &str, project_cache: &Path) -> PrayResult<()> {
    let Some(global_cache) = global_git_cache_directory(clone_url) else {
        return Ok(());
    };
    if global_git_cache_ready(&global_cache) {
        return Ok(());
    }
    let cache_parent = project_cache.parent().ok_or_else(|| {
        PrayError::Resolution(format!(
            "invalid project git cache path: {:?}",
            project_cache
        ))
    })?;
    let cache_name = project_cache
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            PrayError::Resolution(format!(
                "invalid project git cache path: {:?}",
                project_cache
            ))
        })?;
    if let Some(parent) = global_cache.parent() {
        fs::create_dir_all(parent)?;
    }
    let destination = global_cache.to_str().ok_or_else(|| {
        PrayError::Resolution(format!("invalid global git cache path: {:?}", global_cache))
    })?;
    if global_cache.exists() {
        remove_path_if_exists(&global_cache)?;
    }
    run_git_success(
        cache_parent,
        &["clone", "--bare", "--quiet", cache_name, destination],
    )?;
    Ok(())
}

pub(crate) fn apply_sparse_checkout(repository: &Path, subdir: &str) -> PrayResult<()> {
    run_git_success(repository, &["sparse-checkout", "init", "--cone"])?;
    run_git_success(repository, &["sparse-checkout", "set", subdir])?;
    Ok(())
}

pub(crate) fn resolve_distribution_root(
    repo_root: &Path,
    subdir: Option<&str>,
) -> PrayResult<PathBuf> {
    if let Some(subdir) = subdir {
        let path = repo_root.join(subdir);
        if is_local_distribution_root(&path) {
            return Ok(path);
        }
        return Err(PrayError::Resolution(format!(
            "no pray distribution root at subdir {:?} in git source {:?}",
            path, repo_root
        )));
    }
    require_distribution_root(repo_root)
}

pub(crate) fn finalize_git_repository(
    clone_url: &str,
    git_cache_directory: &Path,
    revision: String,
) -> PrayResult<(PathBuf, String)> {
    gate_git_source(&effective_trust_home()?, clone_url, git_cache_directory)?;
    if crate::client_trust::env_truthy("PRAY_TRUST_IMPORT") {
        let global_scope = crate::client_trust::env_truthy("PRAY_TRUST_GLOBAL");
        crate::client_trust::prompt_import_signing_keys_for_source(
            &effective_trust_home()?,
            clone_url,
            git_cache_directory,
            global_scope,
        )?;
    }
    Ok((git_cache_directory.to_path_buf(), revision))
}

pub fn git_source_cache_directory(project_root: &Path, clone_url: &str) -> PathBuf {
    project_root
        .join(".pray/cache/git")
        .join(cache_key(clone_url))
}

pub(crate) fn ensure_git_remote_origin(repository: &Path, clone_url: &str) -> PrayResult<()> {
    if run_git_success(repository, &["remote", "get-url", "origin"]).is_ok() {
        run_git_success(repository, &["remote", "set-url", "origin", clone_url])?;
    } else {
        run_git_success(repository, &["remote", "add", "origin", clone_url])?;
    }
    Ok(())
}

pub(crate) fn refresh_global_git_cache(clone_url: &str) -> PrayResult<()> {
    let Some(global_cache) = global_git_cache_directory(clone_url) else {
        return Ok(());
    };
    if !global_git_cache_ready(&global_cache) {
        return Ok(());
    }
    ensure_git_remote_origin(&global_cache, clone_url)?;
    run_git_success(&global_cache, &["fetch", "--depth", "1", "origin"])?;
    Ok(())
}

pub(crate) fn refresh_git_worktree(repository: &Path, clone_url: &str) -> PrayResult<()> {
    ensure_git_remote_origin(repository, clone_url)?;
    run_git_success(repository, &["fetch", "--depth", "1", "origin"])?;
    run_git_success(repository, &["reset", "--hard", "FETCH_HEAD"])?;
    Ok(())
}

pub(crate) fn checkout_git_revision(
    repository: &Path,
    clone_url: &str,
    revision: &str,
    allow_fetch: bool,
) -> PrayResult<()> {
    if git_object_exists(repository, revision) {
        run_git_success(repository, &["reset", "--hard", revision])?;
        return Ok(());
    }
    if !allow_fetch {
        return Err(PrayError::Resolution(format!(
            "git source {:?} is locked to revision {revision}, but that commit is not available locally; rerun pray install without --locked to refresh the cache",
            repository
        )));
    }
    ensure_git_remote_origin(repository, clone_url)?;
    run_git_success(repository, &["fetch", "--depth", "1", "origin", revision])?;
    if git_object_exists(repository, revision) {
        run_git_success(repository, &["reset", "--hard", revision])?;
        return Ok(());
    }
    run_git_success(repository, &["fetch", "origin", revision])?;
    run_git_success(repository, &["reset", "--hard", revision])?;
    Ok(())
}

pub(crate) fn git_object_exists(repository: &Path, object: &str) -> bool {
    run_git_success(repository, &["cat-file", "-e", object]).is_ok()
}

pub(crate) fn git_head_revision(repository: &Path) -> PrayResult<String> {
    let output = run_git_command(repository, &["rev-parse", "HEAD"])?;
    if !output.status.success() {
        return Err(command_error("git rev-parse HEAD", output));
    }
    let revision = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if revision.is_empty() {
        return Err(PrayError::Resolution(
            "git repository has no HEAD revision".to_string(),
        ));
    }
    Ok(revision)
}

pub(crate) fn require_distribution_root(repo_root: &Path) -> PrayResult<PathBuf> {
    discover_distribution_root(repo_root).ok_or_else(|| {
        PrayError::Resolution(format!(
            "no pray distribution root in git source {:?}. \
             Expected v1/packages at the repository root or under prayers/. \
             Publish with `pray publish --root ./prayers` or point the source at a distribution repository.",
            repo_root
        ))
    })
}

pub(crate) fn local_git_source_root(clone_url: &str) -> Option<PathBuf> {
    let path = if let Some(path) = clone_url.strip_prefix("file://") {
        PathBuf::from(path)
    } else {
        PathBuf::from(clone_url)
    };

    if !path.exists() {
        return None;
    }
    discover_distribution_root(&path)
}

pub fn discover_distribution_root(path: &Path) -> Option<PathBuf> {
    if is_local_distribution_root(path) {
        return Some(path.to_path_buf());
    }

    let prayers_root = path.join("prayers");
    if is_local_distribution_root(&prayers_root) {
        return Some(prayers_root);
    }

    None
}

pub(crate) fn is_local_distribution_root(path: &Path) -> bool {
    path.join("v1/packages").is_dir()
}

pub(crate) fn cache_key(text: &str) -> String {
    sha256_prefixed(text.as_bytes())
        .trim_start_matches("sha256:")
        .chars()
        .take(16)
        .collect()
}
