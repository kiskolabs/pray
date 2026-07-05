use crate::hashing::{checksum_managed_span_content, marker_id};
use crate::lockfile::ManagedSpanRecord;
use crate::resolve::ResolvedProject;
use crate::{PrayError, PrayResult};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RenderedTarget {
    pub path: PathBuf,
    pub content: String,
    pub managed_spans: Vec<ManagedSpanRecord>,
}

pub fn render_project(project: &ResolvedProject) -> PrayResult<Vec<RenderedTarget>> {
    let mut rendered = Vec::new();
    for target in &project.manifest.targets {
        let output = target.outputs.first().ok_or_else(|| {
            PrayError::Render(format!("target {} has no output file", target.name))
        })?;
        rendered.push(render_target(project, target, Path::new(output))?);
    }
    Ok(rendered)
}

pub fn write_rendered_targets(
    project: &ResolvedProject,
    rendered: &[RenderedTarget],
) -> PrayResult<()> {
    for target in rendered {
        let path = project.project_root.join(&target.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &target.content)?;
    }
    materialize_target_skills(project)?;
    Ok(())
}

pub fn materialize_target_skills(project: &ResolvedProject) -> PrayResult<()> {
    for target in &project.manifest.targets {
        for skills_root in &target.skills {
            let destination_root = project.project_root.join(skills_root);
            for package in &project.packages {
                for (skill_name, skill) in &package.spec.skills {
                    let skill_files = package.skill_files.get(skill_name).ok_or_else(|| {
                        PrayError::Render(format!(
                            "package {} has no indexed files for skill {}",
                            package.declaration.name, skill_name
                        ))
                    })?;
                    copy_skill_tree(
                        &package.root.join(&skill.path),
                        &destination_root.join(skill_name),
                        skill_files,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn copy_skill_tree(
    source_root: &Path,
    destination_root: &Path,
    skill_files: &[String],
) -> PrayResult<()> {
    if !source_root.is_dir() {
        return Err(PrayError::Render(format!(
            "skill source directory missing: {}",
            source_root.display()
        )));
    }

    if skill_files.is_empty() {
        return Err(PrayError::Render(format!(
            "no skill files listed in package manifest for {}",
            source_root.display()
        )));
    }

    for relative in skill_files {
        let source = source_root.join(relative);
        if !source.is_file() {
            return Err(PrayError::Render(format!(
                "skill file missing: {}",
                source.display()
            )));
        }
        let destination = destination_root.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&source, &destination)?;
    }

    Ok(())
}

struct ContentBuilder {
    content: String,
}

impl ContentBuilder {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            content: String::with_capacity(capacity),
        }
    }

    fn next_line_number(&self) -> usize {
        self.content.matches('\n').count() + 1
    }

    fn append_line(&mut self, line: &str) {
        self.content.push_str(line);
        self.content.push('\n');
    }

    fn append_empty_line(&mut self) {
        self.content.push('\n');
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
        .is_none_or(|export| export.kind != "skill")
}

fn render_target(
    project: &ResolvedProject,
    target: &crate::manifest::ManifestTarget,
    output: &Path,
) -> PrayResult<RenderedTarget> {
    let mut builder = ContentBuilder::with_capacity(8_192);
    if project.manifest.render.header {
        let output_name = output
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| output.to_string_lossy().to_string());
        builder.append_line("<!-- pray:0 ignore-comments -->");
        builder.append_empty_line();
        builder.append_line("# Agent context");
        builder.append_empty_line();
        builder.append_line(&format!(
            "Do not edit managed blocks in `{output_name}` or skills under `.agents/`."
        ));
        builder.append_line("To change shared guidance, update `Prayfile` and run `pray install`.");
        builder.append_empty_line();
    }

    if !project.local_files.is_empty() {
        builder.append_line("## Additional instructions");
        builder.append_empty_line();
    }
    for local in &project.local_files {
        if local.content.is_empty() && local.optional {
            continue;
        }
        builder.append_line(&format!("### {}", local.path.display()));
        builder.append_body(&local.content);
        builder.append_empty_line();
    }

    builder.append_line("## Shared instructions");
    builder.append_empty_line();

    let mut managed_spans = Vec::new();
    for package in &project.packages {
        for export in &package.selected_exports {
            if !should_inline_export(package, export) {
                continue;
            }
            let body = package.export_bodies.get(export).ok_or_else(|| {
                PrayError::Render(format!(
                    "package {} is missing cached export {}",
                    package.declaration.name, export
                ))
            })?;
            let id = marker_id(&format!(
                "{}:{}:{}",
                package.declaration.name, export, target.name
            ));
            let open_line = builder.next_line_number();
            builder.append_line(&format!("<!-- pray:{id} -->"));
            builder.append_body(body);
            let close_line = builder.next_line_number();
            builder.append_line(&format!("<!-- pray:{id} -->"));
            managed_spans.push(ManagedSpanRecord {
                id,
                target: output.to_string_lossy().to_string(),
                open_line,
                close_line,
                ideal_checksum: checksum_managed_span_content(body),
                package: package.declaration.name.clone(),
                export: export.clone(),
                source_checksum: package.source_checksum.clone(),
                silenced: false,
            });
            builder.append_empty_line();
        }
    }

    Ok(RenderedTarget {
        path: output.to_path_buf(),
        content: builder.finish(),
        managed_spans,
    })
}
