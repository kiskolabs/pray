use crate::paths::normalize_package_relative_path;
use crate::resource_limits::{
    MAX_ARCHIVE_ENTRIES, MAX_ARCHIVE_ENTRY_BYTES, MAX_ARCHIVE_TOTAL_BYTES,
};
use crate::{PrayError, PrayResult};
use std::collections::BTreeSet;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

pub(crate) fn unpack_bounded_catalog_tar(archive_bytes: &[u8], dest: &Path) -> PrayResult<()> {
    if archive_bytes.len() as u64 > MAX_ARCHIVE_TOTAL_BYTES {
        return Err(PrayError::Integrity(format!(
            "git catalog archive exceeds {MAX_ARCHIVE_TOTAL_BYTES} bytes"
        )));
    }
    let mut archive = tar::Archive::new(Cursor::new(archive_bytes));
    let mut written_paths = BTreeSet::new();
    let mut total_bytes = 0u64;
    let mut entry_count = 0usize;
    for entry in archive
        .entries()
        .map_err(|error| PrayError::Integrity(error.to_string()))?
    {
        let mut entry = entry.map_err(|error| PrayError::Integrity(error.to_string()))?;
        let entry_type = entry.header().entry_type();
        if entry_type.is_dir() {
            continue;
        }
        // git archive emits a pax global header; ignore without applying path.
        if matches!(entry_type, tar::EntryType::XGlobalHeader) {
            continue;
        }
        if entry_type.is_symlink() || entry_type.is_hard_link() || !entry_type.is_file() {
            return Err(PrayError::Integrity(
                "unsupported git catalog archive entry type".to_string(),
            ));
        }
        entry_count += 1;
        if entry_count > MAX_ARCHIVE_ENTRIES {
            return Err(PrayError::Integrity(format!(
                "git catalog archive exceeds {MAX_ARCHIVE_ENTRIES} entries"
            )));
        }
        let path = entry
            .path()
            .map_err(|error| PrayError::Integrity(error.to_string()))?
            .into_owned();
        let path = normalize_package_relative_path(&path)?;
        if !written_paths.insert(path.clone()) {
            return Err(PrayError::Integrity(format!(
                "duplicate git catalog archive path: {}",
                path.display()
            )));
        }
        let size = entry.header().size().unwrap_or(0);
        if size > MAX_ARCHIVE_ENTRY_BYTES {
            return Err(PrayError::Integrity(format!(
                "git catalog archive entry exceeds {MAX_ARCHIVE_ENTRY_BYTES} bytes: {}",
                path.display()
            )));
        }
        total_bytes = total_bytes.saturating_add(size);
        if total_bytes > MAX_ARCHIVE_TOTAL_BYTES {
            return Err(PrayError::Integrity(format!(
                "git catalog archive exceeds {MAX_ARCHIVE_TOTAL_BYTES} bytes"
            )));
        }
        let destination = dest.join(&path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut destination_file = fs::File::create(&destination)?;
        let copied = std::io::copy(
            &mut (&mut entry).take(MAX_ARCHIVE_ENTRY_BYTES.saturating_add(1)),
            &mut destination_file,
        )
        .map_err(|error| PrayError::Integrity(error.to_string()))?;
        if copied > MAX_ARCHIVE_ENTRY_BYTES {
            return Err(PrayError::Integrity(format!(
                "git catalog archive entry exceeds {MAX_ARCHIVE_ENTRY_BYTES} bytes: {}",
                path.display()
            )));
        }
    }
    Ok(())
}
