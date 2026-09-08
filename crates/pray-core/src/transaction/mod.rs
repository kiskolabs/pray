mod journal;
mod owner;

use crate::{PrayError, PrayResult};
use journal::Journal;
use std::{cell::RefCell, fs, path::Path};

thread_local! { static ACTIVE: RefCell<Option<Journal>> = const { RefCell::new(None) }; }

pub fn run<T>(root: &Path, operation: impl FnOnce() -> PrayResult<T>) -> PrayResult<T> {
    let root = if root.is_absolute() {
        root.to_path_buf()
    } else {
        std::env::current_dir()?.join(root)
    };
    if ACTIVE.with(|active| {
        active
            .borrow()
            .as_ref()
            .is_some_and(|journal| journal.root == root)
    }) {
        return operation();
    }
    if ACTIVE.with(|active| active.borrow().is_some()) {
        return Err(owner::failure(
            "a command cannot write two projects in one transaction",
        ));
    }
    let directory = root.join(".pray/write-state");
    crate::render_path_guard::ensure_safe_destination_ancestors(
        &root,
        Path::new(".pray/write-state/owner"),
        "project write state",
    )?;
    fs::create_dir_all(&directory)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        use std::os::unix::fs::PermissionsExt;
        let handle = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY)
            .open(&directory)?;
        handle.set_permissions(fs::Permissions::from_mode(0o700))?;
    }
    crate::render_path_guard::ensure_safe_destination_ancestors(
        &root,
        Path::new(".pray/write-state/owner"),
        "project write state",
    )?;
    for path in [&directory, &root.join(".pray"), &root] {
        owner::sync_directory(path)?;
    }
    let _owner = owner::acquire(&directory)?;
    let mut journal = Journal::open(root, directory)?;
    journal.recover()?;
    ACTIVE.with(|active| *active.borrow_mut() = Some(journal));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation));
    let mut journal = ACTIVE.with(|active| active.borrow_mut().take().unwrap());
    match result {
        Ok(Ok(value)) => {
            journal.commit()?;
            Ok(value)
        }
        Ok(Err(error)) => match journal.recover() {
            Ok(()) => Err(error),
            Err(recovery) => Err(PrayError::Render(format!(
                "{error}\nRecovery stopped: {recovery}"
            ))),
        },
        Err(panic) => {
            let _ = journal.recover();
            std::panic::resume_unwind(panic)
        }
    }
}

pub fn write_file(path: &Path, bytes: impl AsRef<[u8]>) -> PrayResult<()> {
    let bytes = bytes.as_ref();
    if ACTIVE.with(|active| active.borrow().is_some()) {
        let before = match crate::render_file::destination_kind(path)? {
            crate::render_file::DestinationKind::Missing => None,
            _ => Some(crate::render_file::read_regular_bytes(
                path,
                &path.display().to_string(),
            )?),
        };
        replace(path, before.as_deref(), Some(bytes))?;
    } else {
        fs::write(path, bytes)?;
    }
    Ok(())
}

pub(crate) fn remove_file(path: &Path) -> PrayResult<()> {
    if ACTIVE.with(|active| active.borrow().is_some()) {
        let before = crate::render_file::read_regular_bytes(path, &path.display().to_string())?;
        replace(path, Some(&before), None)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub(crate) fn replace(
    path: &Path,
    before: Option<&[u8]>,
    after: Option<&[u8]>,
) -> PrayResult<bool> {
    ACTIVE.with(|active| {
        let mut active = active.borrow_mut();
        let Some(journal) = active.as_mut() else {
            return Ok(false);
        };
        journal.replace(path, before, after)?;
        Ok(true)
    })
}
