use crate::lockfile::Lockfile;
use crate::render::RenderedTarget;
use crate::render_file::{destination_kind, read_regular_bytes, symlink_error, DestinationKind};
use crate::{PrayError, PrayResult};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) fn layout_rendered_content(
    path: &Path,
    display: &str,
    rendered: &str,
    retired: &BTreeMap<String, String>,
) -> PrayResult<String> {
    match destination_kind(path)? {
        DestinationKind::Missing => Ok(rendered.to_string()),
        DestinationKind::Regular => {
            let bytes = read_regular_bytes(path, display)?;
            let existing =
                String::from_utf8(bytes).map_err(|error| PrayError::Render(error.to_string()))?;
            Ok(crate::render_patch::patch_rendered_content_with_retired(
                &existing, rendered, retired,
            ))
        }
        DestinationKind::Symlink => Err(symlink_error(display)),
        DestinationKind::Other => Err(PrayError::Render(format!(
            "refusing to write `{display}`; destination is not a regular file"
        ))),
    }
}

pub(crate) fn retired_span_checksums(
    previous_lockfile: Option<&Lockfile>,
    target: &RenderedTarget,
) -> BTreeMap<String, String> {
    let current_ids: BTreeSet<_> = target
        .managed_spans
        .iter()
        .map(|span| span.id.as_str())
        .collect();
    previous_lockfile
        .into_iter()
        .flat_map(|lockfile| &lockfile.managed_span)
        .filter(|span| {
            span.target == target.path.to_string_lossy() && !current_ids.contains(span.id.as_str())
        })
        .map(|span| (span.id.clone(), span.ideal_checksum.clone()))
        .collect()
}
