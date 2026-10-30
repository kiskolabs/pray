use crate::hashing::checksum_managed_body_line_refs;
use crate::lockfile::{Lockfile, ManagedSpanRecord};
use crate::render::RenderedTarget;
use crate::{PrayError, PrayResult};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) fn reject_managed_span_conflicts(
    project_root: &Path,
    lockfile: &Lockfile,
    rendered: &[RenderedTarget],
) -> PrayResult<()> {
    let active_by_target: BTreeMap<&str, BTreeSet<&str>> = rendered
        .iter()
        .map(|target| {
            (
                target.path.to_str().unwrap_or_default(),
                target
                    .managed_spans
                    .iter()
                    .map(|span| span.id.as_str())
                    .collect(),
            )
        })
        .collect();
    let mut target_spans: BTreeMap<&str, Vec<&ManagedSpanRecord>> = BTreeMap::new();
    for span in &lockfile.managed_span {
        target_spans
            .entry(span.target.as_str())
            .or_default()
            .push(span);
    }
    for (target_path, spans) in target_spans {
        let Some(active_ids) = active_by_target.get(target_path) else {
            continue;
        };
        let relative = crate::paths::validate_destination_path(target_path)?;
        crate::render_path_guard::ensure_safe_destination_ancestors(
            project_root,
            relative.as_path(),
            relative.as_str(),
        )?;
        let absolute = relative.join_root(project_root);
        if !absolute.exists() {
            continue;
        }
        let text = crate::render_file::read_destination_text(&absolute)?;
        let lines: Vec<&str> = text.lines().collect();
        let markers = marker_checksums(&lines);
        for span in spans {
            if !active_ids.contains(span.id.as_str()) {
                continue;
            }
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
