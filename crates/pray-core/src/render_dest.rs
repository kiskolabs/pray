use crate::hashing::sha256_prefixed;
use crate::lockfile::{Lockfile, ProvisionedFileRecord};
use crate::paths::validate_destination_path;
use crate::render_file::{
    create_regular_bytes, destination_kind, open_regular, read_regular_bytes, symlink_error,
    DestinationKind,
};
use crate::render_path_guard::ensure_safe_destination_ancestors;
use crate::render_provisioned::{
    expected_provisioned_bytes, planned_provisioned_files, PlannedProvisionedFile,
};
use crate::resolve::ResolvedProject;
use crate::{PrayError, PrayResult};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvisionedDestinationStatus {
    Missing,
    Unchanged,
    ManagedUpdate,
}

pub fn provisioned_lock_records(
    project: &ResolvedProject,
) -> PrayResult<Vec<ProvisionedFileRecord>> {
    let mut records = Vec::new();
    for file in planned_provisioned_files(project)? {
        let expected = expected_provisioned_bytes(&file.source, &project.manifest.symbols)?;
        records.push(ProvisionedFileRecord {
            path: file.path.to_string_lossy().replace('\\', "/"),
            content_hash: sha256_prefixed(&expected),
            package: file.package,
            export: file.export,
        });
    }
    Ok(records)
}

pub fn materialize_provisioned_exports(
    project: &ResolvedProject,
    previous_lockfile: Option<&Lockfile>,
) -> PrayResult<()> {
    let planned = planned_provisioned_files(project)?;
    let previous = previous_lock_map(previous_lockfile);
    let mut planned_paths = BTreeSet::new();
    for file in &planned {
        let relative = validate_destination_path(&file.path.to_string_lossy())?;
        planned_paths.insert(lock_path(relative.as_path()));
        write_provisioned_leaf(project, file, &previous)?;
    }
    if let Some(lockfile) = previous_lockfile {
        prune_dropped_leaves(project, lockfile, &planned_paths)?;
    }
    Ok(())
}

pub fn provisioned_destination_status(
    project: &ResolvedProject,
    file: &PlannedProvisionedFile,
    previous_lockfile: Option<&Lockfile>,
) -> PrayResult<ProvisionedDestinationStatus> {
    let relative = validate_destination_path(&file.path.to_string_lossy())?;
    ensure_safe_destination_ancestors(
        &project.project_root,
        relative.as_path(),
        relative.as_str(),
    )?;
    let destination = relative.join_root(&project.project_root);
    let expected = expected_provisioned_bytes(&file.source, &project.manifest.symbols)?;
    let normalized = lock_path(relative.as_path());
    let previous = previous_lockfile.and_then(|lockfile| {
        lockfile
            .provisioned
            .iter()
            .find(|record| record.path == normalized)
    });
    classify_destination(&destination, &normalized, &expected, previous)
}

fn previous_lock_map(lockfile: Option<&Lockfile>) -> BTreeMap<String, ProvisionedFileRecord> {
    lockfile
        .map(|lockfile| {
            lockfile
                .provisioned
                .iter()
                .map(|record| (record.path.clone(), record.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn write_provisioned_leaf(
    project: &ResolvedProject,
    file: &PlannedProvisionedFile,
    previous: &BTreeMap<String, ProvisionedFileRecord>,
) -> PrayResult<()> {
    let relative = validate_destination_path(&file.path.to_string_lossy())?;
    ensure_safe_destination_ancestors(
        &project.project_root,
        relative.as_path(),
        relative.as_str(),
    )?;
    let destination = relative.join_root(&project.project_root);
    let path_text = lock_path(relative.as_path());
    let expected = expected_provisioned_bytes(&file.source, &project.manifest.symbols)?;
    let record = previous.get(&path_text);
    match classify_destination(&destination, &path_text, &expected, record)? {
        ProvisionedDestinationStatus::Missing => {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            ensure_safe_destination_ancestors(
                &project.project_root,
                relative.as_path(),
                relative.as_str(),
            )?;
            create_regular_bytes(&destination, &path_text, &expected)
        }
        ProvisionedDestinationStatus::Unchanged => Ok(()),
        ProvisionedDestinationStatus::ManagedUpdate => {
            let Some(record) = record else {
                return Err(PrayError::Render(format!(
                    "missing lock ownership for `{path_text}`"
                )));
            };
            ensure_safe_destination_ancestors(
                &project.project_root,
                relative.as_path(),
                relative.as_str(),
            )?;
            update_regular_bytes(&destination, &path_text, &expected, &record.content_hash)
        }
    }
}

fn classify_destination(
    destination: &Path,
    path_text: &str,
    expected: &[u8],
    previous: Option<&ProvisionedFileRecord>,
) -> PrayResult<ProvisionedDestinationStatus> {
    match destination_kind(destination)? {
        DestinationKind::Missing => Ok(ProvisionedDestinationStatus::Missing),
        DestinationKind::Regular => {
            let on_disk = read_regular_bytes(destination, path_text)?;
            if on_disk == expected {
                return Ok(ProvisionedDestinationStatus::Unchanged);
            }
            if let Some(record) = previous {
                if sha256_prefixed(&on_disk) == record.content_hash {
                    return Ok(ProvisionedDestinationStatus::ManagedUpdate);
                }
                return Err(PrayError::Render(format!(
                    "refusing to overwrite `{path_text}`; it was provisioned and then edited"
                )));
            }
            Err(PrayError::Render(format!(
                "refusing to overwrite `{path_text}`; it already exists and is not the expected provisioned file"
            )))
        }
        DestinationKind::Symlink => Err(symlink_error(path_text)),
        DestinationKind::Other => Err(PrayError::Render(format!(
            "refusing to write `{path_text}`; destination is not a regular file"
        ))),
    }
}

fn prune_dropped_leaves(
    project: &ResolvedProject,
    previous: &Lockfile,
    planned_paths: &BTreeSet<String>,
) -> PrayResult<()> {
    for record in &previous.provisioned {
        if planned_paths.contains(&record.path) {
            continue;
        }
        let relative = validate_destination_path(&record.path)?;
        ensure_safe_destination_ancestors(
            &project.project_root,
            relative.as_path(),
            relative.as_str(),
        )?;
        let destination = relative.join_root(&project.project_root);
        match destination_kind(&destination)? {
            DestinationKind::Regular => {
                let on_disk = read_regular_bytes(&destination, &record.path)?;
                if sha256_prefixed(&on_disk) == record.content_hash {
                    ensure_safe_destination_ancestors(
                        &project.project_root,
                        relative.as_path(),
                        relative.as_str(),
                    )?;
                    fs::remove_file(&destination)?;
                }
            }
            DestinationKind::Missing | DestinationKind::Symlink | DestinationKind::Other => {}
        }
    }
    Ok(())
}

fn lock_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn update_regular_bytes(
    path: &Path,
    display: &str,
    bytes: &[u8],
    authorized_hash: &str,
) -> PrayResult<()> {
    let mut file = open_regular(path, display, true)?;
    let mut on_disk = Vec::new();
    file.read_to_end(&mut on_disk)?;
    if on_disk == bytes {
        return Ok(());
    }
    if sha256_prefixed(&on_disk) != authorized_hash {
        return Err(PrayError::Render(format!(
            "refusing to overwrite `{display}`; it was provisioned and then edited"
        )));
    }
    file.rewind()?;
    file.set_len(0)?;
    file.write_all(bytes)?;
    Ok(())
}
