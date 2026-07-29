use crate::constraint::version_satisfies;
use crate::registry::{RegistryPackageMetadata, RegistryPackageVersion};
use crate::{PrayError, PrayResult};
use semver::Version;

pub(crate) fn select_package_version(
    metadata: &RegistryPackageMetadata,
    constraint: &str,
    preferred_version: Option<&str>,
) -> PrayResult<RegistryPackageVersion> {
    if let Some(preferred_version) = preferred_version {
        if let Some(version) = metadata
            .versions
            .iter()
            .find(|version| version.version == preferred_version)
        {
            if version_satisfies(&version.version, constraint)? {
                // SPEC §60: existing lockfile may continue using a yanked version.
                return Ok(version.clone());
            }
            // Prayfile constraint changed; fall through to the highest satisfying version.
        }
    }
    let mut selected: Option<RegistryPackageVersion> = None;
    for version in &metadata.versions {
        if version.yanked {
            continue;
        }
        if !version_satisfies(&version.version, constraint)? {
            continue;
        }
        match &selected {
            Some(existing) if compare_versions(&version.version, &existing.version)? <= 0 => {}
            _ => selected = Some(version.clone()),
        }
    }
    selected.ok_or_else(|| {
        PrayError::Resolution(format!(
            "no registry version for {} satisfies {}",
            metadata.name, constraint
        ))
    })
}

pub fn apply_yank_policy(
    package_name: &str,
    selected: &RegistryPackageVersion,
    fail_on_yanked: bool,
) -> PrayResult<()> {
    if !selected.yanked {
        return Ok(());
    }
    if fail_on_yanked {
        return Err(PrayError::Resolution(format!(
            "package {package_name} {} is yanked",
            selected.version
        )));
    }
    eprintln!(
        "[pray] warning: package {package_name} {} is yanked; keeping locked version. Run `pray update` to move away.",
        selected.version
    );
    Ok(())
}

pub fn set_package_version_yanked(
    metadata: &mut RegistryPackageMetadata,
    version: &str,
    yanked: bool,
) -> PrayResult<()> {
    let entry = metadata
        .versions
        .iter_mut()
        .find(|entry| entry.version == version)
        .ok_or_else(|| {
            PrayError::Resolution(format!(
                "package {} has no version {version}",
                metadata.name
            ))
        })?;
    entry.yanked = yanked;
    Ok(())
}

pub fn highest_registry_version(
    metadata: &RegistryPackageMetadata,
) -> PrayResult<Option<RegistryPackageVersion>> {
    let mut selected: Option<RegistryPackageVersion> = None;
    for version in &metadata.versions {
        if version.yanked {
            continue;
        }
        match &selected {
            Some(existing) if compare_versions(&version.version, &existing.version)? <= 0 => {}
            _ => selected = Some(version.clone()),
        }
    }
    Ok(selected)
}

pub fn version_is_greater_than(left: &str, right: &str) -> PrayResult<bool> {
    Ok(compare_versions(left, right)? > 0)
}

fn compare_versions(left: &str, right: &str) -> PrayResult<i32> {
    let left = Version::parse(left).map_err(|error| PrayError::Resolution(error.to_string()))?;
    let right = Version::parse(right).map_err(|error| PrayError::Resolution(error.to_string()))?;
    Ok(if left < right {
        -1
    } else if left == right {
        0
    } else {
        1
    })
}
