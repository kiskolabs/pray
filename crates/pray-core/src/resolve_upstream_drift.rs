use super::{source_map, ResolvedProject};
use crate::config::load_user_config;
use crate::lockfile::Lockfile;
use crate::package_upstream::{overlay_drift_line, overlay_file_changes};
use crate::resolve_context::ResolveOptions;
use crate::resolve_git_sources::prepare_git_sources;
use crate::PrayResult;

use super::input_output::{content_file_bytes, local_content_for_refresh};
use super::resolution_context::UpstreamResolutionContext;

pub fn path_fork_drift_lines(
    project: &ResolvedProject,
    previous: Option<&Lockfile>,
    options: &ResolveOptions,
) -> PrayResult<Vec<String>> {
    let user_config = load_user_config()?;
    let git_sources = prepare_git_sources(
        &project.project_root,
        &project.manifest.sources,
        previous,
        options,
    )?;
    let sources = source_map(&project.manifest.sources);
    let mut lines = Vec::new();
    for package in &project.packages {
        if package.declaration.path.is_none() {
            continue;
        }
        let Some(upstream) = &package.upstream else {
            continue;
        };
        let resolution_context = UpstreamResolutionContext::new(
            &project.project_root,
            &sources,
            &git_sources,
            &user_config,
            previous,
            options,
        );
        let resolved = resolution_context.resolve(
            &upstream.name,
            &format!("= {}", upstream.version),
            upstream.source.as_deref(),
        )?;
        let upstream_content = content_file_bytes(&resolved.root, &resolved.spec)?;
        let local_content = local_content_for_refresh(&package.root, &package.spec)?;
        if local_content.is_empty() {
            lines.push(format!(
                "{} has no content files; run pray install to copy {} {}",
                package.declaration.name, upstream.name, upstream.version
            ));
            continue;
        }
        for (path, change) in overlay_file_changes(&local_content, &upstream_content) {
            lines.push(overlay_drift_line(
                &package.declaration.name,
                &upstream.name,
                &upstream.version,
                &path,
                &change,
            ));
        }
    }
    Ok(lines)
}
