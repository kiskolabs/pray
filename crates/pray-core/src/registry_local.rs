use crate::manifest::ManifestPackage;
use crate::registry::RegistryPackageResolution;
use crate::resolve_context::PackageResolutionContext;
use crate::PrayResult;
use std::path::{Path, PathBuf};

pub(crate) fn local_registry_root(project_root: &Path, source_url: &str) -> PathBuf {
    let path = source_url.strip_prefix("file://").unwrap_or(source_url);
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

pub(crate) fn resolve_non_http_registry(
    project_root: &Path,
    source_url: &str,
    declaration: &ManifestPackage,
    context: &PackageResolutionContext,
) -> PrayResult<Option<RegistryPackageResolution>> {
    if crate::ssh_client::is_pray_ssh_url(source_url) {
        return Ok(Some(
            crate::registry_ssh::resolve_ssh_registry_package_root(
                project_root,
                source_url,
                declaration,
                context,
            )?,
        ));
    }
    if source_url.starts_with("http://") || source_url.starts_with("https://") {
        return Ok(None);
    }
    let source_root = local_registry_root(project_root, source_url);
    Ok(Some(crate::registry::resolve_local_registry_package_root(
        project_root,
        source_url,
        &source_root,
        declaration,
        context,
    )?))
}
