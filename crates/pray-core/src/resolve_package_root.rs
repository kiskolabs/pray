use crate::lockfile::Lockfile;
use crate::manifest::{ManifestPackage, ManifestSource};
use crate::registry::{resolve_local_registry_package_root, resolve_registry_package_root};
use crate::resolve_context::{PackageResolutionContext, ResolveOptions};
use crate::resolve_git_sources::{resolve_git_package_root, GitSourceCheckout};
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct PackageRootResolution {
    pub root: PathBuf,
    pub signer_fingerprint: Option<String>,
    pub registry_latest_version: Option<String>,
}

pub(crate) fn resolve_package_root(
    project_root: &Path,
    sources: &BTreeMap<String, ManifestSource>,
    git_sources: &BTreeMap<String, GitSourceCheckout>,
    user_config: &crate::config::PrayConfig,
    declaration: &ManifestPackage,
    lockfile: Option<&Lockfile>,
    options: &ResolveOptions,
) -> PrayResult<PackageRootResolution> {
    if let Some(local_path) = user_config.local.package.get(&declaration.name) {
        return Ok(PackageRootResolution {
            root: project_root.join(local_path),
            signer_fingerprint: None,
            registry_latest_version: None,
        });
    }
    if let Some(path) = &declaration.path {
        return Ok(PackageRootResolution {
            root: project_root.join(path),
            signer_fingerprint: None,
            registry_latest_version: None,
        });
    }
    if let Some(tarball) = &declaration.tarball {
        return Ok(PackageRootResolution {
            root: crate::resolve_tarball::resolve_tarball_package_root(
                project_root,
                tarball,
                options,
            )?,
            signer_fingerprint: None,
            registry_latest_version: None,
        });
    }
    let source_name = implied_source_name(declaration, sources)?;
    if let Some(source_name) = source_name {
        let source = sources
            .get(&source_name)
            .ok_or_else(|| PrayError::Resolution(format!("unknown source: {source_name}")))?;
        let context = PackageResolutionContext::from_lockfile(lockfile, &declaration.name, options);
        if let Some(local_path) = user_config.local.source.get(&source_name) {
            let source_root = project_root.join(local_path);
            let resolved = resolve_local_registry_package_root(
                project_root,
                &format!("local:{source_name}"),
                &source_root,
                declaration,
                &context,
            )?;
            return Ok(PackageRootResolution {
                root: resolved.root,
                signer_fingerprint: resolved.signer_fingerprint,
                registry_latest_version: resolved.registry_latest_version,
            });
        }
        if source.kind == "path" {
            let slug = declaration.name.replace('/', "-");
            return Ok(PackageRootResolution {
                root: project_root.join(&source.url).join(slug),
                signer_fingerprint: None,
                registry_latest_version: None,
            });
        }
        if source.kind == "registry" || source.kind == "static index" || source.kind == "pray_ssh" {
            let resolved =
                resolve_registry_package_root(project_root, &source.url, declaration, &context)?;
            return Ok(PackageRootResolution {
                root: resolved.root,
                signer_fingerprint: resolved.signer_fingerprint,
                registry_latest_version: resolved.registry_latest_version,
            });
        }
        if source.kind == "git" {
            let resolved = resolve_git_package_root(
                project_root,
                &source_name,
                &source.url,
                git_sources,
                declaration,
                &context,
            )?;
            return Ok(PackageRootResolution {
                root: resolved.root,
                signer_fingerprint: resolved.signer_fingerprint,
                registry_latest_version: resolved.registry_latest_version,
            });
        }
        return Err(PrayError::Unsupported(format!(
            "source kind {} not implemented yet",
            source.kind
        )));
    }
    if declaration.git.is_some() || declaration.oci.is_some() {
        return Err(PrayError::Unsupported(
            "remote sources are not implemented yet".to_string(),
        ));
    }
    let slug = declaration.name.replace('/', "-");
    Ok(PackageRootResolution {
        root: project_root.join(slug),
        signer_fingerprint: None,
        registry_latest_version: None,
    })
}

pub(crate) fn implied_source_name(
    declaration: &ManifestPackage,
    sources: &BTreeMap<String, ManifestSource>,
) -> PrayResult<Option<String>> {
    if let Some(name) = &declaration.source {
        return Ok(Some(name.clone()));
    }
    if let Some(namespace) = declaration.name.split_once('/').map(|(name, _)| name) {
        if sources.contains_key(namespace) {
            return Ok(Some(namespace.to_string()));
        }
    }
    match sources.len() {
        0 => Ok(None),
        1 => Ok(sources.keys().next().cloned()),
        _ => Err(PrayError::Resolution(format!(
            "package {} requires source: when multiple sources are declared and the package namespace does not match a source",
            declaration.name
        ))),
    }
}
