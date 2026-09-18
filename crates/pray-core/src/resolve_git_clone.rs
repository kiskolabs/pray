use crate::resolve_git_command::run_git_success;
use crate::PrayResult;
use std::path::Path;

pub(crate) fn clone_bare_git_db(
    working_directory: &Path,
    source: &str,
    destination: &str,
    quiet: bool,
) -> PrayResult<()> {
    let mut filtered = vec!["clone", "--bare", "--depth", "1", "--filter=blob:none"];
    if source.starts_with("file://") || Path::new(source).is_absolute() {
        filtered.push("--no-local");
    }
    if quiet {
        filtered.insert(1, "--quiet");
    }
    filtered.push(source);
    filtered.push(destination);
    if run_git_success(working_directory, &filtered).is_ok() {
        return Ok(());
    }
    let mut full = vec!["clone", "--bare", "--depth", "1"];
    if source.starts_with("file://") || Path::new(source).is_absolute() {
        full.push("--no-local");
    }
    if quiet {
        full.insert(1, "--quiet");
    }
    full.push(source);
    full.push(destination);
    run_git_success(working_directory, &full)
}

pub(crate) fn catalog_sparse_cones(subdir: Option<&str>) -> Vec<String> {
    match subdir {
        Some(subdir) if !subdir.is_empty() => vec![format!("{subdir}/v1/packages")],
        _ => vec!["v1/packages".to_string(), "prayers/v1/packages".to_string()],
    }
}
