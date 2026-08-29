use crate::lockfile::ManagedSpanRecord;
use std::collections::BTreeMap;

pub fn relocate_managed_spans(
    content: &str,
    spans: &[ManagedSpanRecord],
) -> Vec<ManagedSpanRecord> {
    let lines: Vec<&str> = content.lines().collect();
    let positions = marker_positions(&lines);
    spans
        .iter()
        .map(|span| match positions.get(&span.id) {
            Some((open_line, close_line)) => {
                let mut relocated = span.clone();
                relocated.open_line = *open_line;
                relocated.close_line = *close_line;
                relocated
            }
            None => span.clone(),
        })
        .collect()
}

fn marker_positions(lines: &[&str]) -> BTreeMap<String, (usize, usize)> {
    let mut markers = BTreeMap::new();
    let mut active: Option<(String, usize)> = None;
    for (index, line) in lines.iter().enumerate() {
        let Some(id) = marker_id(line) else {
            continue;
        };
        match active.take() {
            None => {
                active = Some((id, index + 1));
            }
            Some((open_id, open_line)) if open_id == id => {
                markers.insert(open_id, (open_line, index + 1));
            }
            Some(previous) => {
                active = Some(previous);
            }
        }
    }
    markers
}

fn marker_id(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let remainder = trimmed.strip_prefix("<!-- pray:")?;
    let id = remainder.strip_suffix(" -->")?;
    if id == "0 ignore-comments" {
        return None;
    }
    if id
        .chars()
        .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    {
        return Some(id.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(id: &str, open_line: usize, close_line: usize) -> ManagedSpanRecord {
        ManagedSpanRecord {
            id: id.to_string(),
            target: "AGENTS.md".to_string(),
            open_line,
            close_line,
            ideal_checksum: "sha256:body".to_string(),
            package: "sample/base".to_string(),
            export: "guidance".to_string(),
            source_checksum: "sha256:source".to_string(),
            silenced: false,
        }
    }

    #[test]
    fn moves_span_lines_to_markers_in_patched_content() {
        let content = "\
keep this unmarked line

<!-- pray:abc123 -->
body
<!-- pray:abc123 -->
";
        let relocated = relocate_managed_spans(content, &[span("abc123", 4, 6)]);
        assert_eq!(relocated[0].open_line, 3);
        assert_eq!(relocated[0].close_line, 5);
        assert_eq!(relocated[0].ideal_checksum, "sha256:body");
    }
}
