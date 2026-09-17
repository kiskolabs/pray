use crate::distribution::RegistryDistributionSettings;
use crate::registry::RegistryPackageMetadata;
use crate::registry_http::{http_get, join_url};
use crate::{PrayError, PrayResult};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

type MetadataKey = (String, String);

fn metadata_cache() -> &'static Mutex<HashMap<MetadataKey, RegistryPackageMetadata>> {
    static CACHE: OnceLock<Mutex<HashMap<MetadataKey, RegistryPackageMetadata>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn distribution_cache() -> &'static Mutex<HashMap<String, RegistryDistributionSettings>> {
    static CACHE: OnceLock<Mutex<HashMap<String, RegistryDistributionSettings>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn fetch_package_metadata(
    source_url: &str,
    package_name: &str,
) -> PrayResult<RegistryPackageMetadata> {
    let key = (source_url.to_string(), package_name.to_string());
    if let Ok(guard) = metadata_cache().lock() {
        if let Some(cached) = guard.get(&key) {
            return Ok(cached.clone());
        }
    }
    let url = join_url(source_url, &format!("v1/packages/{}.json", package_name));
    let response = http_get(&url)?;
    let metadata: RegistryPackageMetadata =
        serde_json::from_slice(&response).map_err(|error| PrayError::Parse {
            kind: "registry metadata",
            message: error.to_string(),
        })?;
    if let Ok(mut guard) = metadata_cache().lock() {
        guard.insert(key, metadata.clone());
    }
    Ok(metadata)
}

pub(crate) fn registry_distribution_settings(
    source_url: &str,
    fetch: impl FnOnce() -> PrayResult<RegistryDistributionSettings>,
) -> PrayResult<RegistryDistributionSettings> {
    if let Ok(guard) = distribution_cache().lock() {
        if let Some(cached) = guard.get(source_url) {
            return Ok(cached.clone());
        }
    }
    let settings = fetch()?;
    if let Ok(mut guard) = distribution_cache().lock() {
        guard.insert(source_url.to_string(), settings.clone());
    }
    Ok(settings)
}
