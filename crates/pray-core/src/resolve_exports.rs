use crate::hashing::normalize_line_endings;
use crate::manifest::ManifestPackage;
use crate::package_spec::PackageSpec;
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) fn select_exports(
    declaration: &ManifestPackage,
    spec: &PackageSpec,
) -> PrayResult<Vec<String>> {
    if !declaration.exports.is_empty() {
        for export in &declaration.exports {
            if !spec.exports.contains_key(export) {
                return Err(PrayError::Resolution(format!(
                    "package {} does not export {}",
                    declaration.name, export
                )));
            }
        }
        return Ok(declaration.exports.clone());
    }

    if declaration.roles.is_empty() && declaration.file.is_none() {
        return Ok(spec.exports.keys().cloned().collect());
    }

    let mut roles = declaration.roles.clone();
    if declaration.file.is_some() && !roles.contains(&crate::manifest::ExportRole::File) {
        roles.push(crate::manifest::ExportRole::File);
    }

    let mut selected = Vec::new();
    for role in roles {
        if role == crate::manifest::ExportRole::Fragment {
            for name in select_fragment_role_exports(&declaration.name, spec)? {
                if !selected.contains(&name) {
                    selected.push(name);
                }
            }
            continue;
        }
        let compatible: Vec<String> = spec
            .exports
            .iter()
            .filter(|(_, export)| crate::destination::export_kind_matches_role(&export.kind, role))
            .map(|(name, _)| name.clone())
            .collect();
        match compatible.as_slice() {
            [name] => {
                if !selected.contains(name) {
                    selected.push(name.clone());
                }
            }
            [] => {
                return Err(PrayError::Resolution(format!(
                    "package {} has no export compatible with {:?}",
                    declaration.name, role
                )));
            }
            _ => {
                return Err(PrayError::Resolution(format!(
                    "package {} has multiple exports compatible with {:?}; set export: \"name\"",
                    declaration.name, role
                )));
            }
        }
    }
    Ok(selected)
}

fn select_fragment_role_exports(package_name: &str, spec: &PackageSpec) -> PrayResult<Vec<String>> {
    let fragments: Vec<String> = spec
        .exports
        .iter()
        .filter(|(_, export)| export.kind == "fragment")
        .map(|(name, _)| name.clone())
        .collect();
    match fragments.as_slice() {
        [name] => Ok(vec![name.clone()]),
        [] => {
            let files: Vec<String> = spec
                .exports
                .iter()
                .filter(|(_, export)| export.kind == "file")
                .map(|(name, _)| name.clone())
                .collect();
            match files.as_slice() {
                [name] => Ok(vec![name.clone()]),
                [] => Err(PrayError::Resolution(format!(
                    "package {package_name} has no export compatible with {:?}",
                    crate::manifest::ExportRole::Fragment
                ))),
                _ => Err(PrayError::Resolution(format!(
                    "package {package_name} has multiple exports compatible with {:?}; set export: \"name\"",
                    crate::manifest::ExportRole::Fragment
                ))),
            }
        }
        _ => Err(PrayError::Resolution(format!(
            "package {package_name} has multiple exports compatible with {:?}; set export: \"name\"",
            crate::manifest::ExportRole::Fragment
        ))),
    }
}

pub(crate) fn read_text(path: &Path) -> PrayResult<String> {
    let text = fs::read_to_string(path)?;
    Ok(normalize_line_endings(&text))
}

pub(crate) fn load_package_file_bytes(
    root: &Path,
    spec: &PackageSpec,
) -> PrayResult<BTreeMap<String, Vec<u8>>> {
    let mut file_bytes = BTreeMap::new();
    for file in &spec.files {
        let path = root.join(file);
        if !path.exists() {
            return Err(PrayError::Integrity(format!(
                "package file missing: {}",
                file
            )));
        }
        if path.is_dir() {
            return Err(PrayError::Integrity(format!(
                "package file is a directory: {}",
                file
            )));
        }
        file_bytes.insert(file.clone(), fs::read(&path)?);
    }
    Ok(file_bytes)
}

pub(crate) fn load_export_bodies(
    file_bytes: &BTreeMap<String, Vec<u8>>,
    spec: &PackageSpec,
    selected_exports: &[String],
) -> PrayResult<BTreeMap<String, String>> {
    let mut export_bodies = BTreeMap::new();
    for export_name in selected_exports {
        let entry = spec.exports.get(export_name).ok_or_else(|| {
            PrayError::Resolution(format!(
                "package {} is missing export {}",
                spec.name, export_name
            ))
        })?;
        if !matches!(entry.kind.as_str(), "fragment" | "file") {
            continue;
        }
        let bytes = file_bytes.get(&entry.path).ok_or_else(|| {
            PrayError::Integrity(format!(
                "package file missing for export {}: {}",
                export_name, entry.path
            ))
        })?;
        let text = match std::str::from_utf8(bytes) {
            Ok(text) => text,
            Err(error) if entry.kind == "fragment" => {
                return Err(PrayError::Integrity(format!(
                    "package file is not valid utf-8 for export {export_name}: {error}"
                )));
            }
            Err(_) => continue,
        };
        export_bodies.insert(export_name.clone(), normalize_line_endings(text));
    }
    Ok(export_bodies)
}

pub(crate) fn build_skill_file_index(spec: &PackageSpec) -> BTreeMap<String, Vec<String>> {
    let mut index = BTreeMap::new();
    for (export_name, export) in &spec.exports {
        if !matches!(export.kind.as_str(), "folder" | "skill") {
            continue;
        }
        let folder_prefix = export.path.trim_end_matches('/');
        let files = indexed_files_under_prefix(&spec.files, folder_prefix);
        if !files.is_empty() {
            index.insert(export_name.clone(), files);
        }
    }
    for (skill_name, skill) in &spec.skills {
        if index.contains_key(skill_name) {
            continue;
        }
        let skill_prefix = skill.path.trim_end_matches('/');
        let files = indexed_files_under_prefix(&spec.files, skill_prefix);
        if !files.is_empty() {
            index.insert(skill_name.clone(), files);
        }
    }
    index
}

pub(crate) fn indexed_files_under_prefix(files: &[String], prefix: &str) -> Vec<String> {
    let mut indexed = Vec::new();
    for file in files {
        if let Some(relative) = skill_relative_file(file, prefix) {
            indexed.push(relative);
        }
    }
    indexed
}

pub(crate) fn skill_relative_file(file: &str, skill_prefix: &str) -> Option<String> {
    let relative = file.strip_prefix(skill_prefix)?.trim_start_matches('/');
    if relative.is_empty() || file == skill_prefix {
        None
    } else {
        Some(relative.to_string())
    }
}
