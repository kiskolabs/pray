use crate::resolve_git::{global_git_cache_directory, global_git_cache_ready};
use crate::resolve_git_paths::git_source_cache_directory;
use std::fs;
use std::path::{Path, PathBuf};

const GIT_DIR_MARKER: &str = ".pray-git-dir";

/// Resolve the git object store for trust and diagnostics.
///
/// Order: ready shared bare for the URL, then a project stub's `.pray-git-dir`.
/// Never select a leftover full clone under `.pray/cache/git` by scanning `.git`.
pub fn git_source_cached_repository(project_root: &Path, clone_url: &str) -> Option<PathBuf> {
    if let Some(global) = global_git_cache_directory(clone_url) {
        if global_git_cache_ready(&global) {
            return Some(global);
        }
    }
    stub_git_dir(&git_source_cache_directory(project_root, clone_url))
        .or_else(|| find_stub_git_dir(project_root, clone_url))
}

pub(crate) fn is_git_checkout(path: &Path) -> bool {
    path.join(".git").exists()
}

fn stub_git_dir(catalog: &Path) -> Option<PathBuf> {
    let recorded = fs::read_to_string(catalog.join(GIT_DIR_MARKER)).ok()?;
    let git_dir = PathBuf::from(recorded.trim());
    if git_dir.as_os_str().is_empty() || !git_dir.exists() {
        return None;
    }
    Some(git_dir)
}

fn find_stub_git_dir(project_root: &Path, clone_url: &str) -> Option<PathBuf> {
    let expected = global_git_cache_directory(clone_url)?;
    let cache_root = project_root.join(".pray/cache/git");
    let entries = fs::read_dir(&cache_root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if is_git_checkout(&path) {
            continue;
        }
        let Some(git_dir) = stub_git_dir(&path) else {
            continue;
        };
        if git_dir == expected {
            return Some(git_dir);
        }
    }
    None
}
