use crate::lockfile::Lockfile;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolveOptions {
    pub offline: bool,
    pub unlocked_packages: BTreeSet<String>,
    /// When true, git sources fetch remote HEAD instead of the revision pinned in Prayfile.lock.
    pub refresh_source_revisions: bool,
    /// When true, resolve against registry constraints instead of versions pinned in Prayfile.lock.
    pub ignore_locked_versions: bool,
    /// When true, refuse yanked versions even if the lockfile pins them (RFC 0050 `--strict`).
    pub fail_on_yanked: bool,
    /// Selected render environment; does not change package resolution.
    pub environment: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackageResolutionContext {
    pub preferred_version: Option<String>,
    pub offline: bool,
    pub fail_on_yanked: bool,
}

impl PackageResolutionContext {
    pub fn from_lockfile(
        lockfile: Option<&Lockfile>,
        package_name: &str,
        options: &ResolveOptions,
    ) -> Self {
        let preferred_version =
            if options.ignore_locked_versions || options.unlocked_packages.contains(package_name) {
                None
            } else {
                lockfile.and_then(|lockfile| {
                    lockfile
                        .package
                        .iter()
                        .find(|package| package.name == package_name)
                        .map(|package| package.version.clone())
                })
            };
        Self {
            preferred_version,
            offline: options.offline,
            fail_on_yanked: options.fail_on_yanked,
        }
    }
}
