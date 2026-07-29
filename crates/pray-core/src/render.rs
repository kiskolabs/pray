use crate::paths::validate_project_relative_path;
use crate::resolve::ResolvedProject;
use crate::{PrayError, PrayResult};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RenderedTarget {
    pub path: PathBuf,
    pub content: String,
    pub managed_spans: Vec<crate::lockfile::ManagedSpanRecord>,
}

pub fn render_project(project: &ResolvedProject) -> PrayResult<Vec<RenderedTarget>> {
    let mut rendered = Vec::new();
    for target in &project.manifest.targets {
        for output in &target.outputs {
            let relative = validate_project_relative_path(output)?;
            let rendered_target =
                crate::render_compose::render_target(project, target, relative.as_path())?;
            if let Some(max_bytes) = target.max_bytes {
                let size = rendered_target.content.len() as u64;
                if size > max_bytes {
                    return Err(PrayError::Render(format!(
                        "target `{}` output `{}` is {size} bytes; max_bytes is {max_bytes}",
                        target.name, output
                    )));
                }
            }
            rendered.push(rendered_target);
        }
    }
    Ok(rendered)
}

pub use crate::render_write::{
    write_rendered_targets, write_rendered_targets_with_previous_lockfile,
};

pub use crate::render_provisioned::{
    expected_provisioned_bytes, materialize_provisioned_exports, planned_provisioned_files,
    PlannedProvisionedFile,
};
