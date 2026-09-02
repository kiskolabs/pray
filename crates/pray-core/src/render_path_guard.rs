use crate::{PrayError, PrayResult};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub(crate) fn ensure_safe_destination_ancestors(
    project_root: &Path,
    relative: &Path,
    display: &str,
) -> PrayResult<()> {
    let Some(parent) = relative.parent() else {
        return Ok(());
    };
    let mut current = project_root.to_path_buf();
    for component in parent.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(PrayError::Render(format!(
                    "refusing to write `{display}` because a destination parent is a symbolic link"
                )));
            }
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => {
                return Err(PrayError::Render(format!(
                    "refusing to write `{display}`; a destination parent is not a directory"
                )));
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}
