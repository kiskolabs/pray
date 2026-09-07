use crate::{PrayError, PrayResult};
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::Path;

pub(crate) fn create_regular_bytes(path: &Path, display: &str, bytes: &[u8]) -> PrayResult<()> {
    if crate::transaction::replace(path, None, Some(bytes))? {
        return Ok(());
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    add_no_follow(&mut options);
    let mut file = options
        .open(path)
        .map_err(|error| map_open_error(error, display))?;
    file.write_all(bytes)?;
    Ok(())
}

pub(crate) fn read_regular_bytes(path: &Path, display: &str) -> PrayResult<Vec<u8>> {
    let mut file = open_regular(path, display, false)?;
    read_destination_bytes(&mut file, display)
}

pub fn read_destination_text(path: &Path) -> PrayResult<String> {
    String::from_utf8(read_regular_bytes(path, &path.display().to_string())?)
        .map_err(|error| PrayError::Render(error.to_string()))
}

pub(crate) const MAX_DESTINATION_BYTES: u64 = 32 * 1024 * 1024;

pub(crate) fn read_destination_bytes(file: &mut File, display: &str) -> PrayResult<Vec<u8>> {
    if file.metadata()?.len() > MAX_DESTINATION_BYTES {
        return Err(destination_size_error(display));
    }
    let mut bytes = Vec::new();
    file.take(MAX_DESTINATION_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_DESTINATION_BYTES {
        return Err(destination_size_error(display));
    }
    Ok(bytes)
}

fn destination_size_error(display: &str) -> PrayError {
    PrayError::Render(format!(
        "refusing to read `{display}`; destination exceeds the 32 MiB limit"
    ))
}

pub(crate) fn open_regular(path: &Path, display: &str, writable: bool) -> PrayResult<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(writable);
    add_no_follow(&mut options);
    let file = options
        .open(path)
        .map_err(|error| map_open_error(error, display))?;
    if !file.metadata()?.is_file() {
        return Err(PrayError::Render(format!(
            "refusing to write `{display}`; destination is not a regular file"
        )));
    }
    Ok(file)
}

#[cfg(unix)]
fn add_no_follow(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    options.custom_flags(libc::O_NOFOLLOW);
}

#[cfg(not(unix))]
fn add_no_follow(_options: &mut OpenOptions) {}

fn map_open_error(error: std::io::Error, display: &str) -> PrayError {
    #[cfg(unix)]
    if error.raw_os_error() == Some(libc::ELOOP) {
        return symlink_error(display);
    }
    error.into()
}

pub(crate) fn symlink_error(display: &str) -> PrayError {
    PrayError::Render(format!(
        "refusing to write `{display}` because it is a symbolic link"
    ))
}

pub(crate) enum DestinationKind {
    Missing,
    Regular,
    Symlink,
    Other,
}

pub(crate) fn destination_kind(path: &Path) -> PrayResult<DestinationKind> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Ok(DestinationKind::Symlink),
        Ok(metadata) if metadata.is_file() => Ok(DestinationKind::Regular),
        Ok(_) => Ok(DestinationKind::Other),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(DestinationKind::Missing),
        Err(error) => Err(error.into()),
    }
}
