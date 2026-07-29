use super::{VerificationFinding, VerificationReport};
use std::collections::BTreeMap;

pub fn format_verification_report(report: &VerificationReport) -> String {
    let mut lines = Vec::new();
    for finding in &report.findings {
        push_finding_lines(&mut lines, finding, "");
    }
    lines.join("\n")
}

pub(super) fn format_drift_report(report: &VerificationReport) -> String {
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
            push_finding_lines(&mut lines, finding, "  ");
        }
    }
    lines.join("\n")
}

fn push_finding_lines(lines: &mut Vec<String>, finding: &VerificationFinding, indent: &str) {
    let mut message_lines = finding.message.lines();
    let Some(first) = message_lines.next() else {
        return;
    };
    lines.push(format!("{indent}{}: {first}", finding.kind));
    for line in message_lines {
        lines.push(format!("{indent}  {line}"));
    }
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
