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
    materialize_provisioned_exports(project)?;
    Ok(())
}

pub fn materialize_provisioned_exports(project: &ResolvedProject) -> PrayResult<()> {
    for target in &project.manifest.targets {
        for folder_root in &target.skills {
            let destination_root = project.project_root.join(folder_root);
            for package in &project.packages {
                materialize_legacy_skill_trees(package, &destination_root)?;
                materialize_selected_exports(package, &destination_root)?;
            }
        }
    }
    Ok(())
}

fn materialize_legacy_skill_trees(
    package: &crate::resolve::ResolvedPackage,
    destination_root: &Path,
) -> PrayResult<()> {
    for (skill_name, skill) in &package.spec.skills {
        if legacy_skill_covered_by_export(package, skill) {
            continue;
        }
        let skill_files = package.skill_files.get(skill_name).ok_or_else(|| {
            PrayError::Render(format!(
                "package {} has no indexed files for legacy skill {}",
                package.declaration.name, skill_name
            ))
        })?;
        copy_tree(
            &package.root.join(&skill.path),
            &destination_root.join(skill_name),
            skill_files,
        )?;
    }
    Ok(())
}

fn legacy_skill_covered_by_export(
    package: &crate::resolve::ResolvedPackage,
    skill: &crate::package_spec::PackageSkill,
) -> bool {
    package.spec.exports.iter().any(|(export_name, export)| {
        package.selected_exports.contains(export_name)
            && is_folder_export_kind(&export.kind)
            && export.path.trim_end_matches('/') == skill.path.trim_end_matches('/')
    })
}

fn materialize_selected_exports(
    package: &crate::resolve::ResolvedPackage,
    destination_root: &Path,
) -> PrayResult<()> {
    for export_name in &package.selected_exports {
        let Some(export) = package.spec.exports.get(export_name) else {
            continue;
        };
        match export.kind.as_str() {
            "folder" | "skill" => {
                let indexed_files = package.skill_files.get(export_name).ok_or_else(|| {
                    PrayError::Render(format!(
                        "package {} has no indexed files for folder export {}",
                        package.declaration.name, export_name
                    ))
                })?;
                let destination_name = folder_destination_name(export_name, &export.path);
                copy_tree(
                    &package.root.join(&export.path),
                    &destination_root.join(destination_name),
                    indexed_files,
                )?;
            }
            "file" => {
                let source = package.root.join(&export.path);
                if !source.is_file() {
                    return Err(PrayError::Render(format!(
                        "file export source missing: {}",
                        source.display()
                    )));
                }
                let file_name = source
                    .file_name()
                    .map(|name| name.to_owned())
                    .ok_or_else(|| {
                        PrayError::Render(format!(
                            "file export path has no file name: {}",
                            export.path
                        ))
                    })?;
                let destination = destination_root.join(export_name).join(file_name);
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&source, &destination)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn is_folder_export_kind(kind: &str) -> bool {
    matches!(kind, "folder" | "skill")
}

fn folder_destination_name(export_name: &str, export_path: &str) -> String {
    Path::new(export_path.trim_end_matches('/'))
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| export_name.to_string())
}

fn copy_tree(source_root: &Path, destination_root: &Path, relative_files: &[String]) -> PrayResult<()> {
    if !source_root.is_dir() {
        return Err(PrayError::Render(format!(
            "folder source directory missing: {}",
            source_root.display()
        )));
    }

    if relative_files.is_empty() {
        return Err(PrayError::Render(format!(
            "no files listed in package manifest for {}",
            source_root.display()
        )));
    }

    for relative in relative_files {
        let source = source_root.join(relative);
        if !source.is_file() {
            return Err(PrayError::Render(format!(
                "provisioned file missing: {}",
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

fn should_inline_export(
    package: &crate::resolve::ResolvedPackage,
    export_name: &str,
    _target: &crate::manifest::ManifestTarget,
) -> bool {
    package
        .spec
        .exports
        .get(export_name)
        .is_none_or(|export| export.kind == "fragment")
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
            "Do not edit managed blocks in `{output_name}` or provisioned files under `.agents/`."
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
        builder.append_line(&format!("### {}", local.manifest_path));
        builder.append_body(&local.content);
        builder.append_empty_line();
    }

    builder.append_line("## Shared instructions");
    builder.append_empty_line();

    let mut managed_spans = Vec::new();
    for package in &project.packages {
        for export in &package.selected_exports {
            if !should_inline_export(package, export, target) {
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
