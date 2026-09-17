use crate::resolve_git_command::run_git_command;
use crate::resolve_git_paths::git_source_cache_directory;
use std::fs;
use std::path::{Path, PathBuf};

pub fn git_source_cached_repository(project_root: &Path, clone_url: &str) -> Option<PathBuf> {
    let shared = git_source_cache_directory(project_root, clone_url);
    if is_git_checkout(&shared) {
        return Some(shared);
    }
    find_cached_origin(project_root, clone_url)
}

pub(crate) fn is_git_checkout(path: &Path) -> bool {
    path.join(".git").exists()
}

fn find_cached_origin(project_root: &Path, clone_url: &str) -> Option<PathBuf> {
    let cache_root = project_root.join(".pray/cache/git");
    let entries = fs::read_dir(&cache_root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !is_git_checkout(&path) {
            continue;
        }
        if origin_matches(&path, clone_url) {
            return Some(path);
        }
    }
    None
}

fn origin_matches(repository: &Path, clone_url: &str) -> bool {
    let Ok(output) = run_git_command(repository, &["remote", "get-url", "origin"]) else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let origin = String::from_utf8_lossy(&output.stdout);
    let origin = origin.trim().strip_prefix("git+").unwrap_or(origin.trim());
    origin == clone_url
}
