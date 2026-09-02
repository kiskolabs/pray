use crate::lockfile::Lockfile;
use crate::paths::validate_destination_path;
use crate::render::RenderedTarget;
use crate::render_file::{
    create_regular_bytes, destination_kind, open_regular, read_regular_bytes, symlink_error,
    DestinationKind,
};
use crate::render_path_guard::ensure_safe_destination_ancestors;
use crate::resolve::ResolvedProject;
use crate::{PrayError, PrayResult};
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::Path;

fn layout_rendered_content(path: &Path, display: &str, rendered: &str) -> PrayResult<String> {
    match destination_kind(path)? {
        DestinationKind::Missing => Ok(rendered.to_string()),
        DestinationKind::Regular => {
            let bytes = read_regular_bytes(path, display)?;
            let existing =
                String::from_utf8(bytes).map_err(|error| PrayError::Render(error.to_string()))?;
            Ok(crate::render_patch::patch_rendered_content(
                &existing, rendered,
            ))
        }
        DestinationKind::Symlink => Err(symlink_error(display)),
        DestinationKind::Other => Err(PrayError::Render(format!(
            "refusing to write `{display}`; destination is not a regular file"
        ))),
    }
}

fn write_rendered_content(path: &Path, display: &str, rendered: &str) -> PrayResult<()> {
    match destination_kind(path)? {
        DestinationKind::Missing => create_regular_bytes(path, display, rendered.as_bytes()),
        DestinationKind::Regular => {
            let mut file = open_regular(path, display, true)?;
            let mut existing = String::new();
            file.read_to_string(&mut existing)?;
            let content = crate::render_patch::patch_rendered_content(&existing, rendered);
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
    let mut laid_out = Vec::with_capacity(rendered.len());
    for target in rendered {
        let relative = validate_destination_path(&target.path.to_string_lossy())?;
        ensure_safe_destination_ancestors(
            &project.project_root,
            relative.as_path(),
            relative.as_str(),
        )?;
        let path = relative.join_root(&project.project_root);
        let content = layout_rendered_content(&path, relative.as_str(), &target.content)?;
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
    if project.manifest.render.conflict == "fail" {
        if let Some(lockfile) = previous_lockfile {
            crate::render_conflict::reject_managed_span_conflicts(&project.project_root, lockfile)?;
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
        write_rendered_content(&path, relative.as_str(), &target.content)?;
    }
    crate::render::materialize_provisioned_exports(project, previous_lockfile)?;
    Ok(())
}
