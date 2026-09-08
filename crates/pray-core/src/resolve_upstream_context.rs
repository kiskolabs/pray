use super::super::{resolve_package, ResolvedPackage};
use crate::lockfile::Lockfile;
use crate::manifest::ManifestPackage;
use crate::resolve_context::ResolveOptions;
use crate::PrayResult;
use std::collections::BTreeMap;
use std::path::Path;

pub(in crate::resolve) struct UpstreamResolutionContext<'a> {
    project_root: &'a Path,
    pub(super) sources: &'a BTreeMap<String, crate::manifest::ManifestSource>,
    git_sources: &'a BTreeMap<String, crate::resolve_git_sources::GitSourceCheckout>,
    user_config: &'a crate::config::PrayConfig,
    pub(super) lockfile: Option<&'a Lockfile>,
    pub(super) options: &'a ResolveOptions,
}

impl<'a> UpstreamResolutionContext<'a> {
    pub(in crate::resolve) fn new(
        project_root: &'a Path,
        sources: &'a BTreeMap<String, crate::manifest::ManifestSource>,
        git_sources: &'a BTreeMap<String, crate::resolve_git_sources::GitSourceCheckout>,
        user_config: &'a crate::config::PrayConfig,
        lockfile: Option<&'a Lockfile>,
        options: &'a ResolveOptions,
    ) -> Self {
        Self {
            project_root,
            sources,
            git_sources,
            user_config,
            lockfile,
            options,
        }
    }

    pub(super) fn resolve(
        &self,
        name: &str,
        constraint: &str,
        source: Option<&str>,
    ) -> PrayResult<ResolvedPackage> {
        let declaration = ManifestPackage {
            name: name.to_string(),
            constraint: constraint.to_string(),
            source: source.map(str::to_string),
            ..ManifestPackage::default()
        };
        resolve_package(
            self.project_root,
            self.sources,
            self.git_sources,
            self.user_config,
            &declaration,
            self.lockfile,
            self.options,
        )
    }
}
