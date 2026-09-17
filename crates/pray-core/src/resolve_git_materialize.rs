use crate::resolve_git_command::{run_git_command, run_git_success};
use crate::PrayResult;
use std::path::{Path, PathBuf};

pub(crate) fn materialize_git_catalog_file(source_root: &Path, relative: &Path) -> PrayResult<()> {
    let full_path = source_root.join(relative);
    if full_path.is_file() {
        return Ok(());
    }
    if !sparse_checkout_enabled(source_root) {
        return Ok(());
    }
    let Some(toplevel) = git_toplevel(source_root) else {
        return Ok(());
    };
    let Some(repo_relative) = repo_relative_text(source_root, relative) else {
        return Ok(());
    };
    if let Some(cone) = artifact_sparse_cone(Path::new(&repo_relative)) {
        run_git_success(&toplevel, &["sparse-checkout", "add", &cone])?;
    }
    run_git_success(&toplevel, &["checkout", "HEAD", "--", &repo_relative])?;
    Ok(())
}

fn repo_relative_text(source_root: &Path, relative: &Path) -> Option<String> {
    let prefix = git_show_prefix(source_root)?;
    let combined = if prefix.is_empty() {
        relative.to_path_buf()
    } else {
        Path::new(&prefix).join(relative)
    };
    combined.to_str().map(str::to_string)
}

fn sparse_checkout_enabled(source_root: &Path) -> bool {
    let Ok(output) = run_git_command(source_root, &["config", "--get", "core.sparseCheckout"])
    else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    String::from_utf8_lossy(&output.stdout).trim() == "true"
}

fn git_toplevel(start: &Path) -> Option<PathBuf> {
    let output = run_git_command(start, &["rev-parse", "--show-toplevel"]).ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(PathBuf::from(text))
    }
}

fn git_show_prefix(source_root: &Path) -> Option<String> {
    let output = run_git_command(source_root, &["rev-parse", "--show-prefix"]).ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn artifact_sparse_cone(repo_relative: &Path) -> Option<String> {
    let mut parts: Vec<_> = repo_relative.iter().collect();
    if parts.len() < 2 {
        return repo_relative.to_str().map(str::to_string);
    }
    parts.pop();
    if parts.last().is_some_and(|part| *part != "artifacts") && parts.len() > 3 {
        parts.pop();
    }
    PathBuf::from_iter(parts).to_str().map(str::to_string)
}
