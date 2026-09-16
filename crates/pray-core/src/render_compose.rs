use crate::compose_dest::{compose_header_text, ensure_html_comment_compose_dest};
use crate::destination::package_bound_to_compose;
use crate::environment::package_matches_environment;
use crate::manifest::{DestinationEntry, DestinationMode};
use crate::render::RenderedTarget;
use crate::render_span::{
    append_managed_export, append_managed_local, should_inline_export, ContentBuilder,
};
use crate::resolve::ResolvedProject;
use crate::PrayResult;
use std::path::Path;

pub(crate) fn render_target(
    project: &ResolvedProject,
    target: &crate::manifest::ManifestTarget,
    output: &Path,
) -> PrayResult<RenderedTarget> {
    ensure_html_comment_compose_dest(output)?;
    if target.scoped && target.mode == DestinationMode::Compose {
        return render_scoped_compose(project, target, output);
    }
    render_legacy_compose(project, target, output)
}

fn render_scoped_compose(
    project: &ResolvedProject,
    target: &crate::manifest::ManifestTarget,
    output: &Path,
) -> PrayResult<RenderedTarget> {
    let mut builder = ContentBuilder::with_capacity(8_192);
    append_compose_header(&mut builder, project, target, output);

    let mut managed_spans = Vec::new();
    for entry in &target.entries {
        match entry {
            DestinationEntry::Local { path } => {
                let Some(local) = project
                    .local_files
                    .iter()
                    .find(|local| local.manifest_path == *path)
                else {
                    continue;
                };
                append_managed_local(
                    &mut builder,
                    &mut managed_spans,
                    local,
                    target,
                    output,
                    &project.manifest.symbols,
                )?;
            }
            DestinationEntry::Package { name } => {
                let Some(package) = project
                    .packages
                    .iter()
                    .find(|package| package.declaration.name == *name)
                else {
                    continue;
                };
                if !package_matches_environment(
                    &package.declaration.groups,
                    project.environment.as_deref(),
                ) {
                    continue;
                }
                for export in &package.selected_exports {
                    if !should_inline_export(package, export) {
                        continue;
                    }
                    append_managed_export(
                        &mut builder,
                        &mut managed_spans,
                        package,
                        export,
                        target,
                        output,
                        &project.manifest.symbols,
                    )?;
                }
            }
        }
    }

    Ok(RenderedTarget {
        path: output.to_path_buf(),
        content: builder.finish(),
        managed_spans,
    })
}

fn render_legacy_compose(
    project: &ResolvedProject,
    target: &crate::manifest::ManifestTarget,
    output: &Path,
) -> PrayResult<RenderedTarget> {
    let mut builder = ContentBuilder::with_capacity(8_192);
    append_compose_header(&mut builder, project, target, output);

    let unbound_locals: Vec<_> = project
        .local_files
        .iter()
        .filter(|local| {
            project
                .manifest
                .local
                .iter()
                .find(|entry| entry.path == local.manifest_path)
                .is_none_or(|entry| !entry.bound)
        })
        .collect();

    if !unbound_locals.is_empty() {
        builder.append_line("## Additional instructions");
        builder.append_empty_line();
    }
    let mut managed_spans = Vec::new();
    for local in unbound_locals {
        if local.content.is_empty() && local.optional {
            continue;
        }
        builder.append_line(&format!("### {}", local.manifest_path));
        append_managed_local(
            &mut builder,
            &mut managed_spans,
            local,
            target,
            output,
            &project.manifest.symbols,
        )?;
    }

    builder.append_line("## Shared instructions");
    builder.append_empty_line();

    for package in &project.packages {
        if !package.explicit {
            continue;
        }
        if !package_matches_environment(&package.declaration.groups, project.environment.as_deref())
        {
            continue;
        }
        if !package_bound_to_compose(&package.declaration, target) {
            continue;
        }
        for export in &package.selected_exports {
            if !should_inline_export(package, export) {
                continue;
            }
            append_managed_export(
                &mut builder,
                &mut managed_spans,
                package,
                export,
                target,
                output,
                &project.manifest.symbols,
            )?;
        }
    }

    Ok(RenderedTarget {
        path: output.to_path_buf(),
        content: builder.finish(),
        managed_spans,
    })
}

fn append_compose_header(
    builder: &mut ContentBuilder,
    project: &ResolvedProject,
    target: &crate::manifest::ManifestTarget,
    output: &Path,
) {
    if let Some(header) = compose_header_text(target, output, project.manifest.render.header) {
        builder.append_body(&header);
        builder.append_empty_line();
    }
}
