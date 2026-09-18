use pray_core::publish_remote::{allowed_publish_names, PublishDestination};
use pray_core::resolve::{ResolvedPackage, ResolvedProject};
use pray_core::{PrayError, PrayResult};

pub(crate) fn selected_packages<'a>(
    project: &'a ResolvedProject,
    listed: &[String],
) -> PrayResult<Vec<&'a ResolvedPackage>> {
    let allowed = allowed_publish_names(&project.manifest, listed);
    let selected: Vec<&ResolvedPackage> = project
        .packages
        .iter()
        .filter(|package| allowed.iter().any(|name| name == &package.declaration.name))
        .collect();
    if selected.is_empty() {
        return Err(PrayError::Usage(
            "no path packages to publish; remote dependencies are not published".to_string(),
        ));
    }
    for package in &selected {
        package.spec.require_release_version()?;
    }
    Ok(selected)
}

pub(crate) fn print_publish_plan(
    project: &ResolvedProject,
    dests: &[PublishDestination],
) -> PrayResult<()> {
    for dest in dests {
        let packages = selected_packages(project, &dest.packages)?;
        let target = dest
            .root
            .as_ref()
            .map(|path| path.display().to_string())
            .or_else(|| dest.server.clone())
            .unwrap_or_else(|| dest.name.clone());
        for package in packages {
            println!(
                "publish {} {} -> {} ({target})",
                package.declaration.name, package.spec.version, dest.name
            );
        }
    }
    Ok(())
}
