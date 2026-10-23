use crate::materialize::build_package_archive_bytes;
use crate::project_paths::manifest_path;
use crate::publish_context::{PublishDestLabel, PublishIdentity};
use crate::publish_dest::publish_destinations;
use crate::publish_integrity::{stored_package_artifact, stored_publish_matches};
use crate::publish_plan::{
    dest_target, print_publish_footer, print_publish_plan, report_published, report_unchanged,
    selected_packages, PublishStats,
};
use crate::publish_server::publish_to_server;
use crate::publish_version::published_registry_package_version;
use crate::registry_ops::{
    current_signer, current_signer_fingerprint, current_timestamp, load_registry_index,
    load_registry_package_metadata, registry_artifact_path, registry_metadata_path,
    write_output_bytes, write_registry_index, write_registry_package_metadata,
    write_torrent_manifest,
};
use crate::revision::{record_root_revision, RevisionAction};
use pray_core::distribution::read_registry_distribution_settings;
use pray_core::package_integrity::resolve_publish_signing_key;
use pray_core::publish_remote::PublishCliDest;
use pray_core::resolve::{resolve_project, ResolvedPackage};
use pray_core::{PrayError, PrayResult};
use std::path::{Path, PathBuf};

pub(crate) fn publish_command(
    roots: Vec<PathBuf>,
    servers: Vec<String>,
    to: Vec<String>,
    signing_key_path: Option<PathBuf>,
    resign: bool,
    dry_run: bool,
) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let (_project_root, dests) = publish_destinations(PublishCliDest { to, roots, servers })?;
    let signer = current_signer()?;
    let signer_fingerprint = current_signer_fingerprint();
    let published_at = current_timestamp()?;
    let signing_key = resolve_publish_signing_key(signing_key_path.as_deref())?;
    if resign && signing_key.is_none() {
        return Err(PrayError::Usage(
            "publish --resign requires a signing key".to_string(),
        ));
    }
    if resign && dests.iter().any(|dest| dest.server.is_some()) {
        return Err(PrayError::Usage(
            "publish --resign requires a local root destination".to_string(),
        ));
    }
    if resign && dry_run {
        return Err(PrayError::Usage(
            "publish --resign cannot be combined with --dry-run".to_string(),
        ));
    }
    if dry_run {
        print_publish_plan(&project, &dests)?;
        return Ok(());
    }
    let runtime = if dests.iter().any(|dest| dest.server.is_some()) {
        Some(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| {
                    PrayError::Unsupported(format!("failed to start publish runtime: {error}"))
                })?,
        )
    } else {
        None
    };

    let identity = PublishIdentity {
        signer: &signer,
        signer_fingerprint: signer_fingerprint.as_deref(),
        published_at,
        signing_key: signing_key.as_ref(),
        resign,
    };
    let mut stats = PublishStats {
        published: 0,
        unchanged: 0,
    };
    for dest in dests {
        let packages = selected_packages(&project, &dest.packages)?;
        let target = dest_target(&dest);
        let label = PublishDestLabel {
            dest_name: &dest.name,
            target: &target,
        };
        if let Some(root) = &dest.root {
            stats.merge(publish_to_root(&packages, &identity, root, &label)?);
            record_root_revision(root, RevisionAction::Publish)?;
        }
        if let (Some(server_url), Some(runtime)) = (&dest.server, &runtime) {
            stats.merge(publish_to_server(
                &packages, &identity, server_url, runtime, &label,
            )?);
        }
    }
    print_publish_footer(&stats);
    Ok(())
}

pub(crate) fn publish_to_root(
    packages: &[&ResolvedPackage],
    identity: &PublishIdentity<'_>,
    root: &Path,
    label: &PublishDestLabel<'_>,
) -> PrayResult<PublishStats> {
    let mut registry_index = load_registry_index(root)?;
    let distribution = read_registry_distribution_settings(root)?;
    let mut package_names = registry_index
        .packages
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let mut stats = PublishStats {
        published: 0,
        unchanged: 0,
    };

    for package in packages {
        let artifact_path =
            registry_artifact_path(&package.declaration.name, &package.spec.version);
        let metadata_path = registry_metadata_path(root, &package.declaration.name);
        let mut metadata =
            load_registry_package_metadata(&metadata_path, &package.declaration.name)?;
        let existing = metadata
            .versions
            .iter()
            .find(|entry| entry.version == package.spec.version);
        let stored_artifact = existing.and_then(|entry| {
            stored_package_artifact(root, &artifact_path, package, entry, &distribution)
        });
        if !identity.resign
            && existing
                .zip(stored_artifact.as_deref())
                .is_some_and(|(entry, artifact_bytes)| {
                    stored_publish_matches(artifact_bytes, package, entry)
                })
        {
            package_names.insert(package.declaration.name.clone());
            // Intent: unchanged packages keep on-disk catalog bytes (newline, key order).
            report_unchanged(package, label.dest_name, label.target);
            stats.unchanged += 1;
            continue;
        }

        let archive_bytes = build_package_archive_bytes(package)?;
        write_output_bytes(&root.join(&artifact_path), &archive_bytes)?;
        write_torrent_manifest(root, package, &artifact_path, &archive_bytes, &distribution)?;
        let mut version_entry = published_registry_package_version(
            package,
            identity.signer,
            identity.signer_fingerprint,
            identity.published_at,
            identity.signing_key,
            &artifact_path,
            &archive_bytes,
        )?;
        if let Some(existing) = metadata
            .versions
            .iter()
            .find(|entry| entry.version == version_entry.version)
        {
            version_entry.yanked = existing.yanked;
            if stored_artifact.is_some() {
                version_entry.published_at = existing.published_at;
            }
        }
        metadata
            .versions
            .retain(|entry| entry.version != version_entry.version);
        metadata.versions.push(version_entry);
        write_registry_package_metadata(&metadata_path, &metadata)?;
        package_names.insert(package.declaration.name.clone());
        report_published(package, label.dest_name, label.target, &artifact_path);
        stats.published += 1;
    }

    registry_index.packages = package_names.into_iter().collect();
    write_registry_index(root, &registry_index)?;
    Ok(stats)
}
