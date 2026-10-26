use crate::{PrayError, PrayResult};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Read};
use std::path::{Component, Path};

pub(crate) enum DeclaredRead {
    Missing,
    Bytes(Vec<u8>),
}

pub(crate) fn declared_directory(
    root: &Path,
    relative: &str,
    label: &str,
) -> PrayResult<std::path::PathBuf> {
    let path = root.join(relative);
    let mut current = root.to_path_buf();
    for component in Path::new(relative).components() {
        let part = match component {
            Component::Normal(part) => part,
            Component::CurDir => continue,
            Component::ParentDir => {
                current.push("..");
                continue;
            }
            _ => {
                return Err(PrayError::Integrity(format!(
                    "{label} escapes its root: {relative}"
                )));
            }
        };
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(PrayError::Integrity(format!(
                    "{label} is a symbolic link: {relative}"
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(path),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(path)
}

pub(crate) fn read_declared_file(
    root: &Path,
    relative: &str,
    label: &str,
) -> PrayResult<DeclaredRead> {
    let mut current = root.to_path_buf();
    let mut parts = Vec::new();
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(part) => parts.push(part.to_os_string()),
            Component::CurDir => {}
            _ => {
                return Err(PrayError::Integrity(format!(
                    "{label} escapes its root: {relative}"
                )));
            }
        }
    }
    if parts.is_empty() {
        return Ok(DeclaredRead::Missing);
    }
    for (index, part) in parts.iter().enumerate() {
        current.push(part);
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(DeclaredRead::Missing),
            Err(error) => return Err(error.into()),
        };
        if metadata.file_type().is_symlink() {
            return Err(PrayError::Integrity(format!(
                "{label} is a symbolic link: {relative}"
            )));
        }
        let last = index + 1 == parts.len();
        if metadata.is_dir() {
            if last {
                return Err(PrayError::Integrity(format!(
                    "{label} is a directory: {relative}"
                )));
            }
            continue;
        }
        if !last || !metadata.is_file() {
            if last {
                return Err(PrayError::Integrity(format!(
                    "{label} is not a regular file: {relative}"
                )));
            }
            return Ok(DeclaredRead::Missing);
        }
    }
    read_without_following(&current, label, relative)
}

fn read_without_following(path: &Path, label: &str, relative: &str) -> PrayResult<DeclaredRead> {
    let mut options = OpenOptions::new();
    options.read(true);
    add_no_follow(&mut options);
    let mut file = match options.open(path) {
        Ok(file) => file,
        Err(error) if is_symlink_open_error(&error) => {
            return Err(PrayError::Integrity(format!(
                "{label} is a symbolic link: {relative}"
            )));
        }
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(DeclaredRead::Missing),
        Err(error) => return Err(error.into()),
    };
    let metadata = file.metadata()?;
    if metadata.is_dir() {
        return Err(PrayError::Integrity(format!(
            "{label} is a directory: {relative}"
        )));
    }
    if !metadata.is_file() {
        return Err(PrayError::Integrity(format!(
            "{label} is not a regular file: {relative}"
        )));
    }
    Ok(DeclaredRead::Bytes(read_rest(&mut file)?))
}

fn read_rest(file: &mut std::fs::File) -> PrayResult<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        match file.read(&mut buffer) {
            Ok(0) => return Ok(bytes),
            Ok(read) => bytes.extend_from_slice(&buffer[..read]),
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) => return Err(error.into()),
        }
    }
}

#[cfg(unix)]
fn add_no_follow(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    options.custom_flags(libc::O_NOFOLLOW);
}

#[cfg(not(unix))]
fn add_no_follow(_options: &mut OpenOptions) {}

fn is_symlink_open_error(error: &std::io::Error) -> bool {
    #[cfg(unix)]
    {
        error.raw_os_error() == Some(libc::ELOOP)
    }
    #[cfg(not(unix))]
    {
        let _ = error;
        false
    }
}
