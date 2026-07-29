use crate::destination::package_bound_to_tree;
use crate::environment::package_matches_environment;
use crate::paths::validate_project_relative_path;
use crate::resolve::ResolvedProject;
use crate::substitute::substitute_pray_symbols;
use crate::{PrayError, PrayResult};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PlannedProvisionedFile {
    pub path: PathBuf,
    pub source: PathBuf,
}

pub fn planned_provisioned_files(
    project: &ResolvedProject,
) -> PrayResult<Vec<PlannedProvisionedFile>> {
    let mut planned = Vec::new();
    collect_exact_file_bindings(project, &mut planned)?;
    for target in &project.manifest.targets {
        for folder_root in &target.skills {
            let destination_root = project.project_root.join(folder_root);
            for package in &project.packages {
                if !package.explicit {
                    continue;
                }
                if !package_matches_environment(
                    &package.declaration.groups,
                    project.environment.as_deref(),
                ) {
                    continue;
                }
                if !package_bound_to_tree(&package.declaration, target) {
                    continue;
                }
                collect_legacy_skill_files(project, package, &destination_root, &mut planned)?;
                collect_selected_export_files(project, package, &destination_root, &mut planned)?;
            }
        }
    }
    planned.sort_by(|left, right| left.path.cmp(&right.path));
    planned.dedup_by(|left, right| left.path == right.path);
    Ok(planned)
}

pub fn materialize_provisioned_exports(project: &ResolvedProject) -> PrayResult<()> {
    for file in planned_provisioned_files(project)? {
        let relative = validate_project_relative_path(&file.path.to_string_lossy())?;
        let destination = relative.join_root(&project.project_root);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        write_provisioned_file(&file.source, &destination, &project.manifest.symbols)?;
    }
    Ok(())
}

pub fn expected_provisioned_bytes(
    source: &Path,
    symbols: &std::collections::BTreeMap<String, String>,
) -> PrayResult<Vec<u8>> {
    let bytes = fs::read(source)?;
    match String::from_utf8(bytes) {
        Ok(text) => Ok(substitute_pray_symbols(&text, symbols)?.into_bytes()),
        Err(error) => Ok(error.into_bytes()),
    }
}

fn write_provisioned_file(
    source: &Path,
    destination: &Path,
    symbols: &std::collections::BTreeMap<String, String>,
) -> PrayResult<()> {
    fs::write(destination, expected_provisioned_bytes(source, symbols)?)?;
    Ok(())
}

fn collect_exact_file_bindings(
    project: &ResolvedProject,
    planned: &mut Vec<PlannedProvisionedFile>,
) -> PrayResult<()> {
    for package in &project.packages {
        if !package.explicit {
            continue;
        }
        let Some(destination) = &package.declaration.file else {
            continue;
        };
        if !package_matches_environment(&package.declaration.groups, project.environment.as_deref())
        {
            continue;
        }
        let mut matched = false;
        for export_name in &package.selected_exports {
            let Some(export) = package.spec.exports.get(export_name) else {
                continue;
            };
            if export.kind != "file" {
                continue;
            }
            let source = package.root.join(&export.path);
            if !source.is_file() {
                return Err(PrayError::Render(format!(
                    "file export source missing: {}",
                    source.display()
                )));
            }
            let relative = validate_project_relative_path(destination)?;
            planned.push(PlannedProvisionedFile {
                path: relative.as_path().to_path_buf(),
                source,
            });
            matched = true;
            break;
        }
        if !matched {
            return Err(PrayError::Render(format!(
                "package {} has file: \"{}\" but no selected file export",
                package.declaration.name, destination
            )));
        }
    }
    Ok(())
}

fn relative_project_path(project: &ResolvedProject, absolute: &Path) -> PathBuf {
    absolute
        .strip_prefix(&project.project_root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| absolute.to_path_buf())
}

