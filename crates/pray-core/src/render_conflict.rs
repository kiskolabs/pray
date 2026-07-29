use crate::hashing::checksum_managed_body_line_refs;
use crate::lockfile::{Lockfile, ManagedSpanRecord};
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) fn reject_managed_span_conflicts(
    project_root: &Path,
    lockfile: &Lockfile,
) -> PrayResult<()> {
    let mut target_spans: BTreeMap<&str, Vec<&ManagedSpanRecord>> = BTreeMap::new();
    for span in &lockfile.managed_span {
        target_spans
            .entry(span.target.as_str())
            .or_default()
            .push(span);
    }
    for (target_path, spans) in target_spans {
        let absolute = project_root.join(target_path);
        if !absolute.exists() {
            continue;
        }
        let text = fs::read_to_string(&absolute)?;
        let lines: Vec<&str> = text.lines().collect();
        let markers = marker_checksums(&lines);
        for span in spans {
            let Some((_, _, checksum)) = markers.get(&span.id) else {
                continue;
            };
            if checksum != &span.ideal_checksum {
                return Err(PrayError::Render(format!(
                    "conflict: managed span `{}` in `{}` was edited (`{}::{}`)",
                    span.id, target_path, span.package, span.export
                )));
            }
        }
    }
    Ok(())
}

fn marker_checksums(lines: &[&str]) -> BTreeMap<String, (usize, usize, String)> {
    let mut markers = BTreeMap::new();
    let mut active: Option<(String, usize, Vec<&str>)> = None;
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let Some(remainder) = trimmed.strip_prefix("<!-- pray:") else {
            if let Some((_, _, body)) = active.as_mut() {
                body.push(line);
            }
            continue;
        };
        let Some(id) = remainder.strip_suffix(" -->") else {
            continue;
        };
        if id == "0 ignore-comments" {
            continue;
        }
        match active.take() {
            None => active = Some((id.to_string(), index + 1, Vec::new())),
            Some((open_id, open_line, body)) if open_id == id => {
                let checksum = checksum_managed_body_line_refs(&body);
                markers.insert(open_id, (open_line, index + 1, checksum));
            }
            Some(previous) => active = Some(previous),
        }
    }
    markers
}
