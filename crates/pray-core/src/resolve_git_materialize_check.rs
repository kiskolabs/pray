use crate::resolve_git::git_object_exists_offline;
use crate::{PrayError, PrayResult};
use std::path::Path;

pub(crate) fn require_revision_present(git_dir: &Path, revision: &str) -> PrayResult<()> {
    require_revision_present_offline(git_dir, revision, false)
}

pub(crate) fn require_revision_present_offline(
    git_dir: &Path,
    revision: &str,
    offline: bool,
) -> PrayResult<()> {
    if git_object_exists_offline(git_dir, revision, offline) {
        return Ok(());
    }
    if offline {
        return Err(PrayError::Resolution(format!(
            "git revision {revision} is not available in {} and offline mode is enabled (git_revision_unfetched).",
            git_dir.display()
        )));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn missing_revision_names_git_revision_unfetched() {
        let error = require_revision_present_offline(
            Path::new("/tmp/pray-missing-git-db"),
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            false,
        )
        .expect_err("missing object");
        let text = error.to_string();
        assert!(text.contains("git_revision_unfetched"), "{text}");
        assert!(text.contains("pray install --locked"), "{text}");
    }

    #[test]
    fn materialize_failure_names_git_cache_stale() {
        let error = annotate_git_materialize_failure(
            Path::new("/tmp/pray-stale-git-db"),
            "abc123",
            PrayError::Resolution("git archive failed".into()),
        );
        let text = error.to_string();
        assert!(text.contains("git_cache_stale"), "{text}");
        assert!(text.contains("pray install --locked"), "{text}");
    }
}
