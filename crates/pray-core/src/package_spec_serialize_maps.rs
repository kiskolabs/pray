use super::{quote, string_array};
use crate::package_spec::{PackageExport, PackageSkill, PackageTemplate};
use std::collections::BTreeMap;

pub(super) fn push_exports(lines: &mut Vec<String>, exports: &BTreeMap<String, PackageExport>) {
    if exports.is_empty() {
        return;
    }
    lines.push("  spec.exports = {".to_string());
    for (name, export) in exports {
        let mut fields = vec![
            format!("type: {}", quote(&export.kind)),
            format!("path: {}", quote(&export.path)),
        ];
        if let Some(summary) = &export.summary {
            fields.push(format!("summary: {}", quote(summary)));
        }
        if !export.only.is_empty() {
            fields.push(format!("only: [{}]", string_array(&export.only)));
        }
        if !export.except.is_empty() {
            fields.push(format!("except: [{}]", string_array(&export.except)));
        }
        if let Some(default_path) = &export.default_path {
            fields.push(format!("default_path: {}", quote(default_path)));
        }
        lines.push(format!(
            "    {} => {{ {} }},",
            quote(name),
            fields.join(", ")
        ));
    }
    lines.push("  }".to_string());
}

pub(super) fn push_skills(lines: &mut Vec<String>, entries: &BTreeMap<String, PackageSkill>) {
    if entries.is_empty() {
        return;
    }
    lines.push("  spec.skills = {".to_string());
    for (name, entry) in entries {
        let summary = entry
            .summary
            .as_ref()
            .map(|value| format!(", summary: {}", quote(value)))
            .unwrap_or_default();
        lines.push(format!(
            "    {} => {{ path: {}{summary} }},",
            quote(name),
            quote(&entry.path)
        ));
    }
    lines.push("  }".to_string());
}

pub(super) fn push_templates(lines: &mut Vec<String>, entries: &BTreeMap<String, PackageTemplate>) {
    if entries.is_empty() {
        return;
    }
    lines.push("  spec.templates = {".to_string());
    for (name, entry) in entries {
        let summary = entry
            .summary
            .as_ref()
            .map(|value| format!(", summary: {}", quote(value)))
            .unwrap_or_default();
        lines.push(format!(
            "    {} => {{ path: {}{summary} }},",
            quote(name),
            quote(&entry.path)
        ));
    }
    lines.push("  }".to_string());
}

pub(super) fn push_strings(
    lines: &mut Vec<String>,
    field: &str,
    entries: &BTreeMap<String, String>,
) {
    if entries.is_empty() {
        return;
    }
    let values = entries
        .iter()
        .map(|(name, value)| format!("{} => {}", quote(name), quote(value)))
        .collect::<Vec<_>>()
        .join(", ");
    lines.push(format!("  spec.{field} = {{ {values} }}"));
}
