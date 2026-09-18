use crate::project_paths::manifest_path;
use pray_core::manifest::{parse_manifest, read_manifest_text};
use pray_core::publish_remote::{
    resolve_path_remote, resolve_publish_destinations, ManifestPublishRemote, PublishCliDest,
    PublishDestination,
};
use pray_core::resolve::project_root_from_manifest;
use pray_core::{PrayError, PrayResult};
use std::path::{Path, PathBuf};

pub(crate) fn load_publish_remotes() -> PrayResult<(PathBuf, Vec<ManifestPublishRemote>)> {
    let path = manifest_path();
    let manifest = parse_manifest(&read_manifest_text(&path)?)?;
    Ok((project_root_from_manifest(&path), manifest.publish_remotes))
}

pub(crate) fn publish_destinations(
    cli: PublishCliDest,
) -> PrayResult<(PathBuf, Vec<PublishDestination>)> {
    let (project_root, remotes) = load_publish_remotes()?;
    let dests = resolve_publish_destinations(&remotes, &cli, &project_root)?;
    Ok((project_root, dests))
}

pub(crate) fn required_path_remote_root(
    to: Option<&str>,
    root: Option<&Path>,
) -> PrayResult<PathBuf> {
    match load_publish_remotes() {
        Ok((project_root, remotes)) => resolve_path_remote(&remotes, to, root, &project_root),
        Err(_) => root
            .map(Path::to_path_buf)
            .ok_or_else(|| PrayError::Usage("requires --root PATH or --to NAME".to_string())),
    }
}

pub(crate) fn optional_path_remote_root(to: Option<&str>, root: &Path) -> PrayResult<PathBuf> {
    match load_publish_remotes() {
        Ok((project_root, remotes)) if !remotes.is_empty() || to.is_some() => {
            let cli_root = if root == Path::new(".") {
                None
            } else {
                Some(root)
            };
            resolve_path_remote(&remotes, to, cli_root, &project_root)
        }
        Ok(_) => Ok(root.to_path_buf()),
        Err(_) if to.is_none() => Ok(root.to_path_buf()),
        Err(error) => Err(error),
    }
}
