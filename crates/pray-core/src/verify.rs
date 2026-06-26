use crate::hashing::{normalize_line_endings, sha256_prefixed};
use crate::lockfile::{Lockfile, ManagedSpanRecord};
use crate::render::render_project;
use crate::resolve::ResolvedProject;
use crate::{PrayError, PrayResult};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

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

pub fn verify_project(
    project: &ResolvedProject,
    lockfile: &Lockfile,
    strict: bool,
) -> PrayResult<VerificationReport> {
    let report = collect_verification_report(project, lockfile)?;
    if report.is_clean() {
        return Ok(report);
    }

    if strict || report.has_errors() {
        Err(PrayError::Verify(format_verification_report(&report)))
    } else {
        Ok(report)
    }
}

fn collect_verification_report(
    project: &ResolvedProject,
    lockfile: &Lockfile,
) -> PrayResult<VerificationReport> {
    let mut report = VerificationReport::default();
    let manifest_hash = project.manifest.manifest_hash()?;
    if manifest_hash != lockfile.manifest_hash {
        report.findings.push(VerificationFinding {
            kind: "verify_error".to_string(),
            message: "manifest hash differs from lockfile".to_string(),
        });
    }

    let mut locked_packages: BTreeMap<String, &crate::lockfile::LockedPackage> = lockfile
        .package
        .iter()
        .map(|package| (package.name.clone(), package))
        .collect();
    for package in &project.packages {
        match locked_packages.remove(&package.declaration.name) {
            Some(locked) => {
                if locked.tree_hash != package.tree_hash {
                    report.findings.push(VerificationFinding {
                        kind: "package_integrity".to_string(),
                        message: format!("package {} tree hash mismatch", package.declaration.name),
                    });
                }
                if locked.version != package.spec.version {
                    report.findings.push(VerificationFinding {
                        kind: "verify_error".to_string(),
                        message: format!(
                            "package {} version differs: lock {} vs current {}",
                            package.declaration.name, locked.version, package.spec.version
                        ),
                    });
                }
            }
            None => report.findings.push(VerificationFinding {
                kind: "verify_error".to_string(),
                message: format!("package {} missing from lockfile", package.declaration.name),
            }),
        }
    }
    for locked in locked_packages.values() {
        report.findings.push(VerificationFinding {
            kind: "verify_error".to_string(),
            message: format!("lockfile contains unexpected package {}", locked.name),
        });
    }

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
                message: format!("missing target file: {}", target_path),
            });
            continue;
        }
        let text = fs::read_to_string(&absolute_path)?;
        let lines: Vec<&str> = text.lines().collect();
        let markers = marker_positions(&lines);
        let marker_ids: BTreeSet<String> = markers.keys().cloned().collect();
        for span in &spans {
            match markers.get(&span.id) {
                None => report.findings.push(VerificationFinding {
                    kind: "removed_prayer".to_string(),
                    message: format!("{} missing marker pair {}", target_path, span.id),
                }),
                Some((open_line, close_line, body)) => {
                    let actual_checksum = sha256_prefixed(normalize_line_endings(body).as_bytes());
                    if actual_checksum != span.ideal_checksum {
                        report.findings.push(VerificationFinding {
                            kind: "custom_implementation".to_string(),
                            message: format!("{} marker {} body changed", target_path, span.id),
                        });
                    }
                    if *open_line != span.open_line || *close_line != span.close_line {
                        report.findings.push(VerificationFinding {
                            kind: "position_drift".to_string(),
                            message: format!("{} marker {} moved", target_path, span.id),
                        });
                    }
                }
            }
        }
        for marker_id in marker_ids {
            if !spans.iter().any(|span| span.id == marker_id) && marker_id != "0" {
                report.findings.push(VerificationFinding {
                    kind: "orphan_marker".to_string(),
                    message: format!("{} has orphan marker {}", target_path, marker_id),
                });
            }
        }
    }

    for local in &project.local_files {
        if local.optional {
            continue;
        }
        if !project.project_root.join(&local.path).exists() {
            report.findings.push(VerificationFinding {
                kind: "verify_error".to_string(),
                message: format!("missing local file: {}", local.path.display()),
            });
        }
    }

    Ok(report)
}

