use crate::lockfile::Lockfile;
use crate::manifest::ManifestSource;
use crate::resolve_context::ResolveOptions;
use crate::resolve_git_ensure::ensure_git_repository;
use crate::resolve_git_sources::{
    is_local_filesystem_source, local_git_repo_path, local_git_source_root,
    pinned_revision_for_source, GitSourceCheckout,
};
use crate::{PrayError, PrayResult};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) struct GitSourceSet {
    project_root: PathBuf,
    sources: BTreeMap<String, ManifestSource>,
    pins: BTreeMap<String, String>,
    refresh: bool,
    offline: bool,
    checkouts: RefCell<BTreeMap<String, GitSourceCheckout>>,
}

impl GitSourceSet {
    pub(crate) fn new(
        project_root: &Path,
        sources: &[ManifestSource],
        lockfile: Option<&Lockfile>,
        options: &ResolveOptions,
    ) -> Self {
        let refresh = options.refresh_source_revisions;
        let offline = options.offline;
        let mut pins = BTreeMap::new();
        let mut git_sources = BTreeMap::new();
        for source in sources {
            if source.kind != "git" {
                continue;
            }
            git_sources.insert(source.name.clone(), source.clone());
            if !refresh {
                if let Some(revision) = pinned_revision_for_source(lockfile, source) {
                    pins.insert(source.name.clone(), revision);
                }
            }
        }
        Self {
            project_root: project_root.to_path_buf(),
            sources: git_sources,
            pins,
            refresh,
            offline,
            checkouts: RefCell::new(BTreeMap::new()),
        }
    }

    pub(crate) fn ensure(&self, name: &str) -> PrayResult<GitSourceCheckout> {
        if let Some(existing) = self.checkouts.borrow().get(name).cloned() {
            return Ok(existing);
        }
        let Some(source) = self.sources.get(name) else {
            return Err(PrayError::Resolution(format!(
                "git source {name} was not prepared"
            )));
        };
        let Some(checkout) = prepare_one_git_source(
            &self.project_root,
            source,
            self.pins.get(name).map(String::as_str),
            self.refresh,
            self.offline,
        )?
        else {
            return Err(PrayError::Resolution(format!(
                "git source {name} was not prepared"
            )));
        };
        self.checkouts
            .borrow_mut()
            .insert(name.to_string(), checkout.clone());
        Ok(checkout)
    }

    pub(crate) fn revisions(&self) -> BTreeMap<String, String> {
        self.checkouts
            .borrow()
            .iter()
            .filter_map(|(name, checkout)| {
                if checkout.revision.is_empty() {
                    None
                } else {
                    Some((name.clone(), checkout.revision.clone()))
                }
            })
            .collect()
    }
}

pub(crate) fn prepare_git_sources(
    project_root: &Path,
    sources: &[ManifestSource],
    lockfile: Option<&Lockfile>,
    options: &ResolveOptions,
) -> PrayResult<GitSourceSet> {
    Ok(GitSourceSet::new(project_root, sources, lockfile, options))
}

fn prepare_one_git_source(
    project_root: &Path,
    source: &ManifestSource,
    pinned_revision: Option<&str>,
    refresh: bool,
    offline: bool,
) -> PrayResult<Option<GitSourceCheckout>> {
    let clone_url = source.url.strip_prefix("git+").unwrap_or(&source.url);
    if is_local_filesystem_source(clone_url)
        && local_git_repo_path(project_root, clone_url).is_none()
    {
        if let Some(source_root) = local_git_source_root(project_root, clone_url) {
            return Ok(Some(GitSourceCheckout {
                cache_directory: source_root,
                revision: String::new(),
                subdir: source.subdir.clone(),
            }));
        }
    }
    let (cache_directory, revision) = ensure_git_repository(
        project_root,
        clone_url,
        refresh,
        pinned_revision,
        source.subdir.as_deref(),
        offline,
    )?;
    Ok(Some(GitSourceCheckout {
        cache_directory,
        revision,
        subdir: source.subdir.clone(),
    }))
}
