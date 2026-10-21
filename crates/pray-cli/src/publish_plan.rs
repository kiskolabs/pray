use crate::cli_output::{println_stdout, println_verbose, redact_destination_target};
use pray_core::publish_remote::{allowed_publish_names, PublishDestination};
use pray_core::resolve::{ResolvedPackage, ResolvedProject};
use pray_core::{PrayError, PrayResult};

pub(crate) struct PublishStats {
    pub published: usize,
    pub unchanged: usize,
}

impl PublishStats {
    pub(crate) fn merge(&mut self, other: PublishStats) {
        self.published += other.published;
        self.unchanged += other.unchanged;
    }
}

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

pub(crate) fn dest_target(dest: &PublishDestination) -> String {
    let raw = dest
        .root
        .as_ref()
        .map(|path| path.display().to_string())
        .or_else(|| dest.server.clone())
        .unwrap_or_else(|| dest.name.clone());
    redact_destination_target(&raw)
}

pub(crate) fn print_publish_plan(
    project: &ResolvedProject,
    dests: &[PublishDestination],
) -> PrayResult<()> {
    for dest in dests {
        let packages = selected_packages(project, &dest.packages)?;
        let target = dest_target(dest);
        for package in packages {
            println_stdout(format_publish_line(
                "publish",
                &package.declaration.name,
                &package.spec.version,
                &dest.name,
                &target,
            ));
        }
    }
    Ok(())
}

pub(crate) fn report_published(
    package: &ResolvedPackage,
    dest_name: &str,
    target: &str,
    artifact_path: &str,
) {
    println_stdout(format_publish_line(
        "published",
        &package.declaration.name,
        &package.spec.version,
        dest_name,
        target,
    ));
    println_verbose(format!("  artifact {artifact_path}"));
}

pub(crate) fn report_unchanged(package: &ResolvedPackage, dest_name: &str, target: &str) {
    println_stdout(format_publish_line(
        "unchanged",
        &package.declaration.name,
        &package.spec.version,
        dest_name,
        target,
    ));
    println_verbose("  skip reason: stored publish matches");
}

pub(crate) fn print_publish_footer(stats: &PublishStats) {
    if stats.published == 0 && stats.unchanged == 0 {
        println_stdout("nothing to publish");
        return;
    }
    println_stdout(format!(
        "{} published, {} unchanged",
        stats.published, stats.unchanged
    ));
}

fn format_publish_line(
    verb: &str,
    name: &str,
    version: &str,
    dest_name: &str,
    target: &str,
) -> String {
    format!(
        "{verb} {name} {version} -> {} ({})",
        redact_destination_target(dest_name),
        redact_destination_target(target)
    )
}
