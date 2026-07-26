use pray_core::{PrayError, PrayResult};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) fn materialize_package_directory(
    package: &pray_core::resolve::ResolvedPackage,
    output_directory: &Path,
) -> PrayResult<()> {
    if output_directory.exists() {
        remove_path_if_exists(output_directory)?;
    }
    fs::create_dir_all(output_directory)?;
    let metadata = serde_json::json!({
        "name": package.declaration.name,
        "version": package.spec.version,
        "tree_hash": package.tree_hash,
        "artifact_hash": package.artifact_hash,
        "exports": package.spec.exports.keys().cloned().collect::<Vec<_>>(),
        "files": package.spec.files,
        "dependencies": package
            .spec
            .dependencies
            .iter()
            .map(|dependency| serde_json::json!({
                "name": dependency.name,
                "constraint": dependency.constraint,
                "optional": dependency.optional,
            }))
            .collect::<Vec<_>>(),
    });
    fs::write(
        output_directory.join("metadata.json"),
        serde_json::to_string_pretty(&metadata)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    )?;
    copy_prayspec_file(&package.root, output_directory)?;

    for file in &package.spec.files {
        copy_package_file(&package.root, output_directory, file)?;
    }
    Ok(())
}

pub(crate) fn write_package_archive(
    package: &pray_core::resolve::ResolvedPackage,
    output_path: &Path,
) -> PrayResult<()> {
    if output_path.exists() {
        remove_path_if_exists(output_path)?;
    }
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let archive_bytes = build_package_archive_bytes(package)?;
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&archive_bytes)?;
    output_file.flush()?;
    Ok(())
}

pub(crate) fn build_package_archive_bytes(
    package: &pray_core::resolve::ResolvedPackage,
) -> PrayResult<Vec<u8>> {
    let metadata = package_metadata(package)?;
    let mut tar_bytes = Vec::new();
    {
        let mut archive = tar::Builder::new(&mut tar_bytes);
        append_archive_file(
            &mut archive,
            Path::new("metadata.json"),
            metadata.as_bytes(),
        )?;
        let prayspec_path = find_prayspec_file(&package.root)?;
        let prayspec_name = prayspec_path
            .file_name()
            .ok_or_else(|| PrayError::Integrity("missing prayspec filename".to_string()))?;
        let prayspec_bytes = fs::read(&prayspec_path)?;
        append_archive_file(&mut archive, Path::new(prayspec_name), &prayspec_bytes)?;
        for file in &package.spec.files {
            let content = read_package_file_bytes(&package.root, file)?;
            append_archive_file(&mut archive, Path::new(file), &content)?;
        }
        archive.finish()?;
    }

    let mut output = Vec::new();
    zstd::stream::copy_encode(&tar_bytes[..], &mut output, 0)?;
    Ok(output)
}

pub(crate) fn package_metadata(package: &pray_core::resolve::ResolvedPackage) -> PrayResult<String> {
    serde_json::to_string_pretty(&serde_json::json!({
        "name": package.declaration.name,
        "version": package.spec.version,
        "tree_hash": package.tree_hash,
        "artifact_hash": package.artifact_hash,
        "exports": package.spec.exports.keys().cloned().collect::<Vec<_>>(),
        "files": package.spec.files,
        "dependencies": package
            .spec
            .dependencies
            .iter()
            .map(|dependency| serde_json::json!({
                "name": dependency.name,
                "constraint": dependency.constraint,
                "optional": dependency.optional,
            }))
            .collect::<Vec<_>>(),
    }))
    .map_err(|error| PrayError::Manifest(error.to_string()))
}

pub(crate) fn append_archive_file(
    archive: &mut tar::Builder<&mut Vec<u8>>,
    path: &Path,
    contents: &[u8],
) -> PrayResult<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(contents.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(0);
    header.set_uid(0);
    header.set_gid(0);
    header.set_cksum();
    archive.append_data(&mut header, path, contents)?;
    Ok(())
}

pub(crate) fn read_package_file_bytes(source_root: &Path, relative_path: &str) -> PrayResult<Vec<u8>> {
    let relative = Path::new(relative_path);
    validate_package_relative_path(relative)?;
    let source = source_root.join(relative);
    if !source.exists() {
        return Err(PrayError::Integrity(format!(
            "package file missing: {}",
            relative_path
        )));
    }
    if source.is_dir() {
        return Err(PrayError::Integrity(format!(
            "package file is a directory: {}",
            relative_path
        )));
    }
    Ok(fs::read(source)?)
}

pub(crate) fn find_prayspec_file(root: &Path) -> PrayResult<PathBuf> {
    let mut prayspec_files = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("prayspec") {
            prayspec_files.push(path);
        }
    }
    match prayspec_files.len() {
        1 => Ok(prayspec_files.remove(0)),
        0 => Err(PrayError::Integrity(format!(
            "no prayspec file found in {:?}",
            root
        ))),
        _ => Err(PrayError::Integrity(format!(
            "multiple prayspec files found in {:?}",
            root
        ))),
    }
}

pub(crate) fn copy_prayspec_file(source_root: &Path, archive_root: &Path) -> PrayResult<()> {
    let prayspec_path = find_prayspec_file(source_root)?;
    let prayspec_name = prayspec_path
        .file_name()
        .ok_or_else(|| PrayError::Integrity("missing prayspec filename".to_string()))?;
    let destination = archive_root.join(prayspec_name);
    fs::copy(prayspec_path, destination)?;
    Ok(())
}

pub(crate) fn remove_path_if_exists(path: &Path) -> PrayResult<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => {
            fs::remove_dir_all(path)?;
            Ok(())
        }
        Ok(_) => {
            fs::remove_file(path)?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn copy_package_file(
    source_root: &Path,
    archive_root: &Path,
    relative_path: &str,
) -> PrayResult<()> {
    let relative = Path::new(relative_path);
    validate_package_relative_path(relative)?;
    let source = source_root.join(relative);
    if !source.exists() {
        return Err(PrayError::Integrity(format!(
            "package file missing: {}",
            relative_path
        )));
    }
    if source.is_dir() {
        return Err(PrayError::Integrity(format!(
            "package file is a directory: {}",
            relative_path
        )));
    }
    let destination = archive_root.join(relative);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, destination)?;
    Ok(())
}

fn validate_package_relative_path(path: &Path) -> PrayResult<()> {
    if path.is_absolute() {
        return Err(PrayError::Integrity(format!(
            "package file path must be relative: {}",
            path.display()
        )));
    }
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(PrayError::Integrity(format!(
                "package file path may not traverse upward: {}",
                path.display()
            )));
        }
    }
    Ok(())
}
