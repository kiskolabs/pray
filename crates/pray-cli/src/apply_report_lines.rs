use pray_core::lockfile::Lockfile;
use pray_core::manifest::ManifestPackage;
use pray_core::resolve::ResolvedProject;

pub(crate) fn materialization_summary_lines(
    previous: Option<&Lockfile>,
    updated: &Lockfile,
    project: &ResolvedProject,
) -> Vec<String> {
    let mut lines = local_summary_lines(previous, project);
    lines.extend(package_summary_lines(previous, updated, project));
    lines
}

fn local_summary_lines(previous: Option<&Lockfile>, project: &ResolvedProject) -> Vec<String> {
    let previous_checksums = previous_local_checksums(previous);
    let mut lines = Vec::new();
    for local in &project.local_files {
        if local.content.is_empty() && local.optional {
            continue;
        }
        let checksum = local.source_checksum.as_str();
        match previous_checksums.get(local.manifest_path.as_str()) {
            None => lines.push(format!("Installing {} ({checksum})", local.manifest_path)),
            Some(previous) if *previous == checksum => lines.push(format!(
                "Using {} ({checksum} checked)",
                local.manifest_path
            )),
            Some(previous) => lines.push(format!(
                "Updating {} ({checksum} was {previous})",
                local.manifest_path
            )),
        }
    }
    lines
}

fn previous_local_checksums(previous: Option<&Lockfile>) -> std::collections::BTreeMap<&str, &str> {
    previous
        .into_iter()
        .flat_map(|lockfile| lockfile.managed_span.iter())
        .filter(|span| span.package == pray_core::hashing::LOCAL_EMBED_PACKAGE)
        .map(|span| (span.export.as_str(), span.source_checksum.as_str()))
        .collect()
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

fn package_source_label(declaration: &ManifestPackage) -> String {
    if let Some(path) = &declaration.path {
        return format!("path:{path}");
    }
    if let Some(source) = &declaration.source {
        return format!("source:{source}");
    }
    "default".to_string()
}

pub(crate) fn outdated_local_lines(
    previous: Option<&Lockfile>,
    project: &ResolvedProject,
) -> Vec<String> {
    let previous_checksums = previous_local_checksums(previous);
    let mut lines = Vec::new();
    for local in &project.local_files {
        if local.content.is_empty() && local.optional {
            continue;
        }
        let checksum = local.source_checksum.as_str();
        match previous_checksums.get(local.manifest_path.as_str()) {
            Some(previous) if *previous != checksum => {
                lines.push(format!("{} {previous} -> {checksum}", local.manifest_path))
            }
            None if previous.is_some() => {
                lines.push(format!("{} (new) -> {checksum}", local.manifest_path))
            }
            _ => {}
        }
    }
    lines
}

pub(crate) fn summary_footer(
    preview: &super::MaterializationPreview,
    mode: super::MaterializationMode,
) -> String {
    let inventory = inventory_phrase(preview);
    let changed_targets = preview
        .targets
        .iter()
        .filter(|(_, change)| *change != super::TargetChange::Unchanged)
        .count();
    let changed_provisioned = preview
        .provisioned
        .iter()
        .filter(|(_, change)| *change != super::TargetChange::Unchanged)
        .count();
    let lockfile_changed = preview.lockfile != super::LockfileChange::Unchanged;
    let has_warnings = !preview.warnings.is_empty();

    if changed_targets == 0 && changed_provisioned == 0 && !lockfile_changed && !has_warnings {
        return format!(
            "{}. {inventory}, everything up to date.",
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
    format!("{}. {inventory}, {detail}.", mode.completion_label())
}

fn inventory_phrase(preview: &super::MaterializationPreview) -> String {
    let local_count = preview
        .package_lines
        .iter()
        .filter(|line| line.contains("(sha256:"))
        .count();
    let package_count = preview.package_lines.len().saturating_sub(local_count);
    let mut parts = vec![format!("{package_count} packages")];
    if local_count > 0 {
        parts.push(format!(
            "{local_count} local file{}",
            if local_count == 1 { "" } else { "s" }
        ));
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::super::{LockfileChange, MaterializationMode, MaterializationPreview, TargetChange};
    use super::summary_footer;
    use std::path::PathBuf;

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
    fn summary_counts_local_checksum_lines_apart_from_packages() {
        let preview = MaterializationPreview {
            package_lines: vec![
                "Using .agents/project.md (sha256:abc checked)".to_string(),
                "Using sample/base 1.0.0 (path:packages/base)".to_string(),
            ],
            lockfile: LockfileChange::Unchanged,
            targets: vec![(PathBuf::from("AGENTS.md"), TargetChange::Unchanged)],
            provisioned: Vec::new(),
            warnings: Vec::new(),
        };

        assert_eq!(
            summary_footer(&preview, MaterializationMode::Install),
            "Install complete. 1 packages, 1 local file, everything up to date."
        );
    }
}
