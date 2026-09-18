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
    maybe_declare_publish_remote(&root, &distribution_root)?;
    Ok(())
}

fn maybe_declare_publish_remote(project_root: &Path, distribution_root: &Path) -> PrayResult<()> {
    let manifest_path = project_root.join("Prayfile");
    if !manifest_path.exists() {
        return Ok(());
    }
    let text = fs::read_to_string(&manifest_path)?;
    let manifest = pray_core::manifest::parse_manifest(&text)?;
    if !manifest.publish_remotes.is_empty() {
        return Ok(());
    }
    let relative = if distribution_root == project_root {
        ".".to_string()
    } else {
        distribution_root
            .strip_prefix(project_root)
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| "prayers".to_string())
    };
    let statement = format!("publish \"prayers\", path: \"{relative}\"");
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let insertion = lines
        .iter()
        .rposition(|line| line.trim_start().starts_with("source "))
        .map(|index| index + 1)
        .or_else(|| {
            lines
                .iter()
                .position(|line| line.trim_start().starts_with("prayfile "))
                .map(|index| index + 1)
        })
        .unwrap_or(1);
    lines.insert(insertion, statement);
    let mut updated = lines.join("\n");
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    fs::write(manifest_path, updated)?;
    Ok(())
}

fn repo_distribution_root(root: &Path) -> PathBuf {
    if root.file_name().and_then(|value| value.to_str()) == Some("prayers") {
        root.to_path_buf()
    } else {
        root.join("prayers")
    }
}