pub fn drift_project(
    project: &ResolvedProject,
    lockfile: &Lockfile,
) -> PrayResult<VerificationReport> {
    let mut report = collect_verification_report(project, lockfile)?;

    let rendered = render_project(project)?;
    let lock_targets = lockfile_targets(lockfile);
    for target in rendered {
        let on_disk = fs::read_to_string(project.project_root.join(&target.path))?;
        if normalize_line_endings(&on_disk) != normalize_line_endings(&target.content) {
            report.findings.push(VerificationFinding {
                kind: "renderer_drift".to_string(),
                message: format!("{} differs from fresh render", target.path.display()),
            });
        }
        if !lock_targets.contains(&target.path.to_string_lossy().to_string()) {
            report.findings.push(VerificationFinding {
                kind: "renderer_drift".to_string(),
                message: format!("{} is not tracked in lockfile", target.path.display()),
            });
        }
    }

    if report.findings.is_empty() {
        Ok(report)
    } else {
        Err(PrayError::Verify(format_drift_report(&report)))
    }
}

fn marker_positions(lines: &[&str]) -> BTreeMap<String, (usize, usize, String)> {
    let mut markers = BTreeMap::new();
    let mut active: Option<(String, usize, Vec<String>)> = None;
    for (index, line) in lines.iter().enumerate() {
        if let Some(id) = parse_marker(line) {
            if id == "0" {
                continue;
            }
            match active.take() {
                None => {
                    active = Some((id, index + 1, Vec::new()));
                }
                Some((open_id, open_line, body)) if open_id == id => {
                    markers.insert(id, (open_line, index + 1, body.join("\n")));
                }
                Some(previous) => {
                    active = Some(previous);
                }
            }
            continue;
        }
        if let Some((_, _, body)) = active.as_mut() {
            body.push((*line).to_string());
        }
    }
    markers
}

fn parse_marker(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let remainder = trimmed.strip_prefix("<!-- pray:")?;
    let id = remainder.strip_suffix(" -->")?;
    if id == "0 ignore-comments" {
        return Some("0".to_string());
    }
    if id
        .chars()
        .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    {
        return Some(id.to_string());
    }
    None
}

fn lockfile_targets(lockfile: &Lockfile) -> BTreeSet<String> {
    lockfile
        .target
        .iter()
        .flat_map(|target| target.outputs.iter().cloned())
        .collect()
}

pub fn format_verification_report(report: &VerificationReport) -> String {
    report
        .findings
        .iter()
        .map(|finding| format!("{}: {}", finding.kind, finding.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_drift_report(report: &VerificationReport) -> String {
    let mut sections: BTreeMap<&'static str, Vec<&VerificationFinding>> = BTreeMap::new();
    for finding in &report.findings {
        sections
            .entry(drift_section_for_kind(&finding.kind))
            .or_default()
            .push(finding);
    }

    let ordered_sections = [
        "Lockfile changes",
        "Package changes",
        "Managed span changes",
        "Rendered file changes",
        "Warnings",
    ];
    let mut lines = Vec::new();
    for section in ordered_sections {
        let Some(findings) = sections.get(section) else {
            continue;
        };
        lines.push(section.to_string());
        for finding in findings {
            lines.push(format!("  {}: {}", finding.kind, finding.message));
        }
    }
    lines.join("\n")
}

fn drift_section_for_kind(kind: &str) -> &'static str {
    match kind {
        "verify_error" => "Lockfile changes",
        "package_integrity" => "Package changes",
        "custom_implementation" | "removed_prayer" | "position_drift" | "orphan_marker" => {
            "Managed span changes"
        }
        "renderer_drift" => "Rendered file changes",
        _ => "Warnings",
    }
}
