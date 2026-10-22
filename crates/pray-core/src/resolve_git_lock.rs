use crate::{PrayError, PrayResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const LOCK_ATTEMPTS: u32 = 2_400;
const LOCK_RETRY_MILLIS: u64 = 25;
const STALE_LOCK_SECS: u64 = 600;

pub(crate) struct GitCacheLock {
    lock_directory: PathBuf,
}

impl Drop for GitCacheLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.lock_directory);
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
    for _ in 0..LOCK_ATTEMPTS {
        match fs::create_dir(&lock_directory) {
            Ok(()) => {
                write_lock_owner(&lock_directory)?;
                return Ok(GitCacheLock { lock_directory });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if try_clear_stale_lock(&lock_directory) {
                    continue;
                }
                thread::sleep(Duration::from_millis(LOCK_RETRY_MILLIS));
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
        "timed out waiting for git cache lock {}. \
         If no other pray process is using that cache, remove the lock directory and retry.",
        lock_directory.display()
    )))
}

fn write_lock_owner(lock_directory: &Path) -> PrayResult<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0);
    let body = format!("{}\n{now}\n", std::process::id());
    fs::write(lock_directory.join("owner"), body).map_err(|error| {
        PrayError::Resolution(format!(
            "failed to write git cache lock owner {}: {error}",
            lock_directory.display()
        ))
    })
}

fn try_clear_stale_lock(lock_directory: &Path) -> bool {
    let age_secs = lock_age_secs(lock_directory);
    if age_secs < STALE_LOCK_SECS {
        return false;
    }
    let _ = fs::remove_dir_all(lock_directory);
    !lock_directory.exists()
}

fn lock_age_secs(lock_directory: &Path) -> u64 {
    if let Ok(text) = fs::read_to_string(lock_directory.join("owner")) {
        if let Some(stamp_text) = text.lines().nth(1) {
            if let Ok(stamp) = stamp_text.parse::<u64>() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|value| value.as_secs())
                    .unwrap_or(0);
                return now.saturating_sub(stamp);
            }
        }
    }
    let Ok(metadata) = fs::metadata(lock_directory) else {
        return 0;
    };
    let Ok(modified) = metadata.modified() else {
        return 0;
    };
    SystemTime::now()
        .duration_since(modified)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        acquire_git_cache_lock, lock_path_for_git_db, try_clear_stale_lock, STALE_LOCK_SECS,
    };
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn clears_owner_lock_when_stamp_is_stale() {
        let root = std::env::temp_dir().join(format!(
            "pray-lock-stale-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let db = root.join("cache");
        fs::create_dir_all(&db).expect("db");
        let lock = lock_path_for_git_db(&db);
        fs::create_dir_all(&lock).expect("lock");
        let old = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_secs()
            .saturating_sub(STALE_LOCK_SECS + 5);
        fs::write(lock.join("owner"), format!("1\n{old}\n")).expect("owner");
        assert!(try_clear_stale_lock(&lock));
        assert!(!lock.exists());
        let _guard = acquire_git_cache_lock(&db).expect("acquire");
        assert!(lock.exists());
        drop(_guard);
        let _ = fs::remove_dir_all(root);
    }
}
