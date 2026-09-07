use super::*;

pub fn resolve_project_in_context(
    manifest_path: &Path,
    project_root: &Path,
    options: &ResolveOptions,
) -> PrayResult<ResolvedProject> {
    let manifest_text = crate::manifest::read_manifest_text(manifest_path)?;
    let manifest = crate::manifest::parse_manifest(&manifest_text)?;
    resolve_manifest_in_context(manifest_path, project_root, manifest, options)
}

pub fn resolve_manifest_in_context(
    manifest_path: &Path,
    project_root: &Path,
    manifest: Manifest,
    options: &ResolveOptions,
) -> PrayResult<ResolvedProject> {
    let user_config = crate::config::load_user_config()?;
    let lockfile_path = project_root.join("Prayfile.lock");
    let lockfile_hints = crate::lockfile::read_lockfile(&lockfile_path).ok();
    crate::environment::validate_environment(&manifest, options.environment.as_deref())?;
    let manifest_hash = manifest.manifest_hash()?;
    let sources = source_map(&manifest.sources);
    let git_sources = prepare_git_sources(
        project_root,
        &manifest.sources,
        lockfile_hints.as_ref(),
        options,
    )?;
    let source_host_keys = prepare_pray_ssh_host_keys(&manifest.sources)?;
    let mut queue = crate::resolve_queue::ResolveQueue::seed(&manifest.packages)?;
    let outcome = queue.resolve_all(&manifest.packages, &sources, |declaration| {
        resolve_package(
            project_root,
            &sources,
            &git_sources,
            &user_config,
            declaration,
            lockfile_hints.as_ref(),
            options,
        )
    });
    if !outcome.errors.is_empty() {
        let message = outcome.errors.join("\n");
        return Err(if outcome.saw_network_error {
            PrayError::Network(message)
        } else {
            PrayError::Resolution(message)
        });
    }
    let packages = outcome.packages;
    let mut local_files = Vec::new();
    let mut local_errors = Vec::new();
    for local in &manifest.local {
        match resolve_local_file(project_root, local) {
            Ok(resolved) => local_files.push(resolved),
            Err(error) => local_errors.push(format!("local {}: {error}", local.path)),
        }
    }
    if !local_errors.is_empty() {
        return Err(PrayError::Resolution(local_errors.join("\n")));
    }
    crate::resolve_deps::reject_dependency_cycles(&packages)?;
    Ok(ResolvedProject {
        manifest_path: manifest_path.to_path_buf(),
        project_root: project_root.to_path_buf(),
        manifest,
        manifest_hash,
        packages,
        local_files,
        source_revisions: git_sources
            .into_iter()
            .filter_map(|(name, checkout)| {
                if checkout.revision.is_empty() {
                    None
                } else {
                    Some((name, checkout.revision))
                }
            })
            .collect(),
        source_host_keys,
        environment: options.environment.clone(),
    })
}
