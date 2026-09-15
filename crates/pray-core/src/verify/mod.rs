mod format;
mod integrity;
mod locked_dest;
pub mod position;
mod provisioned;

use crate::hashing::normalize_line_endings;
use crate::lockfile::{Lockfile, ManagedSpanRecord};
use crate::render::render_project;
use crate::resolve::ResolvedProject;
use crate::{PrayError, PrayResult};
use format::format_drift_report;
pub use format::format_verification_report;
use integrity::push_package_lock_findings;
pub use locked_dest::{find_orphan_marker_findings, inspect_locked_destinations};
use locked_dest::{find_orphan_marker_findings_from_markers, marker_positions};
use position::{format_position_drift_message, summarize_position_drift};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationFinding {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct VerificationReport {
    pub findings: Vec<VerificationFinding>,
}

impl VerificationReport {
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        self.findings.iter().any(VerificationFinding::is_warning)
    }

    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(VerificationFinding::is_error)
    }
}

impl VerificationFinding {
    pub fn is_warning(&self) -> bool {
        matches!(self.kind.as_str(), "orphan_marker")
    }

    pub fn is_error(&self) -> bool {
        !self.is_warning()
    }
}

pub fn inspect_project(
    project: &ResolvedProject,
    lockfile: &Lockfile,
) -> PrayResult<VerificationReport> {
    let (report, _, _) = collect_verification_report(project, lockfile)?;
    Ok(report)
}

pub fn verify_project(
    project: &ResolvedProject,
    lockfile: &Lockfile,
    strict: bool,
) -> PrayResult<VerificationReport> {
    let report = inspect_project(project, lockfile)?;
    if report.is_clean() {
        return Ok(report);
    }

    if strict || report.has_errors() {
        Err(PrayError::Verify(format_verification_report(&report)))
    } else {
        Ok(report)
    }
}

type CollectedVerification = (
    VerificationReport,
    BTreeMap<String, String>,
    BTreeMap<String, String>,
);

fn collect_verification_report(
    project: &ResolvedProject,
    lockfile: &Lockfile,
) -> PrayResult<CollectedVerification> {
    let mut report = VerificationReport::default();
    let mut rendered_targets = BTreeMap::new();
    let fresh_targets: BTreeMap<String, String> = render_project(project)?
        .into_iter()
        .map(|target| (target.path.to_string_lossy().to_string(), target.content))
        .collect();
    if project.manifest_hash != lockfile.manifest_hash {
        report.findings.push(VerificationFinding {
            kind: "verify_error".to_string(),
            message:
                "Prayfile changed since `Prayfile.lock` was generated. Run `pray install` to refresh the lockfile."
                    .to_string(),
        });
    }

    push_package_lock_findings(project, lockfile, &mut report.findings);

    let mut target_spans: BTreeMap<String, Vec<&ManagedSpanRecord>> = BTreeMap::new();
    for span in &lockfile.managed_span {
        target_spans
            .entry(span.target.clone())
            .or_default()
            .push(span);
    }

    for (target_path, spans) in target_spans {
        let absolute_path = project.project_root.join(&target_path);
        if !absolute_path.exists() {
            report.findings.push(VerificationFinding {
                kind: "verify_error".to_string(),
                message: format!(
                    "Rendered file `{}` is missing. Run `pray install` to generate it.",
                    target_path
                ),
            });
            continue;
        }
        let text = crate::render_file::read_destination_text(&absolute_path)?;
        rendered_targets.insert(target_path.clone(), text.clone());
        let lines: Vec<&str> = text.lines().collect();
        let markers = marker_positions(&lines);
        for span in &spans {
            match markers.get(&span.id) {
                None => report.findings.push(VerificationFinding {
                    kind: "removed_prayer".to_string(),
                    message: format!(
                        "`{}` is missing managed marker `{}` for `{}::{}`. Run `pray install` to restore the managed span.",
                        target_path, span.id, span.package, span.export
                    ),
                }),
                Some((_, _, checksum)) => {
                    if checksum != &span.ideal_checksum {
                        report.findings.push(VerificationFinding {
                            kind: "custom_implementation".to_string(),
                            message: format!(
                                "`{}` marker `{}` (`{}::{}`) was edited. Restore the managed block or run `pray install` to regenerate it.",
                                target_path, span.id, span.package, span.export
                            ),
                        });
                    }
                }
            }
        }
        let fresh_lines: Vec<&str> = fresh_targets
            .get(&target_path)
            .map(|fresh| fresh.lines().collect())
            .unwrap_or_default();
        if let Some(summary) = summarize_position_drift(
            &target_path,
            &spans,
            &markers,
            &lines,
            fresh_targets
                .contains_key(&target_path)
                .then_some(fresh_lines.as_slice()),
            &project.local_files,
        ) {
            report.findings.push(VerificationFinding {
                kind: "position_drift".to_string(),
                message: format_position_drift_message(&summary),
            });
        }
        for finding in find_orphan_marker_findings_from_markers(&spans, &markers, &target_path) {
            report.findings.push(finding);
        }
    }

    provisioned::push_provisioned_and_local_findings(project, lockfile, &mut report.findings)?;

    Ok((report, rendered_targets, fresh_targets))
}

pub fn drift_project(
    project: &ResolvedProject,
    lockfile: &Lockfile,
) -> PrayResult<VerificationReport> {
    let (mut report, rendered_targets, fresh_targets) =
        collect_verification_report(project, lockfile)?;

    let lock_targets = lockfile_targets(lockfile);
    for (path, fresh_content) in &fresh_targets {
        let normalized_fresh = normalize_line_endings(fresh_content);
        let on_disk = rendered_targets
            .get(path)
            .map(|text| normalize_line_endings(text));
        let matches = on_disk.as_ref() == Some(&normalized_fresh);
        if !matches {
            report.findings.push(VerificationFinding {
                kind: "renderer_drift".to_string(),
                message: format!("{path} differs from fresh render"),
            });
        }
        if !lock_targets.contains(path) {
            report.findings.push(VerificationFinding {
                kind: "renderer_drift".to_string(),
                message: format!("{path} is not tracked in lockfile"),
            });
        }
    }

    if report.findings.is_empty() {
        Ok(report)
    } else {
        Err(PrayError::Verify(format_drift_report(&report)))
    }
}

fn lockfile_targets(lockfile: &Lockfile) -> BTreeSet<String> {
    lockfile
        .target
        .iter()
        .flat_map(|target| target.outputs.iter().cloned())
        .collect()
}
