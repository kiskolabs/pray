use crate::paths::remove_path_if_exists;
use crate::resolve_git::is_local_distribution_root;
use crate::resolve_git_clone::catalog_sparse_cones;
use crate::resolve_git_command::{run_git_command, run_git_success};
use crate::{PrayError, PrayResult};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

const REVISION_MARKER: &str = ".pray-revision";
const GIT_DIR_MARKER: &str = ".pray-git-dir";

pub(crate) fn materialize_catalog_tree(
    git_dir: &Path,
    dest: &Path,
    revision: &str,
    subdir: Option<&str>,
    refresh: bool,
) -> PrayResult<()> {
    if !refresh && catalog_matches(dest, git_dir, revision) {
        return Ok(());
    }
    if dest.exists() {
        remove_path_if_exists(dest)?;
    }
    fs::create_dir_all(dest)?;
    let mut unpacked = false;
    for prefix in catalog_sparse_cones(subdir) {
        if unpack_archive_prefix(git_dir, dest, revision, &prefix)? {
            unpacked = true;
        }
    }
    if !unpacked {
        return Err(PrayError::Resolution(format!(
            "no pray distribution root in git source at revision {revision}"
        )));
    }
    fs::write(dest.join(REVISION_MARKER), revision)?;
    let git_dir_text = git_dir.to_str().ok_or_else(|| {
        PrayError::Resolution(format!("invalid global git cache path: {:?}", git_dir))
    })?;
    fs::write(dest.join(GIT_DIR_MARKER), git_dir_text)?;
    Ok(())
}

pub(crate) fn materialize_git_catalog_file(source_root: &Path, relative: &Path) -> PrayResult<()> {
    let full_path = source_root.join(relative);
    if full_path.is_file() {
        return Ok(());
    }
    let Some((git_dir, revision, work_tree)) = find_catalog_markers(source_root) else {
        return Ok(());
    };
    let Ok(repo_relative) = full_path.strip_prefix(&work_tree) else {
        return Ok(());
    };
    let Some(repo_relative) = repo_relative.to_str() else {
        return Ok(());
    };
    let work_tree_text = work_tree.to_str().ok_or_else(|| {
        PrayError::Resolution(format!("invalid catalog cache path: {:?}", work_tree))
    })?;
    run_git_success(
        &git_dir,
        &[
            "--work-tree",
            work_tree_text,
            "checkout",
            &revision,
            "--",
            repo_relative,
        ],
    )
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
) -> PrayResult<bool> {
    let output = run_git_command(
        git_dir,
        &["archive", "--format=tar", revision, "--", prefix],
    )?;
    if !output.status.success() || output.stdout.is_empty() {
        return Ok(false);
    }
    tar::Archive::new(Cursor::new(output.stdout))
        .unpack(dest)
        .map_err(|error| {
            PrayError::Resolution(format!("failed to unpack git catalog archive: {error}"))
        })?;
    Ok(true)
}

fn find_catalog_markers(start: &Path) -> Option<(PathBuf, String, PathBuf)> {
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
            return Some((git_dir, revision, current));
        }
        current = current.parent()?.to_path_buf();
    }
    None
}
