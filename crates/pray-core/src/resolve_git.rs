use crate::client_trust::{effective_trust_home, gate_git_source};
use crate::resolve_git_command::{command_error, run_git_command, run_git_success};
use crate::resolve_git_paths::cache_key;
use crate::{PrayError, PrayResult};
use std::fs;
use std::path::{Path, PathBuf};

pub use crate::resolve_git_paths::git_source_cache_directory;

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

const READY_MARKER: &str = ".pray-ready";

pub fn global_git_cache_ready(global_cache: &Path) -> bool {
    if !global_cache.join("HEAD").is_file() || !global_cache.join("objects").is_dir() {
        return false;
    }
    if global_cache.join(READY_MARKER).is_file() {
        return true;
    }
    // Legacy caches written before the ready marker: require a resolvable HEAD.
    git_head_revision(global_cache).is_ok()
}

pub(crate) fn mark_git_cache_ready(global_cache: &Path) -> PrayResult<()> {
    fs::write(global_cache.join(READY_MARKER), "1").map_err(|error| {
        PrayError::Resolution(format!(
            "failed to mark git cache ready at {}: {error}",
            global_cache.display()
        ))
    })
}

pub(crate) fn offline_git_source_uncached(clone_url: &str) -> PrayError {
    PrayError::Resolution(format!(
        "git source {clone_url} is not cached locally and offline mode is enabled"
    ))
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
    trust_repository: &Path,
    catalog: PathBuf,
    revision: String,
) -> PrayResult<(PathBuf, String)> {
    gate_git_source(
        &effective_trust_home()?,
        clone_url,
        trust_repository,
        Some(&revision),
    )?;
    if crate::client_trust::env_truthy("PRAY_TRUST_IMPORT") {
        let global_scope = crate::client_trust::env_truthy("PRAY_TRUST_GLOBAL");
        crate::client_trust::prompt_import_signing_keys_for_source(
            &effective_trust_home()?,
            clone_url,
            trust_repository,
            global_scope,
        )?;
    }
    Ok((catalog, revision))
}

pub(crate) fn ensure_git_remote_origin(repository: &Path, clone_url: &str) -> PrayResult<()> {
    require_own_git_dir(repository)?;
    if run_git_success(repository, &["remote", "get-url", "origin"]).is_ok() {
        run_git_success(repository, &["remote", "set-url", "origin", clone_url])?;
    } else {
        run_git_success(repository, &["remote", "add", "origin", clone_url])?;
    }
    Ok(())
}

// Git searches parent directories when `repository` is not a valid git
// directory, so a damaged cache inside another checkout would otherwise
// rewrite that checkout's origin.
fn require_own_git_dir(repository: &Path) -> PrayResult<()> {
    let output = run_git_command(repository, &["rev-parse", "--absolute-git-dir"])?;
    let resolved = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let resolved_path = Path::new(&resolved).canonicalize().ok();
    let own = [repository.to_path_buf(), repository.join(".git")]
        .iter()
        .any(|candidate| candidate.canonicalize().ok() == resolved_path);
    if output.status.success() && own {
        return Ok(());
    }
    Err(PrayError::Resolution(format!(
        "git cache {} is not a usable git repository (git resolved {:?}). Remove it and run the command again.",
        repository.display(),
        resolved
    )))
}

pub(crate) fn git_object_exists(repository: &Path, object: &str) -> bool {
    git_object_exists_offline(repository, object, false)
}

pub(crate) fn git_object_exists_offline(repository: &Path, object: &str, offline: bool) -> bool {
    use crate::resolve_git_command::run_git_command_with_env;
    use crate::resolve_git_materialize_markers::offline_git_env;
    let Ok(output) = run_git_command_with_env(
        repository,
        &["cat-file", "-e", object],
        offline_git_env(offline),
    ) else {
        return false;
    };
    output.status.success()
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
