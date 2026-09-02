use crate::compose_dest::{compose_header_text, ensure_html_comment_compose_dest};
use crate::destination::package_bound_to_compose;
use crate::environment::package_matches_environment;
use crate::hashing::{checksum_managed_span_content, marker_id};
use crate::lockfile::ManagedSpanRecord;
use crate::manifest::{DestinationEntry, DestinationMode};
use crate::render::RenderedTarget;
use crate::resolve::ResolvedProject;
use crate::substitute::substitute_pray_symbols;
use crate::{PrayError, PrayResult};
use std::path::Path;

struct ContentBuilder {
    content: String,
    next_line: usize,
}

impl ContentBuilder {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            content: String::with_capacity(capacity),
            next_line: 1,
        }
    }

    fn next_line_number(&self) -> usize {
        self.next_line
    }

    fn append_line(&mut self, line: &str) {
        self.content.push_str(line);
        self.content.push('\n');
        self.next_line += 1;
    }

    fn append_empty_line(&mut self) {
        self.content.push('\n');
        self.next_line += 1;
    }

    fn append_body(&mut self, body: &str) {
        let trimmed = body.trim_end_matches('\n');
        if trimmed.is_empty() {
            return;
        }
        for line in trimmed.split('\n') {
            self.append_line(line);
        }
    }

    fn finish(mut self) -> String {
        while self.content.ends_with("\n\n") {
            self.content.pop();
        }
        if !self.content.ends_with('\n') {
            self.content.push('\n');
        }
        self.content
    }
}

fn should_inline_export(package: &crate::resolve::ResolvedPackage, export_name: &str) -> bool {
    package
        .spec
        .exports
        .get(export_name)
        .is_none_or(|export| matches!(export.kind.as_str(), "fragment" | "file"))
}

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
                if local.content.is_empty() && local.optional {
                    continue;
                }
                let content = substitute_pray_symbols(&local.content, &project.manifest.symbols)?;
                builder.append_body(&content);
                builder.append_empty_line();
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
    for local in unbound_locals {
        if local.content.is_empty() && local.optional {
            continue;
        }
        builder.append_line(&format!("### {}", local.manifest_path));
        let content = substitute_pray_symbols(&local.content, &project.manifest.symbols)?;
        builder.append_body(&content);
        builder.append_empty_line();
    }

    builder.append_line("## Shared instructions");
    builder.append_empty_line();

    let mut managed_spans = Vec::new();
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

fn append_managed_export(
    builder: &mut ContentBuilder,
    managed_spans: &mut Vec<ManagedSpanRecord>,
    package: &crate::resolve::ResolvedPackage,
    export: &str,
    target: &crate::manifest::ManifestTarget,
    output: &Path,
    symbols: &std::collections::BTreeMap<String, String>,
) -> PrayResult<()> {
    let raw = match package.export_bodies.get(export) {
        Some(text) => text,
        None if package
            .spec
            .exports
            .get(export)
            .is_some_and(|entry| entry.kind == "file") =>
        {
            return Err(PrayError::Integrity(format!(
                "compose cannot write binary export {export}; use file: for unmarked bytes"
            )));
        }
        None => {
            return Err(PrayError::Render(format!(
                "package {} is missing cached export {}",
                package.declaration.name, export
            )));
        }
    };
    let body = substitute_pray_symbols(raw, symbols)?;
    let id = marker_id(&format!(
        "{}:{}:{}",
        package.declaration.name, export, target.name
    ));
    let open_line = builder.next_line_number();
    builder.append_line(&format!("<!-- pray:{id} -->"));
    builder.append_body(&body);
    let close_line = builder.next_line_number();
    builder.append_line(&format!("<!-- pray:{id} -->"));
    managed_spans.push(ManagedSpanRecord {
        id,
        target: output.to_string_lossy().to_string(),
        open_line,
        close_line,
        ideal_checksum: checksum_managed_span_content(&body),
        package: package.declaration.name.clone(),
        export: export.to_string(),
        source_checksum: package.source_checksum.clone(),
        silenced: false,
    });
    builder.append_empty_line();
    Ok(())
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
