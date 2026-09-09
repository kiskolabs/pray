use crate::lockfile::read_lockfile;
use crate::PrayError;
use std::path::Path;

pub fn resolution_may_benefit_from_git_source_refresh(message: &str) -> bool {
    message.contains("no registry version")
        || message.contains("not found in distribution")
        || message.contains("not found in git source")
        || message.contains("v1/packages/")
}

pub fn annotate_missing_git_catalog(
    error: PrayError,
    package_name: &str,
    source_name: &str,
    revision: &str,
) -> PrayError {
    match error {
        PrayError::Resolution(message)
            if !revision.is_empty()
                && resolution_may_benefit_from_git_source_refresh(&message)
                && !message.contains("not found in git source") =>
        {
            PrayError::Resolution(format!(
                "package {package_name} was not found in git source {source_name} at revision {revision}. \
                 Run `pray update` to advance the source pin."
            ))
        }
        other => other,
    }
}

pub fn annotate_failed_git_refresh(lockfile_path: &Path, error: PrayError) -> PrayError {
    match error {
        PrayError::Resolution(message)
            if resolution_may_benefit_from_git_source_refresh(&message) =>
        {
            match locked_git_revision_guidance(lockfile_path) {
                Some(pins) => {
                    let package =
                        package_name_from_catalog_miss(&message).unwrap_or("the declared package");
                    PrayError::Resolution(format!(
                        "package {package} was not found in locked git source ({pins}). \
                         Run `pray update` to advance the source pin."
                    ))
                }
                None => PrayError::Resolution(message),
            }
        }
        other => other,
    }
}

fn locked_git_revision_guidance(lockfile_path: &Path) -> Option<String> {
    let lockfile = read_lockfile(lockfile_path).ok()?;
    let pins: Vec<String> = lockfile
        .source
        .iter()
        .filter(|source| source.kind == "git")
        .filter_map(|source| {
            source
                .revision
                .as_ref()
                .map(|revision| format!("{} at {revision}", source.name))
        })
        .collect();
    if pins.is_empty() {
        None
    } else {
        Some(pins.join(", "))
    }
}

fn package_name_from_catalog_miss(message: &str) -> Option<&str> {
    let rest = message.strip_prefix("package ")?;
    let end = rest
        .find(" was not found")
        .or_else(|| rest.find(" not found"))?;
    let name = rest.get(..end)?.trim();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_catalog_miss_messages() {
        assert!(resolution_may_benefit_from_git_source_refresh(
            "no registry version for sample/base satisfies ~> 2.0"
        ));
        assert!(resolution_may_benefit_from_git_source_refresh(
            "package sample/extra not found in distribution /tmp/prayers. Missing /tmp/prayers/v1/packages/sample/extra.json. Check the package name."
        ));
        assert!(resolution_may_benefit_from_git_source_refresh(
            "package sample/extra was not found in git source dist at revision abc. Run `pray update` to advance the source pin."
        ));
        assert!(!resolution_may_benefit_from_git_source_refresh(
            "failed to read package metadata"
        ));
    }

    #[test]
    fn annotates_git_catalog_miss_with_update_guidance() {
        let error = PrayError::Resolution(
            "package sample/extra not found in distribution /cache. \
             Missing /cache/v1/packages/sample/extra.json. Check the package name, version constraint `~> 1.0`, and that the source publishes registry metadata."
                .to_string(),
        );
        let annotated = annotate_missing_git_catalog(error, "sample/extra", "dist", "abc123");
        let text = annotated.to_string();
        assert!(text.contains("sample/extra"));
        assert!(text.contains("dist"));
        assert!(text.contains("abc123"));
        assert!(text.contains("pray update"));
        assert!(!text.contains("check the package name"));
    }
}
