use super::VerificationFinding;
use crate::lockfile::Lockfile;
use crate::resolve::ResolvedProject;
use std::collections::BTreeMap;

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
