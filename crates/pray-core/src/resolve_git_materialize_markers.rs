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
            if revision.is_empty() || !git_dir.exists() {
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
