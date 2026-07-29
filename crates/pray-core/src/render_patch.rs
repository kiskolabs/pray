use crate::hashing::checksum_managed_body_line_refs;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Text(String),
    Managed { id: String, body: String },
}

/// Preserve unmarked text from `existing` while replacing managed spans from `fresh`.
///
/// When `existing` has no managed spans that overlap `fresh`, return `fresh` wholesale
/// so corrupted or empty destinations can be repaired by a full rewrite.
pub fn patch_rendered_content(existing: &str, fresh: &str) -> String {
    let existing_segments = split_segments(existing);
    let fresh_segments = split_segments(fresh);
    let fresh_managed: BTreeMap<String, String> = fresh_segments
        .iter()
        .filter_map(|segment| match segment {
            Segment::Managed { id, body } => Some((id.clone(), body.clone())),
            Segment::Text(_) => None,
        })
        .collect();
    let existing_overlap = existing_segments.iter().any(|segment| match segment {
        Segment::Managed { id, .. } => fresh_managed.contains_key(id),
        Segment::Text(_) => false,
    });
    if !existing_overlap {
        return fresh.to_string();
    }
    let mut used = std::collections::BTreeSet::new();
    let mut output = String::new();
    for segment in existing_segments {
        match segment {
            Segment::Text(text) => output.push_str(&text),
            Segment::Managed { id, body } => {
                let replacement = fresh_managed.get(&id).cloned().unwrap_or(body);
                used.insert(id.clone());
                output.push_str(&format!("<!-- pray:{id} -->\n"));
                if !replacement.is_empty() {
                    output.push_str(replacement.trim_end_matches('\n'));
                    output.push('\n');
                }
                output.push_str(&format!("<!-- pray:{id} -->\n"));
            }
        }
    }
    for segment in fresh_segments {
        if let Segment::Managed { id, body } = segment {
            if used.contains(&id) {
                continue;
            }
            output.push_str(&format!("<!-- pray:{id} -->\n"));
            if !body.is_empty() {
                output.push_str(body.trim_end_matches('\n'));
                output.push('\n');
            }
            output.push_str(&format!("<!-- pray:{id} -->\n"));
        }
    }
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn split_segments(content: &str) -> Vec<Segment> {
    let lines: Vec<&str> = content.lines().collect();
    let mut segments = Vec::new();
    let mut text = String::new();
    let mut index = 0usize;
    while index < lines.len() {
        if let Some(id) = marker_id(lines[index]) {
            if let Some(close) = find_closing_marker(&lines, index + 1, &id) {
                if !text.is_empty() {
                    segments.push(Segment::Text(std::mem::take(&mut text)));
                }
                let body_lines = &lines[index + 1..close];
                let body = if body_lines.is_empty() {
                    String::new()
                } else {
                    let mut body = body_lines.join("\n");
                    body.push('\n');
                    body
                };
                let _ = checksum_managed_body_line_refs(body_lines);
                segments.push(Segment::Managed { id, body });
                index = close + 1;
                continue;
            }
        }
        text.push_str(lines[index]);
        text.push('\n');
        index += 1;
    }
    if !text.is_empty() {
        segments.push(Segment::Text(text));
    }
    segments
}

fn find_closing_marker(lines: &[&str], start: usize, id: &str) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, line)| (marker_id(line).as_deref() == Some(id)).then_some(index))
}

fn marker_id(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let remainder = trimmed.strip_prefix("<!-- pray:")?;
    let id = remainder.strip_suffix(" -->")?;
    if id == "0 ignore-comments" {
        return None;
    }
    Some(id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_unmarked_text_and_updates_managed_body() {
        let existing = "\
## Shared instructions

User note: keep this line.

<!-- pray:abc123 -->
old body
<!-- pray:abc123 -->
";
        let fresh = "\
## Shared instructions

<!-- pray:abc123 -->
new body
<!-- pray:abc123 -->
";
        let patched = patch_rendered_content(existing, fresh);
        assert!(patched.contains("User note: keep this line."));
        assert!(patched.contains("new body"));
        assert!(!patched.contains("old body"));
    }

    #[test]
    fn rewrites_wholesale_when_existing_has_no_managed_overlap() {
        let existing = "broken rendered output\n";
        let fresh = "\
<!-- pray:abc123 -->
new body
<!-- pray:abc123 -->
";
        assert_eq!(patch_rendered_content(existing, fresh), fresh);
    }
}
