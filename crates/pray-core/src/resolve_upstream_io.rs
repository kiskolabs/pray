use crate::package_upstream::content_paths;
use crate::paths::validate_package_relative_path;
use crate::resource_limits::{
    MAX_ARCHIVE_ENTRIES, MAX_ARCHIVE_ENTRY_BYTES, MAX_ARCHIVE_TOTAL_BYTES,
};
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(super) fn content_file_bytes(
    root: &Path,
    spec: &crate::package_spec::PackageSpec,
) -> PrayResult<BTreeMap<String, Vec<u8>>> {
    let content_paths = content_paths(&spec.files);
    if content_paths.len() > MAX_ARCHIVE_ENTRIES {
        return Err(PrayError::Integrity(format!(
            "package content exceeds {MAX_ARCHIVE_ENTRIES} files"
        )));
    }
    let mut files = BTreeMap::new();
    let mut total_bytes = 0u64;
    for relative in content_paths {
        validate_package_relative_path(Path::new(&relative))?;
        let path = root.join(&relative);
        let size = fs::metadata(&path)
            .map_err(|_| PrayError::Integrity(format!("package file missing: {relative}")))?
            .len();
        if size > MAX_ARCHIVE_ENTRY_BYTES {
            return Err(PrayError::Integrity(format!(
                "package file exceeds {MAX_ARCHIVE_ENTRY_BYTES} bytes: {relative}"
            )));
        }
        total_bytes = total_bytes.saturating_add(size);
        if total_bytes > MAX_ARCHIVE_TOTAL_BYTES {
            return Err(PrayError::Integrity(format!(
                "package content exceeds {MAX_ARCHIVE_TOTAL_BYTES} bytes"
            )));
        }
        files.insert(relative, fs::read(path)?);
    }
    Ok(files)
}

pub(super) fn write_content_files(
    root: &Path,
    old_content: &BTreeMap<String, Vec<u8>>,
    merged: &BTreeMap<String, Vec<u8>>,
) -> PrayResult<()> {
    for path in old_content.keys().chain(merged.keys()) {
        validate_package_relative_path(Path::new(path))?;
    }
    for path in old_content.keys() {
        if !merged.contains_key(path) {
            crate::transaction::remove_file(&root.join(path))?;
        }
    }
    for (relative, bytes) in merged {
        crate::transaction::write_file(&root.join(relative), bytes)?;
    }
    Ok(())
}
