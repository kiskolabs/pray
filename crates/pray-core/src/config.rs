use crate::{PrayError, PrayResult};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct PrayConfig {
    #[serde(default)]
    pub local: PrayLocalConfig,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct PrayLocalConfig {
    #[serde(default)]
    pub package: BTreeMap<String, String>,
    #[serde(default)]
    pub source: BTreeMap<String, String>,
}

pub fn load_user_config() -> PrayResult<PrayConfig> {
    let Some(path) = user_config_path() else {
        return Ok(PrayConfig::default());
    };
    if !path.is_file() {
        return Ok(PrayConfig::default());
    }
    let text = fs::read_to_string(&path)?;
    toml::from_str(&text).map_err(|error| PrayError::Parse {
        kind: "config",
        message: format!("{}: {error}", path.display()),
    })
}

pub fn user_config_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("PRAY_CONFIG") {
        return Some(PathBuf::from(path));
    }
    if let Ok(home) = std::env::var("PRAY_HOME") {
        let path = Path::new(&home).join("config.toml");
        if path.is_file() {
            return Some(path);
        }
    }
    home_directory().map(|home| home.join(".config").join("pray").join("config.toml"))
}

fn home_directory() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::PrayConfig;

    #[test]
    fn parses_local_override_tables() {
        let config: PrayConfig = toml::from_str(
            r#"
[local.package]
"sample/base" = "../fork"

[local.source]
dist = "../distribution/prayers"
"#,
        )
        .expect("config");
        assert_eq!(
            config.local.package.get("sample/base").map(String::as_str),
            Some("../fork")
        );
        assert_eq!(
            config.local.source.get("dist").map(String::as_str),
            Some("../distribution/prayers")
        );
    }
}
