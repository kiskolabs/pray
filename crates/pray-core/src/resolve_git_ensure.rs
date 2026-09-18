use crate::resolve_git::finalize_git_repository;
use crate::resolve_git_materialize::materialize_catalog_tree;
use crate::resolve_git_paths::git_source_cache_directory_with_subdir;
use crate::resolve_git_store::ensure_global_git_db;
use crate::PrayResult;
use std::path::{Path, PathBuf};

pub(crate) fn ensure_git_repository(
    project_root: &Path,
    clone_url: &str,
    refresh: bool,
    pinned_revision: Option<&str>,
    sparse_subdir: Option<&str>,
    offline: bool,
) -> PrayResult<(PathBuf, String)> {
    let (db, revision) =
        ensure_global_git_db(clone_url, pinned_revision, refresh, offline, project_root)?;
    let catalog = git_source_cache_directory_with_subdir(project_root, clone_url, sparse_subdir);
    materialize_catalog_tree(&db, &catalog, &revision, sparse_subdir, refresh)?;
    finalize_git_repository(clone_url, &db, catalog, revision)
}
