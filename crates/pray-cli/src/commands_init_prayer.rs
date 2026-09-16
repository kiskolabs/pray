use crate::commands_init_prayer_manifest::{declare_local_prayer, resolve_local_prayer_directory};
use crate::project_paths::{manifest_path, workspace_root};
use pray_core::local_prayer::{validate_local_prayer_name, DEFAULT_LOCAL_PRAYER_NAME};
use pray_core::{PrayError, PrayResult};
use std::env;
use std::fs;
use std::path::Path;

pub(crate) fn prayer_init_command(
    name: Option<String>,
    directory: Option<String>,
) -> PrayResult<()> {
    if manifest_path().exists() {
        scaffold_local_prayer(name, directory)
    } else {
        scaffold_standalone_package(name)
    }
}

fn scaffold_local_prayer(name: Option<String>, directory: Option<String>) -> PrayResult<()> {
    let package_name =
        validate_local_prayer_name(name.as_deref().unwrap_or(DEFAULT_LOCAL_PRAYER_NAME))?;
    let source = resolve_local_prayer_directory(directory.as_deref())?;
    let root = workspace_root().join(&source.directory).join(package_name);
    let prayspec_path = root.join(format!("{package_name}.prayspec"));
    if prayspec_path.exists() {
        return Err(PrayError::Manifest(format!(
            "package spec already exists: {}",
            prayspec_path.display()
        )));
    }
    if root.exists() {
        return Err(PrayError::Manifest(format!(
            "prayer directory already exists: {}",
            root.display()
        )));
    }

    fs::create_dir_all(root.join("exports"))?;
    fs::write(&prayspec_path, local_prayspec(&source.name, package_name))?;
    fs::write(root.join("README.md"), format!("# {package_name}\n"))?;
    fs::write(
        root.join(format!("exports/{package_name}.md")),
        format!("# {package_name}\n"),
    )?;
    declare_local_prayer(&format!("{}/{}", source.name, package_name))
}

fn scaffold_standalone_package(name: Option<String>) -> PrayResult<()> {
    let root = env::current_dir()?;
    let package_name = match name {
        Some(value) => validate_local_prayer_name(&value)?.to_string(),
        None => root
            .file_name()
            .and_then(|value| value.to_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("prayer-package")
            .to_string(),
    };
    write_standalone_scaffold(&root, &package_name)
}

fn write_standalone_scaffold(root: &Path, package_name: &str) -> PrayResult<()> {
    let prayspec_path = root.join(format!("{package_name}.prayspec"));
    if prayspec_path.exists() {
        return Err(PrayError::Manifest(format!(
            "package spec already exists: {}",
            prayspec_path.display()
        )));
    }
    fs::write(&prayspec_path, standalone_prayspec(package_name))?;
    if !root.join("README.md").exists() {
        fs::write(root.join("README.md"), format!("# {package_name}\n"))?;
    }
    fs::create_dir_all(root.join("exports"))?;
    Ok(())
}

fn local_prayspec(source_name: &str, name: &str) -> String {
    format!(
        r#"Package::Specification.new do |spec|
  spec.name = "{source_name}/{name}"
  spec.summary = "Describe this package"
  spec.files = ["README.md", "exports/{name}.md"]
  spec.exports = {{
    "{name}" => {{
      type: "fragment",
      path: "exports/{name}.md"
    }}
  }}
end
"#
    )
}

fn standalone_prayspec(name: &str) -> String {
    format!(
        r#"Package::Specification.new do |spec|
  spec.name = "{name}"
  spec.version = "0.1.0"
  spec.summary = "Describe this package"
  spec.files = ["README.md"]
  spec.exports = {{}}
end
"#
    )
}
