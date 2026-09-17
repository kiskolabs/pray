use crate::resolve_git_command::run_git_success;
use crate::PrayResult;
use std::path::Path;

pub(crate) fn clone_git_cache(
    working_directory: &Path,
    source: &str,
    destination: &str,
    quiet: bool,
) -> PrayResult<()> {
    let mut filtered = vec![
        "clone",
        "--depth",
        "1",
        "--filter=blob:none",
        "--sparse",
        source,
        destination,
    ];
    if quiet {
        filtered.insert(1, "--quiet");
    }
    if run_git_success(working_directory, &filtered).is_ok() {
        return Ok(());
    }
    let mut full = vec!["clone", "--depth", "1", source, destination];
    if quiet {
        full.insert(1, "--quiet");
    }
    run_git_success(working_directory, &full)
}

pub(crate) fn apply_sparse_checkout(repository: &Path, subdir: Option<&str>) -> PrayResult<()> {
    run_git_success(repository, &["sparse-checkout", "init", "--cone"])?;
    let cones = catalog_sparse_cones(subdir);
    let mut arguments = vec!["sparse-checkout".to_string(), "set".to_string()];
    arguments.extend(cones);
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    run_git_success(repository, &borrowed)
}

fn catalog_sparse_cones(subdir: Option<&str>) -> Vec<String> {
    match subdir {
        Some(subdir) if !subdir.is_empty() => vec![format!("{subdir}/v1/packages")],
        _ => vec!["v1/packages".to_string(), "prayers/v1/packages".to_string()],
    }
}
