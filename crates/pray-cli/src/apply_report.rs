use pray_core::lockfile::{lockfiles_equivalent, Lockfile};
use pray_core::manifest::ManifestPackage;
use pray_core::render::{planned_provisioned_files, RenderedTarget};
use pray_core::resolve::ResolvedProject;
use pray_core::verify::{inspect_project, VerificationFinding};
use pray_core::PrayResult;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterializationMode {
    Plan,
    Install,
    Apply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockfileChange {
    Unchanged,
    Updated,
    Created,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetChange {
    Unchanged,
    Updated,
    Written,
}

#[derive(Debug, Clone)]
pub struct MaterializationPreview {
    pub package_lines: Vec<String>,
    pub lockfile: LockfileChange,
    pub targets: Vec<(PathBuf, TargetChange)>,
    pub provisioned: Vec<(PathBuf, TargetChange)>,
    pub warnings: Vec<String>,
}

pub fn build_materialization_preview(
    project: &ResolvedProject,
    rendered: &[RenderedTarget],
    lockfile: &Lockfile,
    lockfile_path: &Path,
    previous_lockfile: Option<&Lockfile>,
) -> PrayResult<MaterializationPreview> {
    let package_lines = package_summary_lines(previous_lockfile, lockfile, project);
    let lockfile_change = lockfile_change_status(lockfile_path, lockfile)?;
    let targets = rendered
        .iter()
        .map(|target| {
            let path = project.project_root.join(&target.path);
            let change = target_change_status(&path, &target.content);
            Ok((target.path.clone(), change))
        })
        .collect::<PrayResult<Vec<_>>>()?;
    let provisioned = planned_provisioned_files(project)?
        .into_iter()
        .map(|file| {
            let change = provisioned_change_status(&project.project_root, &file.path, &file.source);
            (file.path, change)
        })
        .collect();
    let warnings = pre_apply_warnings(project, previous_lockfile, rendered, &targets)?;

    Ok(MaterializationPreview {
        package_lines,
        lockfile: lockfile_change,
        targets,
        provisioned,
        warnings,
    })
}

pub fn materialization_preview_to_json(
    preview: &MaterializationPreview,
    mode: MaterializationMode,
) -> serde_json::Value {
    serde_json::json!({
        "mode": mode.heading().to_lowercase(),
        "packages": preview.package_lines,
        "lockfile": lockfile_change_label(preview.lockfile),
        "targets": preview
            .targets
            .iter()
            .map(|(path, change)| {
                serde_json::json!({
                    "path": path,
                    "change": target_change_label(*change),
                })
            })
            .collect::<Vec<_>>(),
        "provisioned": preview
            .provisioned
            .iter()
            .map(|(path, change)| {
                serde_json::json!({
                    "path": path,
                    "change": target_change_label(*change),
                })
            })
            .collect::<Vec<_>>(),
        "warnings": preview.warnings,
    })
}

fn lockfile_change_label(change: LockfileChange) -> &'static str {
    match change {
        LockfileChange::Unchanged => "unchanged",
        LockfileChange::Updated => "updated",
        LockfileChange::Created => "created",
    }
}

fn target_change_label(change: TargetChange) -> &'static str {
    match change {
        TargetChange::Unchanged => "unchanged",
        TargetChange::Updated => "updated",
        TargetChange::Written => "written",
    }
}

pub fn print_materialization_report(preview: &MaterializationPreview, mode: MaterializationMode) {
    println!("{}...", mode.heading());

    for line in &preview.package_lines {
        println!("{line}");
    }

    println!("{}", lockfile_line(preview.lockfile, mode));

    for (path, change) in &preview.targets {
        println!("{} {}", path.display(), target_verb(*change, mode));
    }

    for line in grouped_provisioned_lines(&preview.provisioned, mode) {
        println!("{line}");
    }

    if !preview.warnings.is_empty() {
        println!();
        println!("Warnings");
        for warning in &preview.warnings {
            println!("  {warning}");
        }
    }

    println!();
    println!("{}", summary_footer(preview, mode));
}

impl MaterializationMode {
    fn heading(self) -> &'static str {
        match self {
            Self::Plan => "Plan",
            Self::Install => "Installing",
            Self::Apply => "Applying",
        }
    }

    fn dry_run(self) -> bool {
        matches!(self, Self::Plan)
    }

    fn completion_label(self) -> &'static str {
        match self {
            Self::Plan => "Plan complete",
            Self::Install => "Install complete",
            Self::Apply => "Apply complete",
        }
    }
}

