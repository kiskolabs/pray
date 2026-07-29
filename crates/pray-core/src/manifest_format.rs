use crate::manifest::ManifestPackage;
use crate::{PrayError, PrayResult};

pub fn format_package_declaration(package: &ManifestPackage) -> String {
    let mut parts = vec![format!("pray \"{}\"", package.name)];
    if package.constraint != "*" {
        parts.push(format!("\"{}\"", package.constraint));
    }
    if let Some(path) = &package.path {
        parts.push(format!("path: \"{path}\""));
    }
    if let Some(source) = &package.source {
        parts.push(format!("source: \"{source}\""));
    }
    if let Some(git) = &package.git {
        parts.push(format!("git: \"{git}\""));
    }
    if let Some(tag) = &package.tag {
        parts.push(format!("tag: \"{tag}\""));
    }
    if let Some(rev) = &package.rev {
        parts.push(format!("rev: \"{rev}\""));
    }
    if let Some(tarball) = &package.tarball {
        parts.push(format!("tarball: \"{tarball}\""));
    }
    if let Some(oci) = &package.oci {
        parts.push(format!("oci: \"{oci}\""));
    }
    if let Some(file) = &package.file {
        parts.push(format!("file: \"{file}\""));
    }
    if !package.exports.is_empty() {
        if package.exports.len() == 1 {
            parts.push(format!("export: \"{}\"", package.exports[0]));
        } else {
            parts.push(format!(
                "exports: [{}]",
                format_string_keyword_list(&package.exports)
            ));
        }
    }
    if !package.targets.is_empty() {
        parts.push(format!(
            "targets: [{}]",
            format_string_keyword_list(&package.targets)
        ));
    }
    if !package.features.is_empty() {
        parts.push(format!(
            "features: [{}]",
            format_string_keyword_list(&package.features)
        ));
    }
    if package.optional {
        parts.push("optional: true".to_string());
    }
    parts.join(", ")
}

pub fn replace_package_declaration(text: &str, package: &ManifestPackage) -> PrayResult<String> {
    let name = &package.name;
    let prefixes = [
        format!("pray \"{name}\""),
        format!("pray '{name}'"),
        format!("use \"{name}\""),
        format!("include \"{name}\""),
        format!("agent \"{name}\""),
        format!("agent '{name}'"),
        format!("package \"{name}\""),
        format!("package '{name}'"),
    ];
    let mut lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let index = lines
        .iter()
        .position(|line| {
            let trimmed = line.trim_start();
            prefixes
                .iter()
                .any(|prefix| trimmed.starts_with(prefix.as_str()))
        })
        .ok_or_else(|| PrayError::Manifest(format!("package {name} not found in manifest")))?;
    lines[index] = format_package_declaration(package);
    let mut output = lines.join("\n");
    if text.ends_with('\n') && !output.ends_with('\n') {
        output.push('\n');
    }
    Ok(output)
}

fn format_string_keyword_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ")
}
