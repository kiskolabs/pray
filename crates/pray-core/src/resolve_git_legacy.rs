use crate::paths::remove_path_if_exists;
use crate::resolve_git_command::run_git_command;
use crate::resolve_git_lookup::is_git_checkout;
use crate::PrayResult;
use std::fs;
use std::path::Path;

/// Rename project-local full git clones for `clone_url` so they cannot win
/// lookup over the shared bare store or a stub catalog.
pub(crate) fn quarantine_legacy_project_git_checkouts(
    project_root: &Path,
    clone_url: &str,
) -> PrayResult<()> {
    let _ = quarantine_legacy_git_checkouts(project_root, clone_url)?;
    Ok(())
}

/// Same as install quarantine; returns how many checkouts were renamed.
pub fn quarantine_legacy_git_checkouts(project_root: &Path, clone_url: &str) -> PrayResult<usize> {
    let cache_root = project_root.join(".pray/cache/git");
    let Ok(entries) = fs::read_dir(&cache_root) else {
        return Ok(0);
    };
    let mut migrated = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.ends_with(".legacy-git") {
            continue;
        }
        if !is_git_checkout(&path) {
            continue;
        }
        if !origin_matches(&path, clone_url) {
            continue;
        }
        let legacy = path.with_file_name(format!("{name}.legacy-git"));
        if legacy.exists() {
            remove_path_if_exists(&legacy)?;
        }
        fs::rename(&path, &legacy)?;
        migrated += 1;
    }
    Ok(migrated)
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
