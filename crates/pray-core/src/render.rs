use crate::destination::{package_bound_to_compose, package_bound_to_tree};
use crate::environment::package_matches_environment;
use crate::hashing::{checksum_managed_span_content, marker_id};
use crate::lockfile::ManagedSpanRecord;
use crate::manifest::{DestinationEntry, DestinationMode};
use crate::resolve::ResolvedProject;
use crate::substitute::substitute_pray_symbols;
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
        let Some(output) = target.outputs.first() else {
            continue;
        };
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

#[derive(Debug, Clone)]
pub struct PlannedProvisionedFile {
    pub path: PathBuf,
    pub source: PathBuf,
}

pub fn planned_provisioned_files(
    project: &ResolvedProject,
) -> PrayResult<Vec<PlannedProvisionedFile>> {
    let mut planned = Vec::new();
    collect_exact_file_bindings(project, &mut planned)?;
    for target in &project.manifest.targets {
        for folder_root in &target.skills {
            let destination_root = project.project_root.join(folder_root);
            for package in &project.packages {
                if !package_matches_environment(
                    &package.declaration.groups,
                    project.environment.as_deref(),
                ) {
                    continue;
                }
                if !package_bound_to_tree(&package.declaration, target) {
                    continue;
                }
                collect_legacy_skill_files(project, package, &destination_root, &mut planned)?;
                collect_selected_export_files(project, package, &destination_root, &mut planned)?;
            }
        }
    }
    planned.sort_by(|left, right| left.path.cmp(&right.path));
    planned.dedup_by(|left, right| left.path == right.path);
    Ok(planned)
}

pub fn materialize_provisioned_exports(project: &ResolvedProject) -> PrayResult<()> {
    for file in planned_provisioned_files(project)? {
        let destination = project.project_root.join(&file.path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        write_provisioned_file(&file.source, &destination, &project.manifest.symbols)?;
    }
    Ok(())
}

pub fn expected_provisioned_bytes(
    source: &Path,
    symbols: &std::collections::BTreeMap<String, String>,
) -> PrayResult<Vec<u8>> {
    let bytes = fs::read(source)?;
    match String::from_utf8(bytes) {
        Ok(text) => Ok(substitute_pray_symbols(&text, symbols)?.into_bytes()),
        Err(error) => Ok(error.into_bytes()),
    }
}

fn write_provisioned_file(
    source: &Path,
    destination: &Path,
    symbols: &std::collections::BTreeMap<String, String>,
) -> PrayResult<()> {
    fs::write(destination, expected_provisioned_bytes(source, symbols)?)?;
    Ok(())
}

fn collect_exact_file_bindings(
    project: &ResolvedProject,
    planned: &mut Vec<PlannedProvisionedFile>,
) -> PrayResult<()> {
    for package in &project.packages {
        let Some(destination) = &package.declaration.file else {
            continue;
        };
        if !package_matches_environment(&package.declaration.groups, project.environment.as_deref())
        {
            continue;
        }
        let mut matched = false;
        for export_name in &package.selected_exports {
            let Some(export) = package.spec.exports.get(export_name) else {
                continue;
            };
            if export.kind != "file" {
                continue;
            }
            let source = package.root.join(&export.path);
            if !source.is_file() {
                return Err(PrayError::Render(format!(
                    "file export source missing: {}",
                    source.display()
                )));
            }
            planned.push(PlannedProvisionedFile {
                path: PathBuf::from(destination),
                source,
            });
            matched = true;
            break;
        }
        if !matched {
            return Err(PrayError::Render(format!(
                "package {} has file: \"{}\" but no selected file export",
                package.declaration.name, destination
            )));
        }
    }
    Ok(())
}

fn relative_project_path(project: &ResolvedProject, absolute: &Path) -> PathBuf {
    absolute
        .strip_prefix(&project.project_root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| absolute.to_path_buf())
}

fn collect_legacy_skill_files(
    project: &ResolvedProject,
    package: &crate::resolve::ResolvedPackage,
    destination_root: &Path,
    planned: &mut Vec<PlannedProvisionedFile>,
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
        collect_tree_files(
            project,
            &package.root.join(&skill.path),
            &destination_root.join(skill_name),
            skill_files,
            &[],
            &[],
            planned,
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

fn collect_selected_export_files(
    project: &ResolvedProject,
    package: &crate::resolve::ResolvedPackage,
    destination_root: &Path,
    planned: &mut Vec<PlannedProvisionedFile>,
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
                collect_tree_files(
                    project,
                    &package.root.join(&export.path),
                    &destination_root.join(destination_name),
                    indexed_files,
                    &export.only,
                    &export.except,
                    planned,
                )?;
            }
            "file" => {
                if package.declaration.file.is_some() {
                    continue;
                }
                let source = package.root.join(&export.path);
                if !source.is_file() {
                    return Err(PrayError::Render(format!(
                        "file export source missing: {}",
                        source.display()
                    )));
                }
                let file_name =
                    source
                        .file_name()
                        .map(|name| name.to_owned())
                        .ok_or_else(|| {
                            PrayError::Render(format!(
                                "file export path has no file name: {}",
                                export.path
                            ))
                        })?;
                let destination = destination_root.join(export_name).join(file_name);
                planned.push(PlannedProvisionedFile {
                    path: relative_project_path(project, &destination),
                    source,
                });
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

fn collect_tree_files(
    project: &ResolvedProject,
    source_root: &Path,
    destination_root: &Path,
    relative_files: &[String],
    only: &[String],
    except: &[String],
    planned: &mut Vec<PlannedProvisionedFile>,
) -> PrayResult<()> {
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

    let mut matched = false;
    for relative in relative_files {
        if !only.is_empty() && !only.iter().any(|entry| entry == relative) {
            continue;
        }
        if except.iter().any(|entry| entry == relative) {
            continue;
        }
        let source = source_root.join(relative);
        if !source.is_file() {
            return Err(PrayError::Render(format!(
                "provisioned file missing: {}",
                source.display()
            )));
        }
        let destination = destination_root.join(relative);
        planned.push(PlannedProvisionedFile {
            path: relative_project_path(project, &destination),
            source,
        });
        matched = true;
    }

    if !matched && only.is_empty() && except.is_empty() {
        return Err(PrayError::Render(format!(
            "no files listed in package manifest for {}",
            source_root.display()
        )));
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
        .is_none_or(|export| export.kind == "fragment")
}

fn render_target(
    project: &ResolvedProject,
    target: &crate::manifest::ManifestTarget,
    output: &Path,
) -> PrayResult<RenderedTarget> {
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
    let raw = package.export_bodies.get(export).ok_or_else(|| {
        PrayError::Render(format!(
            "package {} is missing cached export {}",
            package.declaration.name, export
        ))
    })?;
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
