use crate::project_paths::{default_output_for_target, manifest_path};
use crate::registry_ops::write_registry_index;
use pray_core::distribution::{write_registry_distribution_settings, RegistryDistributionSettings};
use pray_core::registry::RegistryIndex;
use pray_core::trust::{write_registry_trust_settings, RegistryTrustSettings};
use pray_core::{PrayError, PrayResult};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) use crate::commands_init_prayer::prayer_init_command;

pub(crate) fn init_command(targets: Vec<String>) -> PrayResult<()> {
    let manifest_path = manifest_path();
    if manifest_path.exists() {
        return Err(PrayError::Manifest("Prayfile already exists".to_string()));
    }
    let mut text = String::new();
    text.push_str("prayfile \"1\"\n");
    for target in if targets.is_empty() {
        vec!["tool_a".to_string()]
    } else {
        targets
    } {
        text.push_str(&format!(
            "target :{} do\n  output \"{}.md\"\nend\n",
            target,
            default_output_for_target(&target)
        ));
    }
    fs::write(manifest_path, text)?;
    Ok(())
}

pub(crate) fn repo_init_command() -> PrayResult<()> {
    let root = env::current_dir()?;
    let distribution_root = repo_distribution_root(&root);
    let index_path = distribution_root.join("v1/index.json");
    let trust_path = distribution_root.join("v1/trust.json");
    let distribution_path = distribution_root.join("v1/distribution.json");
    if index_path.exists() || trust_path.exists() || distribution_path.exists() {
        return Err(PrayError::Manifest(
            "distribution repo already exists".to_string(),
        ));
    }

    fs::create_dir_all(distribution_root.join("v1/packages"))?;
    fs::create_dir_all(distribution_root.join("v1/artifacts"))?;
    write_registry_index(
        &distribution_root,
        &RegistryIndex {
            spec: "prayfile-distribution-1".to_string(),
            packages: Vec::new(),
        },
    )?;
    write_registry_trust_settings(&distribution_root, &RegistryTrustSettings::default())?;
    write_registry_distribution_settings(
        &distribution_root,
        &RegistryDistributionSettings::default(),
    )?;
    Ok(())
}

fn repo_distribution_root(root: &Path) -> PathBuf {
    if root.file_name().and_then(|value| value.to_str()) == Some("prayers") {
        root.to_path_buf()
    } else {
        root.join("prayers")
    }
}
