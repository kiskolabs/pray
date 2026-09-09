use crate::invocation;
use pray_core::lockfile::Lockfile;
use pray_core::manifest::{parse_manifest, read_manifest_text};
use pray_core::resolve::{
    annotate_failed_git_refresh, resolution_may_benefit_from_git_source_refresh, ResolvedProject,
};
use pray_core::resolve_context::ResolveOptions;
use pray_core::{PrayError, PrayResult};
use std::path::{Path, PathBuf};

pub(crate) fn manifest_path() -> PathBuf {
    invocation::manifest_path()
}

pub(crate) fn lockfile_path() -> PathBuf {
    invocation::lockfile_path()
}

pub(crate) fn workspace_root() -> PathBuf {
    invocation::project_root()
}

pub(crate) fn resolve_project_with_options(
    _manifest_path: &Path,
    options: &ResolveOptions,
) -> PrayResult<ResolvedProject> {
    invocation::resolve_current_project(options)
}

pub(crate) fn resolve_project(_manifest_path: &Path) -> PrayResult<ResolvedProject> {
    invocation::resolve_current_project(&ResolveOptions::default())
}

pub(crate) fn resolve_project_with_git_refresh_fallback(
    _manifest_path: &Path,
    options: &ResolveOptions,
    allow_git_refresh_fallback: bool,
) -> PrayResult<ResolvedProject> {
    match invocation::resolve_current_project(options) {
        Ok(project) => Ok(project),
        Err(PrayError::Resolution(message))
            if allow_git_refresh_fallback
                && !options.offline
                && !options.refresh_source_revisions
                && resolution_may_benefit_from_git_source_refresh(&message) =>
        {
            let refreshed_options = ResolveOptions {
                refresh_source_revisions: true,
                ..options.clone()
            };
            match invocation::resolve_current_project(&refreshed_options) {
                Ok(project) => Ok(project),
                Err(error) => Err(annotate_failed_git_refresh(&lockfile_path(), error)),
            }
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn load_manifest() -> PrayResult<pray_core::manifest::Manifest> {
    let text = read_manifest_text(&manifest_path())?;
    let manifest = parse_manifest(&text)?;
    for warning in manifest.deprecation_warnings() {
        eprintln!("{warning}");
    }
    Ok(manifest)
}

pub(crate) fn default_output_for_target(target: &str) -> String {
    match target {
        "tool_a" => "INSTRUCTIONS".to_string(),
        "tool_b" => "TOOL_B".to_string(),
        other => other.to_uppercase(),
    }
}

pub(crate) fn locked_package<'a>(
    lockfile: &'a Lockfile,
    package: &pray_core::resolve::ResolvedPackage,
) -> Option<&'a pray_core::lockfile::LockedPackage> {
    lockfile.package.iter().find(|record| {
        record.name == package.declaration.name
            && record.source.as_deref() == package.declaration.source.as_deref()
    })
}

pub(crate) fn run_project_command<T>(
    arguments: &[String],
    execute: impl FnOnce() -> PrayResult<T>,
) -> PrayResult<T> {
    let transactional = arguments.first().is_some_and(|command| {
        matches!(
            command.as_str(),
            "install"
                | "apply"
                | "update"
                | "unlock"
                | "add"
                | "remove"
                | "render"
                | "plan"
                | "verify"
                | "drift"
        )
    });
    if transactional {
        pray_core::transaction::run(&invocation::project_root(), execute)
    } else {
        execute()
    }
}
