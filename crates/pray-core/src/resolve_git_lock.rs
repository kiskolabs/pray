use crate::{PrayError, PrayResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

pub(crate) struct GitCacheLock {
    lock_directory: PathBuf,
}

impl Drop for GitCacheLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.lock_directory);
    }
}

pub(crate) fn lock_path_for_git_db(db: &Path) -> PathBuf {
    let mut lock = db.as_os_str().to_os_string();
    lock.push(".lock.d");
    PathBuf::from(lock)
}

pub(crate) fn acquire_git_cache_lock(db: &Path) -> PrayResult<GitCacheLock> {
    let lock_directory = lock_path_for_git_db(db);
    if let Some(parent) = lock_directory.parent() {
        fs::create_dir_all(parent)?;
    }
    for _ in 0..400 {
        match fs::create_dir(&lock_directory) {
            Ok(()) => {
                return Ok(GitCacheLock { lock_directory });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                thread::sleep(Duration::from_millis(25));
            }
            Err(error) => {
                return Err(PrayError::Resolution(format!(
                    "failed to lock git cache {}: {error}",
                    lock_directory.display()
                )));
            }
        }
    }
    Err(PrayError::Resolution(format!(
        "timed out waiting for git cache lock {}",
        lock_directory.display()
    )))
}
