use super::{VerificationFinding, VerificationReport};
use crate::hashing::checksum_managed_body_line_refs;
use crate::lockfile::{Lockfile, ManagedSpanRecord};
use crate::PrayResult;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

/// Check dest files named by `Prayfile.lock` managed spans without resolve or render (RFC 0106).
pub fn inspect_locked_destinations(
    project_root: &Path,
    lockfile: &Lockfile,
) -> PrayResult<VerificationReport> {
    let mut report = VerificationReport::default();
    let mut target_spans: BTreeMap<String, Vec<&ManagedSpanRecord>> = BTreeMap::new();
    for span in &lockfile.managed_span {
        target_spans
            .entry(span.target.clone())
            .or_default()
            .push(span);
    }
    for (target_path, spans) in target_spans {
        let absolute_path = project_root.join(&target_path);
        if !absolute_path.exists() {
            report.findings.push(VerificationFinding {
                kind: "verify_error".to_string(),
                message: format!(
                    "Rendered file `{target_path}` is missing. Run `pray install` to generate it."
                ),
            });
            continue;
        }
        let text = crate::render_file::read_destination_text(&absolute_path)?;
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
        for finding in find_orphan_marker_findings_from_markers(&spans, &markers, &target_path) {
            report.findings.push(finding);
        }
    }
    Ok(report)
}

pub fn find_orphan_marker_findings(
    spans: &[&ManagedSpanRecord],
    lines: &[&str],
    target_path: &str,
) -> Vec<VerificationFinding> {
    let markers = marker_positions(lines);
    find_orphan_marker_findings_from_markers(spans, &markers, target_path)
}

pub(super) fn find_orphan_marker_findings_from_markers(
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
                    "`{target_path}` contains marker `{marker_id}` that is not tracked in `Prayfile.lock`. Remove the marker or run `pray install` to reconcile."
                ),
            });
        }
    }
    findings
}

pub(super) fn marker_positions(lines: &[&str]) -> BTreeMap<String, (usize, usize, String)> {
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
