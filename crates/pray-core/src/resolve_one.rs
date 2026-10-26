use super::{resolve_package_root, PackageRootResolution, ResolvedPackage};
use crate::lockfile::Lockfile;
use crate::manifest::{ManifestPackage, ManifestSource};
use crate::package_spec::{parse_package_spec, PackageSpec};
use crate::paths::find_prayspec_file;
use crate::resolve_context::ResolveOptions;
use crate::resolve_exports::{
    build_skill_file_index, load_export_bodies, load_package_file_bytes, select_exports,
};
use crate::resolve_git_sources::GitSourceSet;
use crate::source_path::{read_declared_file, DeclaredRead};
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;
use std::path::Path;

#[allow(clippy::too_many_arguments)]
pub(in crate::resolve) fn resolve_package(
    project_root: &Path,
    sources: &BTreeMap<String, ManifestSource>,
    git_sources: &GitSourceSet,
    user_config: &crate::config::PrayConfig,
    declaration: &ManifestPackage,
    lockfile: Option<&Lockfile>,
    options: &ResolveOptions,
) -> PrayResult<ResolvedPackage> {
    let PackageRootResolution {
        root,
        signer_fingerprint,
        registry_latest_version,
    } = resolve_package_root(
        project_root,
        sources,
        git_sources,
        user_config,
        declaration,
        lockfile,
        options,
    )?;
    let spec_path = find_prayspec_file(&root)?;
    let spec_name = spec_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| PrayError::Resolution(format!("no prayspec file found in {:?}", root)))?;
    let spec_text = match read_declared_file(&root, spec_name, "prayspec")? {
        DeclaredRead::Bytes(bytes) => String::from_utf8(bytes)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?,
        DeclaredRead::Missing => {
            return Err(PrayError::Resolution(format!(
                "no prayspec file found in {:?}",
                root
            )));
        }
    };
    let spec = parse_package_spec(&spec_text)?.canonicalized();
    if spec.name != declaration.name {
        return Err(PrayError::Resolution(format!(
            "package path {:?} declares {:?}, expected {:?}",
            root, spec.name, declaration.name
        )));
    }
    spec.satisfy_constraint(&declaration.constraint)?;
    let selected_exports = select_exports(declaration, &spec)?;
    let file_bytes = load_package_file_bytes(&root, &spec)?;
    let tree_hash = PackageSpec::tree_hash_from_file_bytes(&file_bytes)?;
    let export_bodies = load_export_bodies(&file_bytes, &spec, &selected_exports)?;
    let skill_files = build_skill_file_index(&spec);
    let source_checksum = tree_hash.clone();
    let upstream_context = super::resolve_upstream::UpstreamResolutionContext::new(
        project_root,
        sources,
        git_sources,
        user_config,
        lockfile,
        options,
    );
    let upstream =
        super::resolve_upstream::lock_path_upstream(&upstream_context, declaration, &spec)?;
    Ok(ResolvedPackage {
        declaration: declaration.clone(),
        root,
        spec: spec.clone(),
        tree_hash: tree_hash.clone(),
        artifact_hash: tree_hash.clone(),
        artifact: format!(
            "path:{}",
            spec_path.parent().unwrap_or(&spec_path).to_string_lossy()
        ),
        selected_exports,
        source_checksum,
        export_bodies,
        skill_files,
        signer_fingerprint,
        registry_latest_version,
        explicit: false,
        upstream,
    })
}
