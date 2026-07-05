use crate::lockfile::Lockfile;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolveOptions {
    pub offline: bool,
    pub unlocked_packages: BTreeSet<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackageResolutionContext {
    pub preferred_version: Option<String>,
    pub offline: bool,
}

impl PackageResolutionContext {
    pub fn from_lockfile(
        lockfile: Option<&Lockfile>,
        package_name: &str,
        unlocked_packages: &BTreeSet<String>,
        offline: bool,
    ) -> Self {
        let preferred_version = if unlocked_packages.contains(package_name) {
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
            offline,
        }
    }
}
