use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
struct Owner {
    version: u32,
    pid: u32,
    host: String,
    token: String,
}

pub(super) struct Guard {
    path: PathBuf,
    token: String,
}

impl Drop for Guard {
    fn drop(&mut self) {
        if read_owner(&self.path).is_ok_and(|owner| owner.token == self.token) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(super) fn acquire(directory: &Path) -> PrayResult<Guard> {
    acquire_path(directory, &directory.join("owner"), 0)
}

fn acquire_path(directory: &Path, path: &Path, depth: usize) -> PrayResult<Guard> {
    if depth > 16 {
        return Err(failure(
            "too many interrupted lock recoveries; inspect .pray/write-state",
        ));
    }
    let token = token()?;
    let owner = Owner {
        version: 1,
        pid: std::process::id(),
        host: host(),
        token: token.clone(),
    };
    let temporary = directory.join(format!("{token}.owner"));
    let mut file = private_file(&temporary)?;
    file.write_all(&serde_json::to_vec(&owner).map_err(|error| failure(&error.to_string()))?)?;
    file.sync_all()?;
    let linked = fs::hard_link(&temporary, path);
    fs::remove_file(&temporary)?;
    match linked {
        Ok(()) => {
            sync_directory(directory)?;
            Ok(Guard {
                path: path.into(),
                token,
            })
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let previous = read_owner(path)?;
            if previous.host != host() || alive(previous.pid) {
                return Err(failure(
                    "another pray command owns this project; retry after it finishes",
                ));
            }
            // A claim tied to the dead owner's unique token serializes stale-lock removal.
            let claim = directory.join(format!("reclaim-{}", previous.token));
            let _claim = acquire_path(directory, &claim, depth + 1)?;
            let current = read_owner(path)?;
            if current.token != previous.token {
                return Err(failure("project ownership changed; retry the command"));
            }
            fs::remove_file(path)?;
            acquire_path(directory, path, depth + 1)
        }
        Err(error) => Err(error.into()),
    }
}

fn read_owner(path: &Path) -> PrayResult<Owner> {
    let mut file = crate::render_file::open_regular(path, "project write owner", false)?;
    if file.metadata()?.len() > 4096 {
        return Err(failure("invalid project write owner"));
    }
    let mut bytes = Vec::new();
    (&mut file).take(4097).read_to_end(&mut bytes)?;
    let owner: Owner =
        serde_json::from_slice(&bytes).map_err(|_| failure("invalid project write owner"))?;
    if owner.version != 1
        || owner.pid == 0
        || owner.token.len() != 32
        || !owner.token.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(failure("invalid project write owner"));
    }
    Ok(owner)
}

pub(super) fn token() -> PrayResult<String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|error| failure(&error.to_string()))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

pub(super) fn private_file(path: &Path) -> PrayResult<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    Ok(options.open(path)?)
}

pub(super) fn sync_directory(path: &Path) -> PrayResult<()> {
    #[cfg(unix)]
    {
        File::open(path)?.sync_all()?;
    }
    Ok(())
}

#[cfg(unix)]
fn alive(pid: u32) -> bool {
    pid > i32::MAX as u32
        || unsafe { libc::kill(pid as i32, 0) } == 0
        || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}
#[cfg(not(unix))]
fn alive(_pid: u32) -> bool {
    true
}

fn host() -> String {
    #[cfg(unix)]
    {
        let mut name = [0u8; 256];
        if unsafe { libc::gethostname(name.as_mut_ptr().cast(), name.len()) } == 0 {
            return String::from_utf8_lossy(
                &name[..name
                    .iter()
                    .position(|byte| *byte == 0)
                    .unwrap_or(name.len())],
            )
            .into();
        }
    }
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".into())
}

pub(super) fn failure(message: &str) -> PrayError {
    PrayError::Render(message.into())
}
