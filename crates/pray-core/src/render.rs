use crate::hashing::{marker_id, normalize_line_endings, sha256_prefixed};
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
                    copy_skill_tree(
                        &package.root.join(&skill.path),
                        &destination_root.join(skill_name),
                        &package.spec.files,
                        &skill.path,
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
    package_files: &[String],
    skill_path: &str,
) -> PrayResult<()> {
    if !source_root.is_dir() {
        return Err(PrayError::Render(format!(
            "skill source directory missing: {}",
            source_root.display()
        )));
    }

    let skill_prefix = skill_path.trim_end_matches('/');
    let mut copied = false;
    for file in package_files {
        let relative = file.strip_prefix(skill_prefix).and_then(|rest| {
            let trimmed = rest.trim_start_matches('/');
            if trimmed.is_empty() || file == skill_prefix {
                None
            } else {
                Some(trimmed)
            }
        });
        let Some(relative) = relative else {
            continue;
        };

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
        copied = true;
    }

    if !copied {
        return Err(PrayError::Render(format!(
            "no skill files listed in package manifest for {}",
            source_root.display()
        )));
    }

    Ok(())
}

fn render_target(
    project: &ResolvedProject,
    target: &crate::manifest::ManifestTarget,
    output: &Path,
) -> PrayResult<RenderedTarget> {
    let mut lines = Vec::<String>::new();
    if project.manifest.render.header {
        let output_name = output
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| output.to_string_lossy().to_string());
        lines.push("<!-- pray:0 ignore-comments -->".to_string());
        lines.push(String::new());
        lines.push("# Agent context".to_string());
        lines.push(String::new());
        lines.push(format!(
            "Do not edit managed blocks in `{output_name}` or skills under `.agents/`."
        ));
        lines.push(
            "To change shared guidance, update `Prayfile` and run `pray install`.".to_string(),
        );
        lines.push(String::new());
    }

    if !project.local_files.is_empty() {
        lines.push("## Additional instructions".to_string());
        lines.push(String::new());
    }
    for local in &project.local_files {
        if local.content.is_empty() && local.optional {
            continue;
        }
        lines.push(format!("### {}", local.path.display()));
        push_body(&mut lines, &local.content);
        lines.push(String::new());
    }

    lines.push("## Shared instructions".to_string());
    lines.push(String::new());

    let mut managed_spans = Vec::new();
    for package in &project.packages {
        for export in &package.selected_exports {
            let entry = package.spec.exports.get(export).ok_or_else(|| {
                PrayError::Render(format!(
                    "package {} is missing export {}",
                    package.declaration.name, export
                ))
            })?;
            let body = read_export_body(&package.root.join(&entry.path))?;
            let id = marker_id(&format!(
                "{}:{}:{}",
                package.declaration.name, export, target.name
            ));
            let normalized_body = normalize_line_endings(&body);
            let body_for_checksum = normalized_body.trim_end_matches('\n').to_string();
            let open_line = lines.len() + 1;
            lines.push(format!("<!-- pray:{} -->", id));
            push_body(&mut lines, &body);
            let close_line = lines.len() + 1;
            lines.push(format!("<!-- pray:{} -->", id));
            managed_spans.push(ManagedSpanRecord {
                id,
                target: output.to_string_lossy().to_string(),
                open_line,
                close_line,
                ideal_checksum: sha256_prefixed(body_for_checksum.as_bytes()),
                package: package.declaration.name.clone(),
                export: export.clone(),
                source_checksum: package.source_checksum.clone(),
                silenced: false,
            });
            lines.push(String::new());
        }
    }

    if lines.last().map(|line| line.is_empty()).unwrap_or(false) {
        lines.pop();
    }

    let mut content = lines.join("\n");
    if !content.ends_with('\n') {
        content.push('\n');
    }
    Ok(RenderedTarget {
        path: output.to_path_buf(),
        content,
        managed_spans,
    })
}

fn push_body(lines: &mut Vec<String>, body: &str) {
    let normalized = normalize_line_endings(body);
    let trimmed = normalized.trim_end_matches('\n');
    if trimmed.is_empty() {
        return;
    }
    for line in trimmed.split('\n') {
        lines.push(line.to_string());
    }
}

fn read_export_body(path: &Path) -> PrayResult<String> {
    let text = fs::read_to_string(path)?;
    Ok(normalize_line_endings(&text))
}
