use super::VerificationFinding;
use crate::hashing::sha256_prefixed;
use crate::render::{expected_provisioned_bytes, planned_provisioned_files};
use crate::resolve::{missing_local_embed_guidance, ResolvedProject};
use crate::PrayResult;
use std::fs;

pub(super) fn push_provisioned_and_local_findings(
    project: &ResolvedProject,
    report_findings: &mut Vec<VerificationFinding>,
) -> PrayResult<()> {
    push_exclusive_file_export_findings(project, report_findings);
    for file in planned_provisioned_files(project)? {
        let path_text = file.path.to_string_lossy().replace('\\', "/");
        let absolute = project.project_root.join(&file.path);
        match fs::symlink_metadata(&absolute) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                report_findings.push(VerificationFinding {
                    kind: "verify_error".to_string(),
                    message: format!(
                        "Provisioned file `{path_text}` is a symbolic link. Remove the link or choose another destination."
                    ),
                });
                continue;
            }
            Ok(_) => {}
            Err(_) => {
                report_findings.push(VerificationFinding {
                    kind: "verify_error".to_string(),
                    message: format!(
                        "Provisioned file `{path_text}` from `{}` is missing. Run `pray install` to materialize it.",
                        file.package
                    ),
                });
                continue;
            }
        }
        let destination_bytes = fs::read(&absolute)?;
        let expected_bytes = expected_provisioned_bytes(&file.source, &project.manifest.symbols)?;
        if sha256_prefixed(&destination_bytes) != sha256_prefixed(&expected_bytes) {
            report_findings.push(VerificationFinding {
                kind: "package_integrity".to_string(),
                message: format!(
                    "Provisioned file `{path_text}` no longer matches package `{}`. Run `pray install` to restore it.",
                    file.package
                ),
            });
        }
    }

    for local in &project.local_files {
        if local.optional {
            continue;
        }
        if !project.project_root.join(&local.path).exists() {
            report_findings.push(VerificationFinding {
                kind: "verify_error".to_string(),
                message: missing_local_embed_guidance(&local.manifest_path),
            });
        }
    }
    Ok(())
}

fn push_exclusive_file_export_findings(
    project: &ResolvedProject,
    report_findings: &mut Vec<VerificationFinding>,
) {
    for package in &project.packages {
        let Some(destination) = &package.declaration.file else {
            continue;
        };
        let has_file_export = package.selected_exports.iter().any(|name| {
            package
                .spec
                .exports
                .get(name)
                .is_some_and(|export| export.kind == "file")
        });
        if !has_file_export {
            report_findings.push(VerificationFinding {
                kind: "verify_error".to_string(),
                message: format!(
                    "Package `{}` declares file: \"{}\" but has no selected file export.",
                    package.declaration.name, destination
                ),
            });
        }
    }
}
