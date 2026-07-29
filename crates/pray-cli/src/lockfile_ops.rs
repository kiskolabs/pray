use pray_core::lockfile::{lockfiles_equivalent, read_lockfile, Lockfile};
use pray_core::{PrayError, PrayResult};
use std::fs;
use std::path::Path;

pub(crate) fn build_lockfile(
    project: &pray_core::resolve::ResolvedProject,
    rendered: &[pray_core::render::RenderedTarget],
) -> PrayResult<Lockfile> {
    Ok(pray_core::lockfile::build_lockfile(
        project.lockfile_hash()?,
        project.environment.clone(),
        &project.project_root,
        &project.manifest.sources,
        &project.manifest.targets,
        rendered,
        &project.packages,
        &project.source_revisions,
        &project.source_host_keys,
    ))
}

pub(crate) fn ensure_existing_lockfile(path: &Path) -> PrayResult<Lockfile> {
    if !path.exists() {
        return Err(PrayError::Verify(
            "missing Prayfile.lock; run pray install first".to_string(),
        ));
    }
    read_lockfile(path)
}

pub(crate) fn ensure_lockfile_current(
    project: &pray_core::resolve::ResolvedProject,
    rendered: &[pray_core::render::RenderedTarget],
    existing: &Lockfile,
) -> PrayResult<()> {
    let current = build_lockfile(project, rendered)?;
    if !lockfiles_equivalent(&current, existing) {
        return Err(PrayError::Verify(
            "lockfile needs update; rerun pray install to refresh Prayfile.lock".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn ensure_rendered_outputs_current(
    project: &pray_core::resolve::ResolvedProject,
    rendered: &[pray_core::render::RenderedTarget],
) -> PrayResult<()> {
    for target in rendered {
        let path = project.project_root.join(&target.path);
        let on_disk = fs::read_to_string(&path).map_err(PrayError::from)?;
        if on_disk != target.content {
            return Err(PrayError::Render(format!(
                "{} is stale; rerun pray install to regenerate it or pray plan to inspect the diff",
                path.display()
            )));
        }
    }
    Ok(())
}
