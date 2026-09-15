//! Supported embedder surface for `pray-core` (RFC 0109).
//!
//! Other public modules remain available to the in-tree CLI. New embedders
//! import this module.

use std::path::{Path, PathBuf};

pub use crate::error::{PrayError, PrayResult};
pub use crate::lockfile::{
    build_lockfile, lockfile_hash, lockfiles_equivalent, parse_lockfile, read_lockfile,
    serialize_lockfile, write_lockfile, write_lockfile_if_changed, LockSource, LockedPackage,
    LockedTarget, Lockfile, ManagedSpanRecord, ProvisionedFileRecord,
};
pub use crate::manifest::{
    parse_manifest, read_manifest_text, Manifest, ManifestLocal, ManifestPackage, ManifestSource,
    ManifestTarget, RenderPolicy,
};
pub use crate::package_spec::{parse_package_spec, PackageSpec};
pub use crate::render::{render_project, write_rendered_targets, RenderedTarget};
pub use crate::resolve::{
    project_root_from_manifest, resolve_project, resolve_project_with_options, ResolvedLocalFile,
    ResolvedPackage, ResolvedProject,
};
pub use crate::resolve_context::ResolveOptions;
pub use crate::verify::{
    drift_project, inspect_locked_destinations, inspect_project, verify_project,
    VerificationFinding, VerificationReport,
};

/// Default `Prayfile` path under `working_directory`.
pub fn default_manifest_path(working_directory: impl AsRef<Path>) -> PathBuf {
    working_directory.as_ref().join("Prayfile")
}

/// Default `Prayfile.lock` path under `project_root`.
pub fn default_lockfile_path(project_root: impl AsRef<Path>) -> PathBuf {
    project_root.as_ref().join("Prayfile.lock")
}

#[cfg(test)]
mod tests {
    use super::{default_lockfile_path, default_manifest_path};
    use std::path::Path;

    #[test]
    fn default_paths_join_project_root() {
        let root = Path::new("/tmp/project");
        assert_eq!(default_manifest_path(root), root.join("Prayfile"));
        assert_eq!(default_lockfile_path(root), root.join("Prayfile.lock"));
    }
}
