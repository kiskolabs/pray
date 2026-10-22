use crate::resolve_git_command::run_git_command_with_env;
use crate::resolve_git_materialize_check::require_revision_present;
use crate::resolve_git_materialize_markers::catalog_root_markers;
use crate::resolve_git_materialize_size::require_tree_prefix_within_limits;
use crate::{PrayError, PrayResult};
use std::path::Path;

pub fn confirm_git_source_revisions_materializable(
    project: &crate::resolve::ResolvedProject,
) -> PrayResult<()> {
    for source in &project.manifest.sources {
        if source.kind != "git" {
            continue;
        }
        let Some(revision) = project.source_revisions.get(&source.name) else {
            continue;
        };
        if revision.is_empty() {
            continue;
        }
        let clone_url = source.url.strip_prefix("git+").unwrap_or(&source.url);
        let catalog = crate::resolve_git_paths::git_source_cache_directory_with_subdir(
            &project.project_root,
            clone_url,
            source.subdir.as_deref(),
        );
        confirm_catalog_revision_materializable(&catalog, revision)?;
    }
    Ok(())
}

pub(crate) fn confirm_catalog_revision_materializable(
    catalog: &Path,
    expected_revision: &str,
) -> PrayResult<()> {
    let Some((git_dir, revision, catalog_root)) = catalog_root_markers(catalog) else {
        return Err(PrayError::Resolution(format!(
            "git catalog {} is missing materialize markers for revision {expected_revision}",
            catalog.display()
        )));
    };
    if catalog_root != catalog {
        return Err(PrayError::Resolution(format!(
            "git catalog markers for {} were found at {}, expected at the materialized catalog root",
            catalog.display(),
            catalog_root.display()
        )));
    }
    if revision != expected_revision {
        return Err(PrayError::Resolution(format!(
            "git catalog {} records revision {revision}, expected {expected_revision}",
            catalog.display()
        )));
    }
    require_revision_present(&git_dir, &revision)?;
    if require_tree_prefix_within_limits(&git_dir, &revision, "v1/packages", false)? {
        return Ok(());
    }
    if require_tree_prefix_within_limits(&git_dir, &revision, "prayers/v1/packages", false)? {
        return Ok(());
    }
    // Fallback for older trees: presence probe without buffering the archive body.
    let listed = run_git_command_with_env(
        &git_dir,
        &[
            "ls-tree",
            "-r",
            "--name-only",
            &revision,
            "--",
            "v1/packages",
        ],
        &[],
    )?;
    if listed.status.success() && !String::from_utf8_lossy(&listed.stdout).trim().is_empty() {
        return Ok(());
    }
    Err(PrayError::Resolution(format!(
        "git revision {revision} in {} is not materializable on the catalog path verify uses",
        catalog.display()
    )))
}
