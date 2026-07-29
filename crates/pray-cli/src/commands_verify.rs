use crate::lockfile_ops::{build_lockfile, ensure_rendered_outputs_current};
use crate::project_paths::{lockfile_path, manifest_path, resolve_project};
use pray_core::lockfile::{read_lockfile, write_lockfile, Lockfile};
use pray_core::render::{render_project, write_rendered_targets_with_previous_lockfile};
use pray_core::verify::{drift_project, format_verification_report, verify_project};
use pray_core::{PrayError, PrayResult};

pub(crate) fn render_command(check_only: bool) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let rendered = render_project(&project)?;
    if check_only {
        ensure_rendered_outputs_current(&project, &rendered)?;
        return Ok(());
    }
    let previous_lockfile = read_lockfile(&lockfile_path()).ok();
    let lockfile = build_lockfile(&project, &rendered)?;
    write_lockfile(&lockfile_path(), &lockfile)?;
    write_rendered_targets_with_previous_lockfile(&project, &rendered, previous_lockfile.as_ref())?;
    Ok(())
}

pub(crate) fn verify_command(strict: bool) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let lockfile = read_lockfile(&lockfile_path())?;
    let report = verify_project(&project, &lockfile, strict)?;
    if !report.is_clean() {
        eprintln!("{}", format_verification_report(&report));
    }
    Ok(())
}

pub(crate) fn drift_command(semantic: bool) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let lockfile = read_lockfile(&lockfile_path())?;
    if semantic {
        drift_semantic_command(&project, &lockfile)
    } else {
        drift_project(&project, &lockfile)?;
        Ok(())
    }
}

fn drift_semantic_command(
    project: &pray_core::resolve::ResolvedProject,
    lockfile: &Lockfile,
) -> PrayResult<()> {
    let lock_versions: std::collections::BTreeMap<&str, (&str, usize)> = lockfile
        .package
        .iter()
        .map(|package| {
            let managed_span_count = lockfile
                .managed_span
                .iter()
                .filter(|span| span.package == package.name)
                .count();
            (
                package.name.as_str(),
                (package.version.as_str(), managed_span_count),
            )
        })
        .collect();

    let mut lines = Vec::new();
    for package in &project.packages {
        let Some((locked_version, managed_span_count)) =
            lock_versions.get(package.declaration.name.as_str())
        else {
            continue;
        };
        if *locked_version != package.spec.version {
            lines.push(format!(
                "{} {} -> {} would change {} managed spans",
                package.declaration.name, locked_version, package.spec.version, managed_span_count,
            ));
        }
    }

    if lines.is_empty() {
        return Ok(());
    }

    let mut report = String::from("Semantic diff");
    for line in lines {
        report.push('\n');
        report.push_str(&line);
    }
    Err(PrayError::Verify(report))
}
