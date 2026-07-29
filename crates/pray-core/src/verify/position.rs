use crate::lockfile::ManagedSpanRecord;
use crate::resolve::ResolvedLocalFile;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionDriftSummary {
    pub target_path: String,
    pub marker_count: usize,
    pub uniform_delta: Option<isize>,
    pub first_id: String,
    pub lock_open: usize,
    pub lock_close: usize,
    pub file_open: usize,
    pub file_close: usize,
    pub cause: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DriftedMarker {
    id: String,
    lock_open: usize,
    lock_close: usize,
    file_open: usize,
    file_close: usize,
}

pub fn summarize_position_drift(
    target_path: &str,
    spans: &[&ManagedSpanRecord],
    markers: &BTreeMap<String, (usize, usize, String)>,
    on_disk_lines: &[&str],
    fresh_lines: Option<&[&str]>,
    local_files: &[ResolvedLocalFile],
) -> Option<PositionDriftSummary> {
    let mut drifted = Vec::new();
    for span in spans {
        let Some((open_line, close_line, checksum)) = markers.get(&span.id) else {
            continue;
        };
        if checksum != &span.ideal_checksum {
            continue;
        }
        if *open_line == span.open_line && *close_line == span.close_line {
            continue;
        }
        drifted.push(DriftedMarker {
            id: span.id.clone(),
            lock_open: span.open_line,
            lock_close: span.close_line,
            file_open: *open_line,
            file_close: *close_line,
        });
    }
    if drifted.is_empty() {
        return None;
    }
    drifted.sort_by_key(|marker| marker.file_open);
    let first = &drifted[0];
    let deltas: Vec<isize> = drifted
        .iter()
        .map(|marker| marker.file_open as isize - marker.lock_open as isize)
        .collect();
    let uniform_delta = deltas
        .iter()
        .all(|delta| *delta == deltas[0])
        .then_some(deltas[0]);
    let cause = fresh_lines
        .and_then(|fresh| unmarked_drift_cause(target_path, on_disk_lines, fresh, local_files));
    Some(PositionDriftSummary {
        target_path: target_path.to_string(),
        marker_count: drifted.len(),
        uniform_delta,
        first_id: first.id.clone(),
        lock_open: first.lock_open,
        lock_close: first.lock_close,
        file_open: first.file_open,
        file_close: first.file_close,
        cause,
    })
}

pub fn format_position_drift_message(summary: &PositionDriftSummary) -> String {
    let shift = match summary.uniform_delta {
        Some(delta) if delta != 0 => format!(" ({delta:+} lines)"),
        _ => String::new(),
    };
    let marker_word = if summary.marker_count == 1 {
        "marker"
    } else {
        "markers"
    };
    let mut parts = vec![
        format!(
            "`{}` position drift{shift} across {} {marker_word}",
            summary.target_path, summary.marker_count
        ),
        format!(
            "first marker `{}` lock {}:{}, file {}:{}",
            summary.first_id,
            summary.lock_open,
            summary.lock_close,
            summary.file_open,
            summary.file_close
        ),
    ];
    if let Some(cause) = &summary.cause {
        parts.push(format!("cause: {cause}"));
    }
    parts.push(
        "Align unmarked text with compose sources, or run `pray install` to refresh lock positions."
            .to_string(),
    );
    parts.join("; ")
}

fn unmarked_drift_cause(
    target_path: &str,
    on_disk_lines: &[&str],
    fresh_lines: &[&str],
    local_files: &[ResolvedLocalFile],
) -> Option<String> {
    let disk_preamble = preamble_lines(on_disk_lines);
    let fresh_preamble = preamble_lines(fresh_lines);
    let (index, disk_line, fresh_line) = first_line_diff(&disk_preamble, &fresh_preamble)?;
    let target_line = index + 1;
    if let Some((path, line)) = locate_line_in_locals(local_files, fresh_line) {
        return Some(format!(
            "`{target_path}:{target_line}` unmarked text differs from `{path}:{line}`"
        ));
    }
    if let Some((path, line)) = locate_line_in_locals(local_files, disk_line) {
        return Some(format!(
            "`{target_path}:{target_line}` unmarked text differs from `{path}:{line}`"
        ));
    }
    Some(format!(
        "`{target_path}:{target_line}` unmarked text differs from fresh composition"
    ))
}

fn preamble_lines<'a>(lines: &[&'a str]) -> Vec<&'a str> {
    let mut preamble = Vec::new();
    for line in lines {
        if is_managed_marker(line) {
            break;
        }
        preamble.push(*line);
    }
    preamble
}

fn is_managed_marker(line: &str) -> bool {
    let trimmed = line.trim();
    let Some(remainder) = trimmed.strip_prefix("<!-- pray:") else {
        return false;
    };
    let Some(id) = remainder.strip_suffix(" -->") else {
        return false;
    };
    id != "0 ignore-comments"
        && id
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
}

fn first_line_diff<'a>(left: &[&'a str], right: &[&'a str]) -> Option<(usize, &'a str, &'a str)> {
    let shared = left.len().min(right.len());
    for index in 0..shared {
        if left[index] != right[index] {
            return Some((index, left[index], right[index]));
        }
    }
    if left.len() == right.len() {
        return None;
    }
    let index = shared;
    Some((
        index,
        left.get(index).copied().unwrap_or(""),
        right.get(index).copied().unwrap_or(""),
    ))
}

fn locate_line_in_locals<'a>(
    local_files: &'a [ResolvedLocalFile],
    line: &str,
) -> Option<(&'a str, usize)> {
    if line.is_empty() {
        return None;
    }
    for local in local_files {
        for (index, candidate) in local.content.lines().enumerate() {
            if candidate == line {
                return Some((local.manifest_path.as_str(), index + 1));
            }
        }
    }
    None
}
