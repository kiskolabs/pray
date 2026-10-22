use crate::resolve_git::global_cache_root;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) const REVISION_MARKER: &str = ".pray-revision";
pub(crate) const GIT_DIR_MARKER: &str = ".pray-git-dir";

pub(crate) fn catalog_root_markers(start: &Path) -> Option<(PathBuf, String, PathBuf)> {
    let mut current = start.to_path_buf();
    for _ in 0..8 {
        let revision_path = current.join(REVISION_MARKER);
        let git_dir_path = current.join(GIT_DIR_MARKER);
        if revision_path.is_file() && git_dir_path.is_file() {
            let revision = fs::read_to_string(revision_path).ok()?.trim().to_string();
            let git_dir = PathBuf::from(fs::read_to_string(git_dir_path).ok()?.trim());
            if !acceptable_revision(&revision) || !acceptable_shared_bare(&git_dir) {
                return None;
            }
            // Markers identify the materialized catalog root. Do not treat
            // intermediate directories as work trees for shared-bare checkout.
            return Some((git_dir, revision, current));
        }
        current = current.parent()?.to_path_buf();
    }
    None
}

pub(crate) fn offline_git_env(offline: bool) -> &'static [(&'static str, &'static str)] {
    const OFFLINE: &[(&str, &str)] = &[("GIT_NO_LAZY_FETCH", "1")];
    if offline {
        OFFLINE
    } else {
        &[]
    }
}

fn acceptable_revision(revision: &str) -> bool {
    if revision.is_empty() || revision.starts_with('-') {
        return false;
    }
    let len = revision.len();
    if len != 40 && len != 64 {
        return false;
    }
    revision.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn acceptable_shared_bare(git_dir: &Path) -> bool {
    if !git_dir.join("HEAD").is_file() || !git_dir.join("objects").is_dir() {
        return false;
    }
    let Some(cache_root) = global_cache_root() else {
        return false;
    };
    let expected_root = cache_root.join("git");
    path_under_root(&expected_root, git_dir)
}

fn path_under_root(root: &Path, candidate: &Path) -> bool {
    if candidate == root {
        return true;
    }
    match (fs::canonicalize(root), fs::canonicalize(candidate)) {
        (Ok(root), Ok(candidate)) => candidate.starts_with(&root),
        _ => candidate.starts_with(root),
    }
}

#[cfg(test)]
mod tests {
    use super::acceptable_revision;

    #[test]
    fn accepts_hex_revisions_only() {
        assert!(acceptable_revision(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
        ));
        assert!(!acceptable_revision(
            "-deadbeefdeadbeefdeadbeefdeadbeefdeadbee"
        ));
        assert!(!acceptable_revision("not-a-revision"));
        assert!(!acceptable_revision(""));
    }
}
