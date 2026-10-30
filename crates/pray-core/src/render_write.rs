use crate::lockfile::Lockfile;
use crate::paths::validate_destination_path;
use crate::render::RenderedTarget;
use crate::render_file::{
    create_regular_bytes, destination_kind, open_regular, read_destination_bytes, symlink_error,
    DestinationKind,
};
use crate::render_path_guard::ensure_safe_destination_ancestors;
use crate::render_reconcile::{layout_rendered_content, retired_span_checksums};
use crate::resolve::ResolvedProject;
use crate::{PrayError, PrayResult};
use std::fs;
use std::io::{Seek, Write};
use std::path::Path;

fn write_rendered_content(
    path: &Path,
    display: &str,
    rendered: &str,
    retired: &std::collections::BTreeMap<String, String>,
) -> PrayResult<()> {
    match destination_kind(path)? {
        DestinationKind::Missing => create_regular_bytes(path, display, rendered.as_bytes()),
        DestinationKind::Regular => {
            let mut file = open_regular(path, display, true)?;
            let existing = String::from_utf8(read_destination_bytes(&mut file, display)?)
                .map_err(|error| PrayError::Render(error.to_string()))?;
            let content = crate::render_patch::patch_rendered_content_with_retired(
                &existing, rendered, retired,
            );
            if crate::transaction::replace(
                path,
                Some(existing.as_bytes()),
                Some(content.as_bytes()),
            )? {
                return Ok(());
            }
            file.rewind()?;
            file.set_len(0)?;
            file.write_all(content.as_bytes())?;
            Ok(())
        }
        DestinationKind::Symlink => Err(symlink_error(display)),
        DestinationKind::Other => Err(PrayError::Render(format!(
            "refusing to write `{display}`; destination is not a regular file"
        ))),
    }
}

pub fn layout_rendered_targets(
    project: &ResolvedProject,
    rendered: &[RenderedTarget],
) -> PrayResult<Vec<RenderedTarget>> {
    layout_rendered_targets_with_previous_lockfile(project, rendered, None)
}

pub fn layout_rendered_targets_with_previous_lockfile(
    project: &ResolvedProject,
    rendered: &[RenderedTarget],
    previous_lockfile: Option<&Lockfile>,
) -> PrayResult<Vec<RenderedTarget>> {
    let mut laid_out = Vec::with_capacity(rendered.len());
    for target in rendered {
        let relative = validate_destination_path(&target.path.to_string_lossy())?;
        ensure_safe_destination_ancestors(
            &project.project_root,
            relative.as_path(),
            relative.as_str(),
        )?;
        let path = relative.join_root(&project.project_root);
        let retired = retired_span_checksums(previous_lockfile, target);
        let content = layout_rendered_content(&path, relative.as_str(), &target.content, &retired)?;
        let managed_spans =
            crate::render_relocate::relocate_managed_spans(&content, &target.managed_spans);
        laid_out.push(RenderedTarget {
            path: target.path.clone(),
            content,
            managed_spans,
        });
    }
    Ok(laid_out)
}

pub fn write_rendered_targets(
    project: &ResolvedProject,
    rendered: &[RenderedTarget],
) -> PrayResult<()> {
    write_rendered_targets_with_previous_lockfile(project, rendered, None)
}

pub fn write_rendered_targets_with_previous_lockfile(
    project: &ResolvedProject,
    rendered: &[RenderedTarget],
    previous_lockfile: Option<&Lockfile>,
) -> PrayResult<()> {
    crate::transaction::run(&project.project_root, || {
        write_project_files(project, rendered, previous_lockfile)
    })
}

fn write_project_files(
    project: &ResolvedProject,
    rendered: &[RenderedTarget],
    previous_lockfile: Option<&Lockfile>,
) -> PrayResult<()> {
    crate::render_dest::provisioned_destination_statuses(project, previous_lockfile)?;
    if project.manifest.render.conflict == "fail" {
        if let Some(lockfile) = previous_lockfile {
            crate::render_conflict::reject_managed_span_conflicts(
                &project.project_root,
                lockfile,
                rendered,
            )?;
        }
    }
    for target in rendered {
        let relative = validate_destination_path(&target.path.to_string_lossy())?;
        ensure_safe_destination_ancestors(
            &project.project_root,
            relative.as_path(),
            relative.as_str(),
        )?;
        let path = relative.join_root(&project.project_root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        ensure_safe_destination_ancestors(
            &project.project_root,
            relative.as_path(),
            relative.as_str(),
        )?;
        let retired = retired_span_checksums(previous_lockfile, target);
        write_rendered_content(&path, relative.as_str(), &target.content, &retired)?;
    }
    crate::render::materialize_provisioned_exports(project, previous_lockfile)?;
    Ok(())
}
