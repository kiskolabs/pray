use crate::registry_http::{http_get, join_url};
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const DISTRIBUTION_CONFIG_SPEC: &str = "pray-distribution-config-1";
const TORRENT_PROTOCOL: &str = "torrent";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryDistributionSettings {
    #[serde(default = "distribution_config_spec")]
    pub spec: String,
    #[serde(default)]
    pub protocols: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bootstrap_trackers: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub enable_dht: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl Default for RegistryDistributionSettings {
    fn default() -> Self {
        Self {
            spec: distribution_config_spec(),
            protocols: Vec::new(),
            bootstrap_trackers: Vec::new(),
            enable_dht: false,
        }
    }
}

fn distribution_config_spec() -> String {
    DISTRIBUTION_CONFIG_SPEC.to_string()
}

impl RegistryDistributionSettings {
    pub fn allows_torrent(&self) -> bool {
        self.protocols.iter().any(|name| name == TORRENT_PROTOCOL)
    }

    fn validate(&self) -> PrayResult<()> {
        if self.spec != DISTRIBUTION_CONFIG_SPEC {
            return Err(parse_error(format!(
                "unsupported distribution config spec: {}",
                self.spec
            )));
        }
        for name in &self.protocols {
            if name != TORRENT_PROTOCOL {
                return Err(parse_error(format!("unsupported protocol: {name}")));
            }
        }
        if self.enable_dht {
            return Err(parse_error("DHT announce is not implemented".to_string()));
        }
        if !self.bootstrap_trackers.is_empty() && !self.allows_torrent() {
            return Err(parse_error(
                "bootstrap_trackers requires protocols to include torrent".to_string(),
            ));
        }
        Ok(())
    }
}

fn parse_error(message: String) -> PrayError {
    PrayError::Parse {
        kind: "distribution config",
        message,
    }
}

pub fn read_registry_distribution_settings(
    root: &Path,
) -> PrayResult<RegistryDistributionSettings> {
    let path = root.join("v1/distribution.json");
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(RegistryDistributionSettings::default());
    };
    parse_registry_distribution_settings(&text)
}

pub fn parse_registry_distribution_settings(
    text: &str,
) -> PrayResult<RegistryDistributionSettings> {
    let settings: RegistryDistributionSettings =
        serde_json::from_str(text).map_err(|error| parse_error(error.to_string()))?;
    settings.validate()?;
    Ok(settings)
}

pub fn write_registry_distribution_settings(
    root: &Path,
    settings: &RegistryDistributionSettings,
) -> PrayResult<()> {
    settings.validate()?;
    let path = root.join("v1/distribution.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(settings)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    )?;
    Ok(())
}

pub fn fetch_registry_distribution_settings(
    source_url: &str,
) -> PrayResult<RegistryDistributionSettings> {
    crate::registry_http_cache::registry_distribution_settings(source_url, || {
        let url = join_url(source_url, "v1/distribution.json");
        match http_get(&url) {
            Ok(bytes) => {
                let text =
                    String::from_utf8(bytes).map_err(|error| parse_error(error.to_string()))?;
                parse_registry_distribution_settings(&text)
            }
            Err(PrayError::Resolution(message) | PrayError::Network(message))
                if message.contains("HTTP 404") =>
            {
                Ok(RegistryDistributionSettings::default())
            }
            Err(error) => Err(error),
        }
    })
}
