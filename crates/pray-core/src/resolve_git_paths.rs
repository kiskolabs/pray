use crate::hashing::sha256_prefixed;
use std::path::{Path, PathBuf};

pub fn git_source_cache_directory(project_root: &Path, clone_url: &str) -> PathBuf {
    git_source_cache_directory_with_subdir(project_root, clone_url, None)
}

pub fn git_source_cache_directory_with_subdir(
    project_root: &Path,
    clone_url: &str,
    subdir: Option<&str>,
) -> PathBuf {
    project_root
        .join(".pray/cache/git")
        .join(cache_key(&worktree_cache_identity(clone_url, subdir)))
}

pub(crate) fn cache_key(text: &str) -> String {
    sha256_prefixed(text.as_bytes())
        .trim_start_matches("sha256:")
        .chars()
        .take(16)
        .collect()
}

fn worktree_cache_identity(clone_url: &str, subdir: Option<&str>) -> String {
    match subdir {
        Some(subdir) if !subdir.is_empty() => format!("{clone_url}\n{subdir}"),
        _ => clone_url.to_string(),
    }
}
