use crate::resolve_git::git_object_exists;
use crate::{PrayError, PrayResult};
use std::path::Path;

pub(crate) fn require_revision_present(git_dir: &Path, revision: &str) -> PrayResult<()> {
    if git_object_exists(git_dir, revision) {
        return Ok(());
    }
    Err(PrayError::Resolution(format!(
        "git revision {revision} is not available in {} (git_revision_unfetched). \
         Run `pray install --locked`, or clear the project git cache and shared bare then install again.",
        git_dir.display()
    )))
}

pub(crate) fn annotate_git_materialize_failure(
    git_dir: &Path,
    revision: &str,
    error: PrayError,
) -> PrayError {
    PrayError::Resolution(format!(
        "git catalog materialize failed for revision {revision} in {} (git_cache_stale): {error}. \
         Run `pray install --locked`, or clear `.pray/cache/git` and the shared bare then install again.",
        git_dir.display()
    ))
}
