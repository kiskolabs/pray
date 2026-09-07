#[path = "recovery.rs"]
mod recovery;

use super::owner::{failure, private_file, sync_directory, token};
use crate::{hashing::sha256_prefixed, render_file::read_regular_bytes, PrayResult};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

const MAX_LOG: u64 = 96 * 1024 * 1024;
const MAX_SAVED: usize = 64 * 1024 * 1024;
const MAX_ENTRIES: usize = 10_000;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: String,
    before: Option<String>,
    after_hash: Option<String>,
    mode: u32,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Record {
    Start { version: u32 },
    Write { entry: Entry },
    Undone { index: usize },
    Commit,
}

pub(super) struct Journal {
    pub root: PathBuf,
    directory: PathBuf,
    path: PathBuf,
    saved: usize,
    entries: usize,
}

impl Journal {
    pub fn open(root: PathBuf, directory: PathBuf) -> PrayResult<Self> {
        Ok(Self {
            root,
            path: directory.join("journal"),
            directory,
            saved: 0,
            entries: 0,
        })
    }

    fn append(&self, record: &Record) -> PrayResult<()> {
        let mut bytes = serde_json::to_vec(record).map_err(|error| failure(&error.to_string()))?;
        bytes.push(b'\n');
        let mut options = OpenOptions::new();
        options.append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        let mut file = options.open(&self.path)?;
        if !file.metadata()?.is_file() || file.metadata()?.len() + bytes.len() as u64 > MAX_LOG {
            return Err(failure("project recovery journal exceeds its 96 MiB limit"));
        }
        file.write_all(&bytes)?;
        file.sync_all()?;
        Ok(())
    }

    pub fn replace(
        &mut self,
        path: &Path,
        before: Option<&[u8]>,
        after: Option<&[u8]>,
    ) -> PrayResult<()> {
        if before == after {
            return Ok(());
        }
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        let relative = absolute
            .strip_prefix(&self.root)
            .map_err(|_| failure("destination is outside this project"))?;
        let display = relative.to_string_lossy().replace('\\', "/");
        self.destination(&display)?;
        if before
            .into_iter()
            .chain(after)
            .any(|bytes| bytes.len() > 32 * 1024 * 1024)
        {
            return Err(failure("destination exceeds the 32 MiB limit"));
        }
        if self.saved + before.map_or(0, <[u8]>::len) + after.map_or(0, <[u8]>::len) > MAX_SAVED
            || self.entries == MAX_ENTRIES
        {
            return Err(failure(
                "project write exceeds the 64 MiB or 10000-file recovery limit",
            ));
        }
        if snapshot(&absolute)?.as_deref() != before {
            return Err(changed(&display));
        }
        let mode = permissions(&absolute)?;
        if !self.path.try_exists()? {
            private_file(&self.path)?.sync_all()?;
            self.append(&Record::Start { version: 1 })?;
            sync_directory(&self.directory)?;
        }
        let entry = Entry {
            path: display.clone(),
            before: before.map(|bytes| STANDARD.encode(bytes)),
            after_hash: after.map(sha256_prefixed),
            mode,
        };
        self.append(&Record::Write { entry })?;
        self.saved += before.map_or(0, <[u8]>::len) + after.map_or(0, <[u8]>::len);
        self.entries += 1;
        self.install(&display, before, after, mode)
    }

    fn destination(&self, display: &str) -> PrayResult<PathBuf> {
        let relative = crate::paths::validate_destination_path(display)?;
        if relative.as_path().starts_with(".pray/write-state") {
            return Err(failure("destination overlaps project recovery state"));
        }
        crate::render_path_guard::ensure_safe_destination_ancestors(
            &self.root,
            relative.as_path(),
            display,
        )?;
        Ok(relative.join_root(&self.root))
    }

    fn install(
        &self,
        display: &str,
        expected: Option<&[u8]>,
        bytes: Option<&[u8]>,
        mode: u32,
    ) -> PrayResult<()> {
        let path = self.destination(display)?;
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent)?;
        self.destination(display)?;
        let temporary = self.directory.join(format!("{}.stage", token()?));
        if let Some(bytes) = bytes {
            let mut file = private_file(&temporary)?;
            file.write_all(bytes)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                file.set_permissions(fs::Permissions::from_mode(mode & 0o777))?;
            }
            file.sync_all()?;
        }
        if snapshot(&path)?.as_deref() != expected {
            return Err(changed(display));
        }
        self.destination(display)?;
        match (expected, bytes) {
            (None, Some(_)) => {
                fs::hard_link(&temporary, &path)?;
                fs::remove_file(&temporary)?;
            }
            (Some(_), Some(_)) => fs::rename(&temporary, &path)?,
            (Some(_), None) => fs::remove_file(&path)?,
            (None, None) => {}
        }
        self.sync_parents(&path)
    }

    fn sync_parents(&self, path: &Path) -> PrayResult<()> {
        let mut current = path.parent().unwrap();
        loop {
            match sync_directory(current) {
                Err(crate::PrayError::Io(error))
                    if error.kind() == std::io::ErrorKind::NotFound => {}
                result => result?,
            }
            if current == self.root {
                break;
            }
            current = current
                .parent()
                .ok_or_else(|| failure("destination is outside this project"))?;
        }
        Ok(())
    }

    pub fn commit(&self) -> PrayResult<()> {
        if self.path.try_exists()? {
            self.append(&Record::Commit)?;
            fs::remove_file(&self.path)?;
            sync_directory(&self.directory)?;
        }
        self.clean_stages()
    }

    fn clean_stages(&self) -> PrayResult<()> {
        for entry in fs::read_dir(&self.directory)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.len() == 38
                && name.ends_with(".stage")
                && name[..32].bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }
}

fn snapshot(path: &Path) -> PrayResult<Option<Vec<u8>>> {
    match crate::render_file::destination_kind(path)? {
        crate::render_file::DestinationKind::Missing => Ok(None),
        _ => Ok(Some(read_regular_bytes(path, &path.display().to_string())?)),
    }
}
fn permissions(path: &Path) -> PrayResult<u32> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::symlink_metadata(path) {
            return Ok(metadata.permissions().mode() & 0o777);
        }
    }
    Ok(0o644)
}
fn changed(display: &str) -> crate::PrayError {
    failure(&format!("`{display}` changed during the interrupted write. Inspect your changes and move the file aside, then retry; recovery data remains in .pray/write-state"))
}
