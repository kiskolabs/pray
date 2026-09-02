use crate::apply_report::{
    build_materialization_preview, print_materialization_report, MaterializationMode,
    MaterializationPreview,
};
use crate::lockfile_ops::{
    build_lockfile, ensure_existing_lockfile, ensure_lockfile_current,
    ensure_rendered_outputs_current,
};
use crate::project_paths::{
    lockfile_path, manifest_path, resolve_project_with_git_refresh_fallback,
};
use pray_core::lockfile::{read_lockfile, write_lockfile_if_changed};
use pray_core::render::{
    layout_rendered_targets, render_project, write_rendered_targets_with_previous_lockfile,
};
use pray_core::resolve::ResolvedProject;
use pray_core::resolve_context::ResolveOptions;
use pray_core::PrayResult;

pub(crate) fn install_command(
    locked: bool,
    frozen: bool,
    resolve_options: ResolveOptions,
    silent_report: bool,
) -> PrayResult<Option<MaterializationPreview>> {
    let report_mode = if locked {
        None
    } else {
        Some(MaterializationMode::Install)
    };
    materialize_command(locked, frozen, report_mode, resolve_options, silent_report)
}

pub(crate) fn apply_command() -> PrayResult<()> {
    materialize_command(
        false,
        false,
        Some(MaterializationMode::Apply),
        ResolveOptions::default(),
        false,
    )?;
    Ok(())
}

pub(crate) fn resolve_project_for_materialization(
    resolve_options: &ResolveOptions,
    locked: bool,
    frozen: bool,
) -> PrayResult<ResolvedProject> {
    resolve_project_with_git_refresh_fallback(&manifest_path(), resolve_options, !locked && !frozen)
}

pub(crate) fn materialize_command(
    locked: bool,
    frozen: bool,
    report_mode: Option<MaterializationMode>,
    resolve_options: ResolveOptions,
    silent_report: bool,
) -> PrayResult<Option<MaterializationPreview>> {
    let project = resolve_project_for_materialization(&resolve_options, locked, frozen)?;
    let rendered = render_project(&project)?;
    let lockfile_path = lockfile_path();
    if locked {
        let lockfile = ensure_existing_lockfile(&lockfile_path)?;
        ensure_lockfile_current(&project, &rendered, &lockfile)?;
        if frozen {
            ensure_rendered_outputs_current(&project, &rendered)?;
            return Ok(None);
        }
        write_rendered_targets_with_previous_lockfile(&project, &rendered, Some(&lockfile))?;
        return Ok(None);
    }

    let laid_out = layout_rendered_targets(&project, &rendered)?;
    let lockfile = build_lockfile(&project, &laid_out)?;
    let previous_lockfile = read_lockfile(&lockfile_path).ok();
    let preview = if report_mode.is_some() {
        Some(build_materialization_preview(
            &project,
            &laid_out,
            &lockfile,
            &lockfile_path,
            previous_lockfile.as_ref(),
        )?)
    } else {
        None
    };
    write_rendered_targets_with_previous_lockfile(&project, &rendered, previous_lockfile.as_ref())?;
    write_lockfile_if_changed(&lockfile_path, &lockfile)?;
    if !silent_report {
        if let (Some(preview), Some(mode)) = (&preview, report_mode) {
            print_materialization_report(preview, mode);
        }
    }
    Ok(preview)
}
