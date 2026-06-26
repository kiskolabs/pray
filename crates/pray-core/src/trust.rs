use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryTrustSettings {
    #[serde(default)]
    pub email_confirmation: EmailConfirmationPolicy,
    #[serde(default)]
    pub passkeys_enabled: bool,
    #[serde(default)]
    pub ssh_keys_enabled: bool,
    #[serde(default)]
    pub ssh_agent_signing_enabled: bool,
}

impl Default for RegistryTrustSettings {
    fn default() -> Self {
        Self {
            email_confirmation: EmailConfirmationPolicy::Required,
            passkeys_enabled: false,
            ssh_keys_enabled: false,
            ssh_agent_signing_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EmailConfirmationPolicy {
    #[default]
    Required,
    Optional,
    Disabled,
}

impl RegistryTrustSettings {
    pub fn email_confirmation_label(&self) -> &'static str {
        match self.email_confirmation {
            EmailConfirmationPolicy::Required => "required",
            EmailConfirmationPolicy::Optional => "optional",
            EmailConfirmationPolicy::Disabled => "disabled",
        }
    }

    pub fn passkeys_label(&self) -> &'static str {
        if self.passkeys_enabled {
            "enabled"
        } else {
            "disabled"
        }
    }

    pub fn ssh_keys_label(&self) -> &'static str {
        if self.ssh_keys_enabled {
            "enabled"
        } else {
            "disabled"
        }
    }

    pub fn ssh_agent_label(&self) -> &'static str {
        if self.ssh_agent_signing_enabled {
            "enabled"
        } else {
            "disabled"
        }
    }
}

pub fn read_registry_trust_settings(root: &Path) -> PrayResult<RegistryTrustSettings> {
    let path = root.join("v1/trust.json");
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(RegistryTrustSettings::default());
    };
    let settings: RegistryTrustSettings =
        serde_json::from_str(&text).map_err(|error| PrayError::Parse {
            kind: "registry trust settings",
            message: error.to_string(),
        })?;
    Ok(settings)
}

pub fn write_registry_trust_settings(
    root: &Path,
    settings: &RegistryTrustSettings,
) -> PrayResult<()> {
    let path = root.join("v1/trust.json");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_require_email_confirmation_and_disable_other_methods() {
        let settings = RegistryTrustSettings::default();
        assert_eq!(settings.email_confirmation_label(), "required");
        assert_eq!(settings.passkeys_label(), "disabled");
        assert_eq!(settings.ssh_keys_label(), "disabled");
        assert_eq!(settings.ssh_agent_label(), "disabled");
    }

    #[test]
    fn parses_optional_email_confirmation_and_key_methods() {
        let settings: RegistryTrustSettings = serde_json::from_str(
            r#"{
                "email_confirmation": "optional",
                "passkeys_enabled": true,
                "ssh_keys_enabled": true,
                "ssh_agent_signing_enabled": true
            }"#,
        )
        .expect("parse trust settings");

        assert_eq!(
            settings.email_confirmation,
            EmailConfirmationPolicy::Optional
        );
        assert_eq!(settings.email_confirmation_label(), "optional");
        assert_eq!(settings.passkeys_label(), "enabled");
        assert_eq!(settings.ssh_keys_label(), "enabled");
        assert_eq!(settings.ssh_agent_label(), "enabled");
    }
}