fn package_summary_lines(
    previous: Option<&Lockfile>,
    updated: &Lockfile,
    project: &ResolvedProject,
) -> Vec<String> {
    let previous_versions: std::collections::BTreeMap<&str, &str> = previous
        .into_iter()
        .flat_map(|lockfile| lockfile.package.iter())
        .map(|package| (package.name.as_str(), package.version.as_str()))
        .collect();
    let sources: std::collections::BTreeMap<&str, String> = project
        .packages
        .iter()
        .map(|package| {
            (
                package.declaration.name.as_str(),
                package_source_label(&package.declaration),
            )
        })
        .collect();

    let mut lines = Vec::new();
    for package in &updated.package {
        let source = sources
            .get(package.name.as_str())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        match previous_versions.get(package.name.as_str()) {
            None => lines.push(format!(
                "Installing {} {} ({source})",
                package.name, package.version
            )),
            Some(previous_version) if *previous_version == package.version => lines.push(format!(
                "Using {} {} ({source})",
                package.name, package.version
            )),
            Some(previous_version) => lines.push(format!(
                "Using {} {} (was {previous_version}) ({source})",
                package.name, package.version
            )),
        }
    }
    lines
}

fn lockfile_change_status(path: &Path, lockfile: &Lockfile) -> PrayResult<LockfileChange> {
    if !path.exists() {
        return Ok(LockfileChange::Created);
    }
    match pray_core::lockfile::read_lockfile(path) {
        Ok(existing) if lockfiles_equivalent(lockfile, &existing) => Ok(LockfileChange::Unchanged),
        Ok(_) | Err(_) => Ok(LockfileChange::Updated),
    }
}

fn target_change_status(path: &Path, content: &str) -> TargetChange {
    match fs::read_to_string(path) {
        Ok(existing) if existing == content => TargetChange::Unchanged,
        Ok(_) => TargetChange::Updated,
        Err(_) => TargetChange::Written,
    }
}

fn provisioned_change_status(
    project_root: &Path,
    relative_path: &Path,
    source: &Path,
) -> TargetChange {
    let destination = project_root.join(relative_path);
    let expected = fs::read(source).ok();
    match fs::read(&destination) {
        Ok(existing) if expected.as_ref() == Some(&existing) => TargetChange::Unchanged,
        Ok(_) => TargetChange::Updated,
        Err(_) => TargetChange::Written,
    }
}

fn grouped_provisioned_lines(
    provisioned: &[(PathBuf, TargetChange)],
    mode: MaterializationMode,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut index = 0;
    while index < provisioned.len() {
        let (path, change) = &provisioned[index];
        if *change == TargetChange::Unchanged {
            index += 1;
            continue;
        }

        let parent = path
            .parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| path.clone());
        let mut grouped = vec![(path.clone(), *change)];
        let mut next = index + 1;
        while next < provisioned.len() {
            let (next_path, next_change) = &provisioned[next];
            if next_path.parent() != Some(parent.as_path())
                || *next_change == TargetChange::Unchanged
            {
                break;
            }
            grouped.push((next_path.clone(), *next_change));
            next += 1;
        }

        if grouped.len() == 1 {
            lines.push(format!(
                "{} {}",
                grouped[0].0.display(),
                target_verb(grouped[0].1, mode)
            ));
        } else {
            let verb = target_verb(grouped[0].1, mode);
            let folder = if parent.as_os_str().is_empty() {
                ".".to_string()
            } else {
                format!("{}/", parent.display())
            };
            lines.push(format!("{folder} {verb} ({} files)", grouped.len()));
        }
        index = next;
    }
    lines
}

fn pre_apply_warnings(
    project: &ResolvedProject,
    previous_lockfile: Option<&Lockfile>,
    rendered: &[RenderedTarget],
    targets: &[(PathBuf, TargetChange)],
) -> PrayResult<Vec<String>> {
    let mut warnings = Vec::new();
    let Some(previous_lockfile) = previous_lockfile else {
        return Ok(warnings);
    };

    let report = inspect_project(project, previous_lockfile)?;
    for finding in report.findings {
        if warning_applies_to_materialization(&finding, rendered, targets) {
            warnings.push(warning_message(&finding));
        }
    }

    for target in rendered {
        let path = project.project_root.join(&target.path);
        let change = target_change_status(&path, &target.content);
        if change == TargetChange::Unchanged {
            continue;
        }
        if let Ok(existing) = fs::read_to_string(&path) {
            if existing != target.content
                && !warnings
                    .iter()
                    .any(|warning| warning.contains(target.path.to_string_lossy().as_ref()))
            {
                warnings.push(format!(
                    "{} has local edits outside tracked managed spans and will be overwritten",
                    target.path.display()
                ));
            }
        }
    }

    Ok(warnings)
}

fn warning_applies_to_materialization(
    finding: &VerificationFinding,
    rendered: &[RenderedTarget],
    targets: &[(PathBuf, TargetChange)],
) -> bool {
    let changes_target = |target_path: &str| {
        targets.iter().any(|(path, change)| {
            *change != TargetChange::Unchanged && path.to_string_lossy() == target_path
        })
    };
    let will_render = |target_path: &str| {
        rendered
            .iter()
            .any(|target| target.path.to_string_lossy() == target_path)
    };

    match finding.kind.as_str() {
        "custom_implementation" | "removed_prayer" | "position_drift" | "orphan_marker" => finding
            .message
            .split('`')
            .nth(1)
            .is_some_and(|target_path| changes_target(target_path) || will_render(target_path)),
        "renderer_drift" => finding
            .message
            .split_whitespace()
            .next()
            .is_some_and(changes_target),
        _ => false,
    }
}

