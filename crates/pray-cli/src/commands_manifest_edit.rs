use crate::commands_materialize::install_command;
use crate::project_paths::manifest_path;
use pray_core::manifest::{parse_manifest, read_manifest_text};
use pray_core::resolve_context::ResolveOptions;
use pray_core::{PrayError, PrayResult};
use std::fs;

pub(crate) fn add_command(name: String, constraint: Option<String>, path: Option<String>) -> PrayResult<()> {
    let manifest_path = manifest_path();
    let manifest_text = read_manifest_text(&manifest_path)?;
    let manifest = parse_manifest(&manifest_text)?;
    if manifest.packages.iter().any(|package| package.name == name) {
        return Err(PrayError::Manifest(format!(
            "package {name} already exists"
        )));
    }

    let declaration = if let Some(path) = path {
        if let Some(constraint) = constraint {
            format!("agent \"{name}\", \"{constraint}\", path: \"{path}\"")
        } else {
            format!("agent \"{name}\", path: \"{path}\"")
        }
    } else if let Some(constraint) = constraint {
        format!("agent \"{name}\", \"{constraint}\"")
    } else {
        format!("agent \"{name}\"")
    };

    fs::write(
        manifest_path,
        insert_manifest_statement(&manifest_text, &declaration),
    )?;
    Ok(())
}

pub(crate) fn remove_command(name: String) -> PrayResult<()> {
    let manifest_path = manifest_path();
    let manifest_text = read_manifest_text(&manifest_path)?;
    let manifest = parse_manifest(&manifest_text)?;
    if !manifest.packages.iter().any(|package| package.name == name) {
        return Err(PrayError::Manifest(format!("package {name} not found")));
    }

    fs::write(
        manifest_path,
        remove_manifest_statement(&manifest_text, &name),
    )?;
    install_command(false, false, ResolveOptions::default(), false)?;
    Ok(())
}

fn insert_manifest_statement(text: &str, statement: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let insertion_index = lines
        .iter()
        .position(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("local ") || trimmed.starts_with("render ")
        })
        .unwrap_or(lines.len());
    lines.insert(insertion_index, statement.to_string());
    let mut output = lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn remove_manifest_statement(text: &str, name: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let package_prefix = format!("agent \"{name}\"");
    if let Some(index) = lines.iter().position(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with(&package_prefix) || trimmed.starts_with(&format!("agent '{name}'"))
    }) {
        lines.remove(index);
        if index < lines.len() && lines[index].trim().is_empty() {
            lines.remove(index);
        } else if index > 0 && lines[index - 1].trim().is_empty() {
            lines.remove(index - 1);
        }
    }
    let mut output = lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}
