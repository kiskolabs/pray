use crate::hashing::{checksum_managed_body_line_refs, normalize_line_endings};
use crate::lockfile::{Lockfile, ManagedSpanRecord};
use crate::render::render_project;
use crate::resolve::{missing_local_embed_guidance, ResolvedProject};
use crate::{PrayError, PrayResult};
use std::collections::{BTreeMap, BTreeSet, HashSet};
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
    let (report, _) = collect_verification_report(project, lockfile)?;
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
) -> PrayResult<(VerificationReport, BTreeMap<String, String>)> {
    let mut report = VerificationReport::default();
    let mut rendered_targets = BTreeMap::new();
    if project.manifest_hash != lockfile.manifest_hash {
        report.findings.push(VerificationFinding {
            kind: "verify_error".to_string(),
            message:
                "Prayfile changed since `Prayfile.lock` was generated. Run `pray install` to refresh the lockfile."
                    .to_string(),
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
                        message: format!(
                            "Package `{}` no longer matches the locked tree hash. Run `pray install` to re-resolve packages.",
                            package.declaration.name
                        ),
                    });
                }
                if locked.version != package.spec.version {
                    report.findings.push(VerificationFinding {
                        kind: "verify_error".to_string(),
                        message: format!(
                            "Package `{}` resolved to version {} but `Prayfile.lock` has {}. Run `pray install` to refresh the lockfile.",
                            package.declaration.name, package.spec.version, locked.version
                        ),
                    });
                }
            }
            None => report.findings.push(VerificationFinding {
                kind: "verify_error".to_string(),
                message: format!(
                    "Package `{}` is declared in Prayfile but missing from `Prayfile.lock`. Run `pray install` to update the lockfile.",
                    package.declaration.name
                ),
            }),
        }
    }
    for locked in locked_packages.values() {
        report.findings.push(VerificationFinding {
            kind: "verify_error".to_string(),
            message: format!(
                "Package `{}` is in `Prayfile.lock` but not declared in Prayfile. Remove it from the lockfile with `pray install` or add it back to Prayfile.",
                locked.name
            ),
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
                message: format!(
                    "Rendered file `{}` is missing. Run `pray install` to generate it.",
                    target_path
                ),
            });
            continue;
        }
        let text = fs::read_to_string(&absolute_path)?;
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
                Some((open_line, close_line, checksum)) => {
                    if checksum != &span.ideal_checksum {
                        report.findings.push(VerificationFinding {
                            kind: "custom_implementation".to_string(),
                            message: format!(
                                "`{}` marker `{}` (`{}::{}`) was edited. Restore the managed block or run `pray install` to regenerate it.",
                                target_path, span.id, span.package, span.export
                            ),
                        });
                    }
                    if *open_line != span.open_line || *close_line != span.close_line {
                        report.findings.push(VerificationFinding {
                            kind: "position_drift".to_string(),
                            message: format!(
                                "`{}` marker `{}` (`{}::{}`) moved to different lines. Run `pray install` to restore expected positions.",
                                target_path, span.id, span.package, span.export
                            ),
                        });
                    }
                }
            }
        }
        for finding in find_orphan_marker_findings_from_markers(&spans, &markers, &target_path) {
            report.findings.push(finding);
        }
    }

    for local in &project.local_files {
        if local.optional {
            continue;
        }
        if !project.project_root.join(&local.path).exists() {
            report.findings.push(VerificationFinding {
                kind: "verify_error".to_string(),
                message: missing_local_embed_guidance(local.path.to_string_lossy()),
            });
        }
    }

    Ok((report, rendered_targets))
}

pub fn find_orphan_marker_findings(
    spans: &[&ManagedSpanRecord],
    lines: &[&str],
    target_path: &str,
) -> Vec<VerificationFinding> {
    let markers = marker_positions(lines);
    find_orphan_marker_findings_from_markers(spans, &markers, target_path)
}

fn find_orphan_marker_findings_from_markers(
    spans: &[&ManagedSpanRecord],
    markers: &BTreeMap<String, (usize, usize, String)>,
    target_path: &str,
) -> Vec<VerificationFinding> {
    let tracked_ids: HashSet<&str> = spans.iter().map(|span| span.id.as_str()).collect();
    let mut findings = Vec::new();
    for marker_id in markers.keys() {
        if marker_id != "0" && !tracked_ids.contains(marker_id.as_str()) {
            findings.push(VerificationFinding {
                kind: "orphan_marker".to_string(),
                message: format!(
                    "`{}` contains marker `{}` that is not tracked in `Prayfile.lock`. Remove the marker or run `pray install` to reconcile.",
                    target_path, marker_id
                ),
            });
        }
    }
    findings
}

pub fn drift_project(
    project: &ResolvedProject,
    lockfile: &Lockfile,
) -> PrayResult<VerificationReport> {
    let (mut report, rendered_targets) = collect_verification_report(project, lockfile)?;

    let rendered = render_project(project)?;
    let lock_targets = lockfile_targets(lockfile);
    for target in rendered {
        let normalized_fresh = normalize_line_endings(&target.content);
        let on_disk = rendered_targets
            .get(target.path.to_string_lossy().as_ref())
            .map(|text| normalize_line_endings(text));
        let matches = on_disk.as_ref() == Some(&normalized_fresh);
        if !matches {
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
    let mut active: Option<(String, usize, Vec<&str>)> = None;
    for (index, line) in lines.iter().enumerate() {
        match parse_marker(line) {
            None => {
                if let Some((_, _, body)) = active.as_mut() {
                    body.push(line);
                }
            }
            Some(ParsedMarker::Ignore) => {}
            Some(ParsedMarker::Id(id)) => match active.take() {
                None => {
                    active = Some((id.to_string(), index + 1, Vec::new()));
                }
                Some((open_id, open_line, body)) if open_id == id => {
                    let checksum = checksum_managed_body_line_refs(&body);
                    markers.insert(open_id, (open_line, index + 1, checksum));
                }
                Some(previous) => {
                    active = Some(previous);
                }
            },
        }
    }
    markers
}

enum ParsedMarker<'a> {
    Ignore,
    Id(&'a str),
}

fn parse_marker(line: &str) -> Option<ParsedMarker<'_>> {
    let trimmed = line.trim();
    let remainder = trimmed.strip_prefix("<!-- pray:")?;
    let id = remainder.strip_suffix(" -->")?;
    if id == "0 ignore-comments" {
        return Some(ParsedMarker::Ignore);
    }
    if id
        .chars()
        .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    {
        return Some(ParsedMarker::Id(id));
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
