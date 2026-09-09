use super::super::implied_source_name;
use super::resolution_context::UpstreamResolutionContext;
use crate::manifest::ManifestPackage;
use crate::package_upstream::LockedUpstream;
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;

pub(in crate::resolve) fn lock_path_upstream(
    context: &UpstreamResolutionContext<'_>,
    declaration: &ManifestPackage,
    spec: &crate::package_spec::PackageSpec,
) -> PrayResult<Option<LockedUpstream>> {
    let Some(upstream) = &spec.upstream else {
        return Ok(None);
    };
    if spec.name == upstream.name {
        return Err(PrayError::Resolution(format!(
            "package {} cannot use itself as upstream",
            spec.name
        )));
    }
    if declaration.path.is_none() {
        return Ok(None);
    }
    let refresh = context.options.ignore_locked_versions
        || context
            .options
            .unlocked_packages
            .contains(&declaration.name);
    let locked = context.lockfile.and_then(|lockfile| {
        lockfile
            .package
            .iter()
            .find(|package| package.name == declaration.name)
            .and_then(|package| package.upstream.clone())
    });
    let (name, constraint, source) = match (&locked, refresh) {
        (Some(locked), false) => {
            if locked.name != upstream.name {
                return Err(PrayError::Integrity(format!(
                    "locked upstream name mismatch for {}: expected {}, found {}",
                    declaration.name, locked.name, upstream.name
                )));
            }
            (
                locked.name.clone(),
                format!("= {}", locked.version),
                locked.source.clone(),
            )
        }
        _ => (upstream.name.clone(), upstream.constraint.clone(), None),
    };
    let source = source.or(implied_upstream_source(&name, context.sources)?);
    let resolved = context.resolve(&name, &constraint, source.as_deref())?;
    let resolved_upstream = LockedUpstream {
        name,
        version: resolved.spec.version,
        source,
        tree_hash: resolved.tree_hash,
        artifact_hash: resolved.artifact_hash,
    };
    if let (Some(locked), false) = (&locked, refresh) {
        ensure_locked_upstream_matches(locked, &resolved_upstream)?;
    }
    Ok(Some(resolved_upstream))
}

fn implied_upstream_source(
    name: &str,
    sources: &BTreeMap<String, crate::manifest::ManifestSource>,
) -> PrayResult<Option<String>> {
    implied_source_name(
        &ManifestPackage {
            name: name.to_string(),
            ..ManifestPackage::default()
        },
        sources,
    )
}

pub(super) fn ensure_locked_upstream_matches(
    locked: &LockedUpstream,
    resolved: &LockedUpstream,
) -> PrayResult<()> {
    if locked.name != resolved.name || locked.version != resolved.version {
        return Err(PrayError::Integrity(
            "locked upstream identity mismatch".to_string(),
        ));
    }
    if locked.source != resolved.source {
        return Err(PrayError::Integrity(
            "locked upstream source mismatch".to_string(),
        ));
    }
    if locked.tree_hash != resolved.tree_hash {
        return Err(PrayError::Integrity(
            "locked upstream tree hash mismatch".to_string(),
        ));
    }
    if locked.artifact_hash != resolved.artifact_hash {
        return Err(PrayError::Integrity(
            "locked upstream artifact hash mismatch".to_string(),
        ));
    }
    Ok(())
}
