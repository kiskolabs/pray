use super::VerificationFinding;
use crate::hashing::sha256_prefixed;
use crate::lockfile::Lockfile;
use crate::render::expected_provisioned_bytes;
use crate::resolve::{missing_local_embed_guidance, ResolvedProject};
use crate::PrayResult;
use std::collections::BTreeMap;
use std::fs;

pub(super) fn push_package_lock_findings(
    project: &ResolvedProject,
    lockfile: &Lockfile,
    report_findings: &mut Vec<VerificationFinding>,
) {
    let mut locked_packages: BTreeMap<String, &crate::lockfile::LockedPackage> = lockfile
        .package
        .iter()
        .map(|package| (package.name.clone(), package))
        .collect();
    for package in &project.packages {
        match locked_packages.remove(&package.declaration.name) {
            Some(locked) => {
                if locked.tree_hash != package.tree_hash {
                    report_findings.push(VerificationFinding {
                        kind: "package_integrity".to_string(),
                        message: format!(
                            "Package `{}` no longer matches the locked tree hash. Run `pray install` to re-resolve packages.",
                            package.declaration.name
                        ),
                    });
                }
                if locked.version != package.spec.version {
                    report_findings.push(VerificationFinding {
                        kind: "verify_error".to_string(),
                        message: format!(
                            "Package `{}` resolved to version {} but `Prayfile.lock` has {}. Run `pray install` to refresh the lockfile.",
                            package.declaration.name, package.spec.version, locked.version
                        ),
                    });
                }
            }
            None => report_findings.push(VerificationFinding {
                kind: "verify_error".to_string(),
                message: format!(
                    "Package `{}` is declared in Prayfile but missing from `Prayfile.lock`. Run `pray install` to update the lockfile.",
                    package.declaration.name
                ),
            }),
        }
    }
    for locked in locked_packages.values() {
        report_findings.push(VerificationFinding {
            kind: "verify_error".to_string(),
            message: format!(
                "Package `{}` is in `Prayfile.lock` but not declared in Prayfile. Remove it from the lockfile with `pray install` or add it back to Prayfile.",
                locked.name
            ),
        });
    }
}

pub(super) fn push_provisioned_and_local_findings(
    project: &ResolvedProject,
    report_findings: &mut Vec<VerificationFinding>,
) -> PrayResult<()> {
    for package in &project.packages {
        let Some(destination) = &package.declaration.file else {
            continue;
        };
        let absolute = project.project_root.join(destination);
        let Some(export_name) = package.selected_exports.iter().find(|name| {
            package
                .spec
                .exports
                .get(*name)
                .is_some_and(|export| export.kind == "file")
        }) else {
            report_findings.push(VerificationFinding {
                kind: "verify_error".to_string(),
                message: format!(
                    "Package `{}` declares file: \"{}\" but has no selected file export.",
                    package.declaration.name, destination
                ),
            });
            continue;
        };
        let source = package.root.join(&package.spec.exports[export_name].path);
        if !absolute.exists() {
            report_findings.push(VerificationFinding {
                kind: "verify_error".to_string(),
                message: format!(
                    "Exclusive file `{}` from `{}` is missing. Run `pray install` to materialize it.",
                    destination, package.declaration.name
                ),
            });
            continue;
        }
        let destination_bytes = fs::read(&absolute)?;
        let expected_bytes = expected_provisioned_bytes(&source, &project.manifest.symbols)?;
        if sha256_prefixed(&destination_bytes) != sha256_prefixed(&expected_bytes) {
            report_findings.push(VerificationFinding {
                kind: "package_integrity".to_string(),
                message: format!(
                    "Exclusive file `{}` no longer matches package `{}`. Run `pray install` to restore it.",
                    destination, package.declaration.name
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