fn collect_legacy_skill_files(
    project: &ResolvedProject,
    package: &crate::resolve::ResolvedPackage,
    destination_root: &Path,
    planned: &mut Vec<PlannedProvisionedFile>,
) -> PrayResult<()> {
    for (skill_name, skill) in &package.spec.skills {
        if legacy_skill_covered_by_export(package, skill) {
            continue;
        }
        let skill_files = package.skill_files.get(skill_name).ok_or_else(|| {
            PrayError::Render(format!(
                "package {} has no indexed files for legacy skill {}",
                package.declaration.name, skill_name
            ))
        })?;
        collect_tree_files(
            project,
            &package.root.join(&skill.path),
            &destination_root.join(skill_name),
            skill_files,
            &[],
            &[],
            planned,
        )?;
    }
    Ok(())
}

fn legacy_skill_covered_by_export(
    package: &crate::resolve::ResolvedPackage,
    skill: &crate::package_spec::PackageSkill,
) -> bool {
    package.spec.exports.iter().any(|(export_name, export)| {
        package.selected_exports.contains(export_name)
            && matches!(export.kind.as_str(), "folder" | "skill")
            && export.path.trim_end_matches('/') == skill.path.trim_end_matches('/')
    })
}

fn collect_selected_export_files(
    project: &ResolvedProject,
    package: &crate::resolve::ResolvedPackage,
    destination_root: &Path,
    planned: &mut Vec<PlannedProvisionedFile>,
) -> PrayResult<()> {
    for export_name in &package.selected_exports {
        let Some(export) = package.spec.exports.get(export_name) else {
            continue;
        };
        match export.kind.as_str() {
            "folder" | "skill" => {
                let indexed_files = package.skill_files.get(export_name).ok_or_else(|| {
                    PrayError::Render(format!(
                        "package {} has no indexed files for folder export {}",
                        package.declaration.name, export_name
                    ))
                })?;
                let destination_name = folder_destination_name(export_name, &export.path);
                collect_tree_files(
                    project,
                    &package.root.join(&export.path),
                    &destination_root.join(destination_name),
                    indexed_files,
                    &export.only,
                    &export.except,
                    planned,
                )?;
            }
            "file" => {
                if package.declaration.file.is_some() {
                    continue;
                }
                let source = package.root.join(&export.path);
                if !source.is_file() {
                    return Err(PrayError::Render(format!(
                        "file export source missing: {}",
                        source.display()
                    )));
                }
                let file_name =
                    source
                        .file_name()
                        .map(|name| name.to_owned())
                        .ok_or_else(|| {
                            PrayError::Render(format!(
                                "file export path has no file name: {}",
                                export.path
                            ))
                        })?;
                let destination = destination_root.join(export_name).join(file_name);
                planned.push(PlannedProvisionedFile {
                    path: relative_project_path(project, &destination),
                    source,
                });
            }
            _ => {}
        }
    }
    Ok(())
}

fn folder_destination_name(export_name: &str, export_path: &str) -> String {
    Path::new(export_path.trim_end_matches('/'))
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| export_name.to_string())
}

fn collect_tree_files(
    project: &ResolvedProject,
    source_root: &Path,
    destination_root: &Path,
    relative_files: &[String],
    only: &[String],
    except: &[String],
    planned: &mut Vec<PlannedProvisionedFile>,
) -> PrayResult<()> {
    if !source_root.is_dir() {
        return Err(PrayError::Render(format!(
            "folder source directory missing: {}",
            source_root.display()
        )));
    }

    if relative_files.is_empty() {
        return Err(PrayError::Render(format!(
            "no files listed in package manifest for {}",
            source_root.display()
        )));
    }

    let mut matched = false;
    for relative in relative_files {
        if !only.is_empty() && !only.iter().any(|entry| entry == relative) {
            continue;
        }
        if except.iter().any(|entry| entry == relative) {
            continue;
        }
        let source = source_root.join(relative);
        if !source.is_file() {
            return Err(PrayError::Render(format!(
                "provisioned file missing: {}",
                source.display()
            )));
        }
        let destination = destination_root.join(relative);
        planned.push(PlannedProvisionedFile {
            path: relative_project_path(project, &destination),
            source,
        });
        matched = true;
    }

    if !matched && only.is_empty() && except.is_empty() {
        return Err(PrayError::Render(format!(
            "no files listed in package manifest for {}",
            source_root.display()
        )));
    }

    Ok(())
}
