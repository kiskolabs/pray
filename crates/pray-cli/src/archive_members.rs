use pray_core::paths::normalize_package_relative_path;
use pray_core::{PrayError, PrayResult};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(crate) fn append_archive_file(
    archive: &mut tar::Builder<&mut Vec<u8>>,
    path: &Path,
    contents: &[u8],
) -> PrayResult<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(contents.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(0);
    header.set_uid(0);
    header.set_gid(0);
    header.set_cksum();
    archive.append_data(&mut header, path, contents)?;
    Ok(())
}

pub(crate) fn append_unique_archive_file(
    archive: &mut tar::Builder<&mut Vec<u8>>,
    written_paths: &mut BTreeSet<PathBuf>,
    path: &Path,
    contents: &[u8],
    auto_included_prayspec: &Path,
) -> PrayResult<()> {
    let Some(normalized) = record_archive_path(written_paths, path, auto_included_prayspec)? else {
        return Ok(());
    };
    append_archive_file(archive, &normalized, contents)
}

fn record_archive_path(
    written_paths: &mut BTreeSet<PathBuf>,
    path: &Path,
    auto_included_prayspec: &Path,
) -> PrayResult<Option<PathBuf>> {
    let normalized = normalize_package_relative_path(path)?;
    let auto_included = normalize_package_relative_path(auto_included_prayspec)?;
    if written_paths.insert(normalized.clone()) {
        return Ok(Some(normalized));
    }
    // Pack already includes the prayspec. A spec.files entry that names it is skipped once.
    if normalized == auto_included {
        return Ok(None);
    }
    Err(PrayError::Integrity(format!(
        "duplicate package archive path: {}",
        normalized.display()
    )))
}
