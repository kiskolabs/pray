use crate::project_paths::{default_output_for_target, manifest_path};
use crate::registry_ops::write_registry_index;
use pray_core::registry::RegistryIndex;
use pray_core::trust::{write_registry_trust_settings, RegistryTrustSettings};
use pray_core::{PrayError, PrayResult};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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

pub(crate) fn prayer_init_command() -> PrayResult<()> {
    let root = env::current_dir()?;
    let package_name = root
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("prayer-package")
        .to_string();
    let prayspec_path = root.join(format!("{package_name}.prayspec"));
    if prayspec_path.exists() {
        return Err(PrayError::Manifest(format!(
            "package spec already exists: {}",
            prayspec_path.display()
        )));
    }

    fs::write(
        &prayspec_path,
        format!(
            r#"Package::Specification.new do |spec|
  spec.name = "{package_name}"
  spec.version = "0.1.0"
  spec.summary = "Describe this package"
  spec.files = ["README.md"]
  spec.exports = {{}}
end
"#
        ),
    )?;
    if !root.join("README.md").exists() {
        fs::write(root.join("README.md"), format!("# {package_name}\n"))?;
    }
    fs::create_dir_all(root.join("exports"))?;
    Ok(())
}

pub(crate) fn repo_init_command() -> PrayResult<()> {
    let root = env::current_dir()?;
    let distribution_root = repo_distribution_root(&root);
    let index_path = distribution_root.join("v1/index.json");
    let trust_path = distribution_root.join("v1/trust.json");
    if index_path.exists() || trust_path.exists() {
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
    Ok(())
}

fn repo_distribution_root(root: &Path) -> PathBuf {
    if root.file_name().and_then(|value| value.to_str()) == Some("prayers") {
        root.to_path_buf()
    } else {
        root.join("prayers")
    }
}
