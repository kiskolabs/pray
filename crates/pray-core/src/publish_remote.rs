use crate::manifest::{Manifest, ManifestPackage, ManifestSource};
use crate::paths::validate_project_relative_path;
use crate::resolve::implied_source_name;
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub use crate::publish_select::{
    resolve_path_remote, resolve_publish_destinations, PublishCliDest, PublishDestination,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestPublishRemote {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,
}

pub fn validate_publish_remotes(manifest: &Manifest) -> PrayResult<()> {
    let mut seen = BTreeSet::new();
    for remote in &manifest.publish_remotes {
        if remote.name.trim().is_empty() {
            return Err(PrayError::Parse {
                kind: "manifest",
                message: "publish requires a name".to_string(),
            });
        }
        if !seen.insert(remote.name.clone()) {
            return Err(PrayError::Manifest(format!(
                "duplicate publish remote: {}",
                remote.name
            )));
        }
        match (&remote.path, &remote.url) {
            (Some(_), Some(_)) => {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: format!(
                        "publish \"{}\" must set path: or a URL, not both",
                        remote.name
                    ),
                });
            }
            (None, None) => {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: format!("publish \"{}\" requires path: or a URL", remote.name),
                });
            }
            (Some(path), None) => {
                validate_project_relative_path(path)?;
            }
            (None, Some(url)) => {
                if !is_publish_url(url) {
                    return Err(PrayError::Parse {
                        kind: "manifest",
                        message: format!(
                            "publish \"{}\" URL must be https://, http://, pray+ssh://, or ssh+pray://",
                            remote.name
                        ),
                    });
                }
            }
        }
        validate_remote_packages(manifest, remote)?;
    }
    Ok(())
}

fn validate_remote_packages(manifest: &Manifest, remote: &ManifestPublishRemote) -> PrayResult<()> {
    for package_name in &remote.packages {
        let Some(package) = manifest
            .packages
            .iter()
            .find(|package| package.name == *package_name)
        else {
            return Err(PrayError::Manifest(format!(
                "publish \"{}\" lists unknown package {package_name}",
                remote.name
            )));
        };
        if !package_is_path_owned(package, &manifest.sources) {
            return Err(PrayError::Manifest(format!(
                "publish \"{}\" lists {package_name}, which is not a path package",
                remote.name
            )));
        }
    }
    Ok(())
}

pub fn package_is_path_owned(package: &ManifestPackage, sources: &[ManifestSource]) -> bool {
    if package.path.is_some() {
        return true;
    }
    if package.git.is_some() || package.tarball.is_some() || package.oci.is_some() {
        return false;
    }
    let map = source_map(sources);
    match implied_source_name(package, &map) {
        Ok(Some(name)) => map.get(&name).is_some_and(|source| source.kind == "path"),
        Ok(None) | Err(_) => false,
    }
}

pub fn path_owned_package_names(manifest: &Manifest) -> Vec<String> {
    manifest
        .packages
        .iter()
        .filter(|package| package_is_path_owned(package, &manifest.sources))
        .map(|package| package.name.clone())
        .collect()
}

pub fn allowed_publish_names(manifest: &Manifest, listed: &[String]) -> Vec<String> {
    if listed.is_empty() {
        path_owned_package_names(manifest)
    } else {
        listed.to_vec()
    }
}

fn is_publish_url(url: &str) -> bool {
    url.starts_with("https://")
        || url.starts_with("http://")
        || url.starts_with("pray+ssh://")
        || url.starts_with("ssh+pray://")
}

fn source_map(sources: &[ManifestSource]) -> BTreeMap<String, ManifestSource> {
    sources
        .iter()
        .cloned()
        .map(|source| (source.name.clone(), source))
        .collect()
}
