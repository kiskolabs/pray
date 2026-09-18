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
/// When `fresh` introduces managed spans before the first shared id, take that
/// leading region from `fresh` so local compose sources can gain markers and update.
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
    let existing_ids: std::collections::BTreeSet<&str> = existing_segments
        .iter()
        .filter_map(|segment| match segment {
            Segment::Managed { id, .. } => Some(id.as_str()),
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
    let remaining_existing = if fresh_managed
        .keys()
        .any(|id| !existing_ids.contains(id.as_str()))
    {
        let Some(shared_id) = fresh_segments.iter().find_map(|segment| match segment {
            Segment::Managed { id, .. } if existing_ids.contains(id.as_str()) => Some(id.as_str()),
            _ => None,
        }) else {
            return fresh.to_string();
        };
        for segment in &fresh_segments {
            match segment {
                Segment::Managed { id, .. } if id == shared_id => break,
                Segment::Text(text) => output.push_str(text),
                Segment::Managed { id, body } => {
                    used.insert(id.clone());
                    push_managed_span(&mut output, id, body);
                }
            }
        }
        skip_until_managed(&existing_segments, shared_id)
    } else {
        existing_segments.as_slice()
    };
    for (index, segment) in remaining_existing.iter().enumerate() {
        match segment {
            Segment::Text(text) => output.push_str(text),
            Segment::Managed { id, body } => {
                let replacement = fresh_managed
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| body.clone());
                used.insert(id.clone());
                push_managed_span(&mut output, id, &replacement);
                if let Some(Segment::Managed { id: next_id, .. }) =
                    remaining_existing.get(index + 1)
                {
                    output.push_str(&text_between(&fresh_segments, id, next_id));
                }
            }
        }
    }
    append_unused_fresh_spans(&fresh_segments, &used, &mut output);
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn text_between(segments: &[Segment], left: &str, right: &str) -> String {
    let mut copying = false;
    let mut text = String::new();
    for segment in segments {
        match segment {
            Segment::Managed { id, .. } if id == left => {
                copying = true;
                text.clear();
            }
            Segment::Managed { id, .. } if copying => {
                return if id == right { text } else { String::new() };
            }
            Segment::Text(body) if copying => text.push_str(body),
            _ => {}
        }
    }
    String::new()
}

fn append_unused_fresh_spans(
    fresh_segments: &[Segment],
    used: &std::collections::BTreeSet<String>,
    output: &mut String,
) {
    let mut pending_text = String::new();
    let mut seen_used = false;
    for segment in fresh_segments {
        match segment {
            Segment::Managed { id, body } => {
                if used.contains(id) {
                    seen_used = true;
                    pending_text.clear();
                    continue;
                }
                output.push_str(&pending_text);
                pending_text.clear();
                push_managed_span(output, id, body);
            }
            Segment::Text(text) => {
                if seen_used {
                    pending_text.push_str(text);
                }
            }
        }
    }
}

fn skip_until_managed<'a>(segments: &'a [Segment], id: &str) -> &'a [Segment] {
    match segments.iter().position(|segment| match segment {
        Segment::Managed { id: found, .. } => found == id,
        Segment::Text(_) => false,
    }) {
        Some(index) => &segments[index..],
        None => segments,
    }
}

fn push_managed_span(output: &mut String, id: &str, body: &str) {
    output.push_str(&format!("<!-- pray:{id} -->\n"));
    if !body.is_empty() {
        output.push_str(body.trim_end_matches('\n'));
        output.push('\n');
    }
    output.push_str(&format!("<!-- pray:{id} -->\n"));
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
