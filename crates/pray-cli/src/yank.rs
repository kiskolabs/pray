use crate::publish_dest::required_path_remote_root;
use crate::registry_ops::{
    load_registry_package_metadata, registry_metadata_path, write_registry_package_metadata,
};
use crate::revision::{record_root_revision, RevisionAction};
use pray_core::registry::set_package_version_yanked;
use pray_core::{PrayError, PrayResult};
use std::path::PathBuf;

pub(crate) fn yank_command(
    package: String,
    version: String,
    root: Option<PathBuf>,
    to: Option<String>,
    undo: bool,
) -> PrayResult<()> {
    let root = required_path_remote_root(to.as_deref(), root.as_deref())?;
    let metadata_path = registry_metadata_path(&root, &package);
    if !metadata_path.exists() {
        return Err(PrayError::Resolution(format!(
            "package {package} not found under {}",
            root.display()
        )));
    }
    let mut metadata = load_registry_package_metadata(&metadata_path, &package)?;
    let yanked = !undo;
    set_package_version_yanked(&mut metadata, &version, yanked)?;
    write_registry_package_metadata(&metadata_path, &metadata)?;
    record_root_revision(&root, RevisionAction::Publish)?;
    if yanked {
        println!("yanked {package} {version} in {}", root.display());
    } else {
        println!("unyanked {package} {version} in {}", root.display());
    }
    Ok(())
}
