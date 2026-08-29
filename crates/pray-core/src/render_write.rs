use crate::lockfile::Lockfile;
use crate::paths::validate_project_relative_path;
use crate::render::RenderedTarget;
use crate::resolve::ResolvedProject;
use crate::PrayResult;
use std::fs;

pub fn layout_rendered_targets(
    project: &ResolvedProject,
    rendered: &[RenderedTarget],
) -> PrayResult<Vec<RenderedTarget>> {
    let mut laid_out = Vec::with_capacity(rendered.len());
    for target in rendered {
        let relative = validate_project_relative_path(&target.path.to_string_lossy())?;
        let path = relative.join_root(&project.project_root);
        let content = if path.exists() {
            let existing = fs::read_to_string(&path)?;
            crate::render_patch::patch_rendered_content(&existing, &target.content)
        } else {
            target.content.clone()
        };
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
        let relative = validate_project_relative_path(&target.path.to_string_lossy())?;
        let path = relative.join_root(&project.project_root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = if path.exists() {
            let existing = fs::read_to_string(&path)?;
            crate::render_patch::patch_rendered_content(&existing, &target.content)
        } else {
            target.content.clone()
        };
        fs::write(path, content)?;
    }
    crate::render::materialize_provisioned_exports(project)?;
    Ok(())
}
