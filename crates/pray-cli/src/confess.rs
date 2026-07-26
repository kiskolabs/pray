use crate::{
    current_signer, lockfile_path, locked_package, manifest_path, resolve_project,
};
use pray_core::hashing::sha256_prefixed;
use pray_core::lockfile::read_lockfile;
use pray_core::registry::{submit_confession, ConfessionSubmission};
use pray_core::{PrayError, PrayResult};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn confess_command(
    package: Option<String>,
    from_lock: Option<String>,
    version: Option<String>,
    accepted: bool,
    rejected: bool,
    note: Option<String>,
    url: Option<String>,
) -> PrayResult<()> {
    if accepted == rejected {
        return Err(PrayError::Unsupported(
            "confess requires exactly one of --accepted or --rejected".to_string(),
        ));
    }

    let project = resolve_project(&manifest_path())?;
    let lockfile = read_lockfile(&lockfile_path()).ok();

    let (package_name, resolved_version, package_resolution) = if let Some(span_id) = from_lock {
        let lockfile = lockfile.as_ref().ok_or_else(|| {
            PrayError::Resolution("confess --from-lock requires an existing lockfile".to_string())
        })?;
        let span = lockfile
            .managed_span
            .iter()
            .find(|record| record.id == span_id)
            .ok_or_else(|| PrayError::Resolution(format!("lockfile span {span_id} not found")))?;
        let package_resolution = project
            .packages
            .iter()
            .find(|resolved_package| resolved_package.declaration.name == span.package)
            .ok_or_else(|| PrayError::Resolution(format!("package {} not found", span.package)))?;
        let locked_package = locked_package(lockfile, package_resolution).ok_or_else(|| {
            PrayError::Resolution(format!("lockfile package {} not found", span.package))
        })?;
        let resolved_version = match version {
            Some(requested_version) if requested_version != locked_package.version => {
                return Err(PrayError::Resolution(format!(
                    "lockfile span {} version {} does not match requested version {}",
                    span_id, locked_package.version, requested_version
                )));
            }
            Some(requested_version) => requested_version,
            None => locked_package.version.clone(),
        };
        (span.package.clone(), resolved_version, package_resolution)
    } else {
        let package_name = package
            .ok_or_else(|| PrayError::Unsupported("confess requires a package name".to_string()))?;
        let package_resolution = project
            .packages
            .iter()
            .find(|resolved_package| resolved_package.declaration.name == package_name)
            .ok_or_else(|| PrayError::Resolution(format!("package {package_name} not found")))?;
        let resolved_version = match version {
            Some(requested_version) if requested_version != package_resolution.spec.version => {
                return Err(PrayError::Resolution(format!(
                    "package {package_name} version {} does not match resolved version {}",
                    requested_version, package_resolution.spec.version
                )));
            }
            Some(requested_version) => requested_version,
            None => package_resolution.spec.version.clone(),
        };
        (package_name, resolved_version, package_resolution)
    };

    let source_name = package_resolution
        .declaration
        .source
        .as_ref()
        .ok_or_else(|| {
            PrayError::Resolution(format!("package {package_name} is missing a source"))
        })?;
    let source_url = if let Some(url) = url {
        url
    } else {
        project
            .manifest
            .sources
            .iter()
            .find(|source| source.name == *source_name)
            .map(|source| source.url.clone())
            .ok_or_else(|| PrayError::Resolution(format!("unknown source: {source_name}")))?
    };

    let lockfile_reference = lockfile
        .as_ref()
        .and_then(|lockfile| lockfile.file_hash().ok());
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PrayError::Resolution(error.to_string()))?
        .as_secs()
        .to_string();
    let mut confession = ConfessionSubmission {
        package: package_name.clone(),
        version: resolved_version,
        status: if accepted {
            "accepted".to_string()
        } else {
            "rejected".to_string()
        },
        note,
        lockfile: lockfile_reference,
        distribution_point: Some(source_url.clone()),
        signer: Some(current_signer()?),
        timestamp: Some(timestamp),
        signature: None,
    };
    let signature_payload =
        serde_json::to_vec(&confession).map_err(|error| PrayError::Manifest(error.to_string()))?;
    confession.signature = Some(sha256_prefixed(&signature_payload));
    submit_confession(&source_url, &confession)?;
    println!(
        "Confession submitted for {} {}",
        confession.package, confession.version
    );
    Ok(())
}
