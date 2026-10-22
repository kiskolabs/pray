use crate::resolve_git_command::run_git_command_with_env;
use crate::resource_limits::{
    MAX_ARCHIVE_ENTRIES, MAX_ARCHIVE_ENTRY_BYTES, MAX_ARCHIVE_TOTAL_BYTES,
};
use crate::{PrayError, PrayResult};
use std::path::Path;

pub(crate) fn require_blob_within_limit(
    git_dir: &Path,
    object: &str,
    offline: bool,
) -> PrayResult<()> {
    let output = run_git_command_with_env(
        git_dir,
        &["cat-file", "-s", object],
        crate::resolve_git_materialize_markers::offline_git_env(offline),
    )?;
    if !output.status.success() {
        return Ok(());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let size: u64 = text.trim().parse().map_err(|_| {
        PrayError::Integrity(format!(
            "invalid git blob size for {object}: {}",
            text.trim()
        ))
    })?;
    if size > MAX_ARCHIVE_ENTRY_BYTES {
        return Err(PrayError::Integrity(format!(
            "git catalog blob exceeds {MAX_ARCHIVE_ENTRY_BYTES} bytes"
        )));
    }
    Ok(())
}

pub(crate) fn require_tree_prefix_within_limits(
    git_dir: &Path,
    revision: &str,
    prefix: &str,
    offline: bool,
) -> PrayResult<bool> {
    let output = run_git_command_with_env(
        git_dir,
        &["ls-tree", "-r", "-l", revision, "--", prefix],
        crate::resolve_git_materialize_markers::offline_git_env(offline),
    )?;
    if !output.status.success() {
        return Ok(false);
    }
    let text = String::from_utf8_lossy(&output.stdout);
    if text.trim().is_empty() {
        return Ok(false);
    }
    let mut entry_count = 0usize;
    let mut total_bytes = 0u64;
    for line in text.lines() {
        let Some(size) = parse_ls_tree_blob_size(line) else {
            continue;
        };
        entry_count += 1;
        if entry_count > MAX_ARCHIVE_ENTRIES {
            return Err(PrayError::Integrity(format!(
                "git catalog archive exceeds {MAX_ARCHIVE_ENTRIES} entries"
            )));
        }
        if size > MAX_ARCHIVE_ENTRY_BYTES {
            return Err(PrayError::Integrity(format!(
                "git catalog archive entry exceeds {MAX_ARCHIVE_ENTRY_BYTES} bytes"
            )));
        }
        total_bytes = total_bytes.saturating_add(size);
        if total_bytes > MAX_ARCHIVE_TOTAL_BYTES {
            return Err(PrayError::Integrity(format!(
                "git catalog archive exceeds {MAX_ARCHIVE_TOTAL_BYTES} bytes"
            )));
        }
    }
    Ok(entry_count > 0)
}

fn parse_ls_tree_blob_size(line: &str) -> Option<u64> {
    // git ls-tree -l: mode SP type SP object SP size TAB path
    let mut parts = line.split_whitespace();
    let _mode = parts.next()?;
    let kind = parts.next()?;
    if kind != "blob" {
        return None;
    }
    let _object = parts.next()?;
    let size_text = parts.next()?;
    size_text.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::parse_ls_tree_blob_size;

    #[test]
    fn parses_ls_tree_blob_sizes() {
        assert_eq!(
            parse_ls_tree_blob_size(
                "100644 blob 45b983be36b73c0788dc9cbcb76cbb80fc7bb057       3\tv1/packages/a.txt"
            ),
            Some(3)
        );
        assert_eq!(parse_ls_tree_blob_size("040000 tree abc\tv1"), None);
    }
}