fn warning_message(finding: &VerificationFinding) -> String {
    match finding.kind.as_str() {
        "custom_implementation" => format!("Conflict: {}", finding.message),
        "removed_prayer" => format!("Conflict: {}", finding.message),
        "position_drift" => format!("Conflict: {}", finding.message),
        "orphan_marker" => format!("Warning: {}", finding.message),
        "renderer_drift" => format!("Warning: {}", finding.message),
        _ => finding.message.clone(),
    }
}

fn lockfile_line(change: LockfileChange, mode: MaterializationMode) -> String {
    let dry_run = mode.dry_run();
    match (change, dry_run) {
        (LockfileChange::Unchanged, _) => "Prayfile.lock unchanged".to_string(),
        (LockfileChange::Created, true) => "Prayfile.lock would be created".to_string(),
        (LockfileChange::Created, false) => "Prayfile.lock created".to_string(),
        (LockfileChange::Updated, true) => "Prayfile.lock would be updated".to_string(),
        (LockfileChange::Updated, false) => "Prayfile.lock updated".to_string(),
    }
}

fn target_verb(change: TargetChange, mode: MaterializationMode) -> &'static str {
    match (change, mode.dry_run()) {
        (TargetChange::Unchanged, _) => "unchanged",
        (TargetChange::Updated, true) => "would be updated",
        (TargetChange::Updated, false) => "updated",
        (TargetChange::Written, true) => "would be written",
        (TargetChange::Written, false) => "written",
    }
}

fn summary_footer(preview: &MaterializationPreview, mode: MaterializationMode) -> String {
    let package_count = preview.package_lines.len();
    let changed_targets = preview
        .targets
        .iter()
        .filter(|(_, change)| *change != TargetChange::Unchanged)
        .count();
    let changed_provisioned = preview
        .provisioned
        .iter()
        .filter(|(_, change)| *change != TargetChange::Unchanged)
        .count();
    let lockfile_changed = preview.lockfile != LockfileChange::Unchanged;
    let has_warnings = !preview.warnings.is_empty();

    if changed_targets == 0 && changed_provisioned == 0 && !lockfile_changed && !has_warnings {
        return format!(
            "{}. {package_count} packages, everything up to date.",
            mode.completion_label()
        );
    }

    let mut parts = Vec::new();
    if lockfile_changed {
        parts.push("lockfile changed".to_string());
    }
    if changed_targets > 0 {
        parts.push(format!(
            "{changed_targets} target file{} changed",
            if changed_targets == 1 { "" } else { "s" }
        ));
    }
    if changed_provisioned > 0 {
        parts.push(format!(
            "{changed_provisioned} provisioned file{} changed",
            if changed_provisioned == 1 { "" } else { "s" }
        ));
    }
    if has_warnings {
        parts.push(format!(
            "{} warning{}",
            preview.warnings.len(),
            if preview.warnings.len() == 1 { "" } else { "s" }
        ));
    }

    let detail = parts.join(", ");
    format!(
        "{}. {package_count} packages, {detail}.",
        mode.completion_label()
    )
}

fn package_source_label(declaration: &ManifestPackage) -> String {
    if let Some(path) = &declaration.path {
        return format!("path:{path}");
    }
    if let Some(source) = &declaration.source {
        return format!("source:{source}");
    }
    "default".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_reports_up_to_date_when_nothing_changes() {
        let preview = MaterializationPreview {
            package_lines: vec!["Using sample/base 1.0.0 (path:packages/base)".to_string()],
            lockfile: LockfileChange::Unchanged,
            targets: vec![(PathBuf::from("INSTRUCTIONS.md"), TargetChange::Unchanged)],
            provisioned: Vec::new(),
            warnings: Vec::new(),
        };

        assert!(
            summary_footer(&preview, MaterializationMode::Apply).contains("everything up to date")
        );
    }

    #[test]
    fn target_verb_uses_past_tense_for_apply() {
        assert_eq!(
            target_verb(TargetChange::Updated, MaterializationMode::Apply),
            "updated"
        );
        assert_eq!(
            target_verb(TargetChange::Written, MaterializationMode::Apply),
            "written"
        );
    }

    #[test]
    fn target_verb_uses_plan_tense_for_dry_run() {
        assert_eq!(
            target_verb(TargetChange::Updated, MaterializationMode::Plan),
            "would be updated"
        );
        assert_eq!(
            target_verb(TargetChange::Written, MaterializationMode::Plan),
            "would be written"
        );
    }

    #[test]
    fn grouped_provisioned_lines_groups_sibling_files() {
        let lines = grouped_provisioned_lines(
            &[
                (
                    PathBuf::from(".agents/skills/audit/SKILL.md"),
                    TargetChange::Written,
                ),
                (
                    PathBuf::from(".agents/skills/audit/details.md"),
                    TargetChange::Written,
                ),
            ],
            MaterializationMode::Install,
        );

        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains(".agents/skills/audit/"));
        assert!(lines[0].contains("2 files"));
    }
}
