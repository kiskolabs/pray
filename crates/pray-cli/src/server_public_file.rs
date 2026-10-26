use pray_core::resource_limits::MAX_HTTP_RESPONSE_BYTES;
use pray_core::{PrayError, PrayResult};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

pub(crate) fn read(root: &Path, relative: &Path) -> PrayResult<Option<(PathBuf, Vec<u8>)>> {
    if !is_public_distribution_path(relative) {
        return Ok(None);
    }
    let path = root.join(relative);
    let mut current = root.to_path_buf();
    for component in relative.iter() {
        current.push(component);
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        if metadata.file_type().is_symlink() {
            return Ok(None);
        }
    }

    let mut file = match OpenOptions::new().read(true).open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Ok(None);
    }
    if metadata.len() > MAX_HTTP_RESPONSE_BYTES {
        return Err(PrayError::Resolution(format!(
            "static response exceeds {MAX_HTTP_RESPONSE_BYTES} bytes"
        )));
    }
    let mut body = Vec::with_capacity(metadata.len() as usize);
    let copied = (&mut file)
        .take(MAX_HTTP_RESPONSE_BYTES.saturating_add(1))
        .read_to_end(&mut body)?;
    if copied as u64 > MAX_HTTP_RESPONSE_BYTES {
        return Err(PrayError::Resolution(format!(
            "static response exceeds {MAX_HTTP_RESPONSE_BYTES} bytes"
        )));
    }
    Ok(Some((path, body)))
}

fn is_public_distribution_path(path: &Path) -> bool {
    let parts = path
        .iter()
        .map(|part| part.to_string_lossy())
        .collect::<Vec<_>>();
    matches!(parts.as_slice(), [v1, file] if v1 == "v1" && matches!(file.as_ref(), "index.json" | "distribution.json" | "trust.json"))
        || matches!(parts.as_slice(), [v1, area, rest @ ..] if v1 == "v1" && matches!(area.as_ref(), "packages" | "artifacts") && !rest.is_empty())
}
