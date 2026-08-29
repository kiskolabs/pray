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
    let mut replaced = 0usize;
    for line in &mut lines {
        let trimmed = line.trim_start();
        if prefixes
            .iter()
            .any(|prefix| trimmed.starts_with(prefix.as_str()))
        {
            *line =
                crate::manifest_constraint::rewrite_constraint_on_line(line, &package.constraint)?;
            replaced += 1;
        }
    }
    if replaced == 0 {
        return Err(PrayError::Manifest(format!(
            "package {name} not found in manifest"
        )));
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ManifestPackage;

    fn package(constraint: &str) -> ManifestPackage {
        ManifestPackage {
            name: "sample/base".to_string(),
            constraint: constraint.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn rewrites_every_matching_declaration_and_keeps_indent() {
        let text = r#"
prayfile "1"
compose "AGENTS.md" do
  pray "sample/base", "~> 1.0"
end
tree ".agents/skills" do
  pray "sample/base", "~> 1.0", export: "testing-basics"
end
"#;
        let updated = replace_package_declaration(text, &package("~> 1.1")).expect("replace");
        assert!(updated.contains(r#"  pray "sample/base", "~> 1.1""#));
        assert!(updated.contains(r#"  pray "sample/base", "~> 1.1", export: "testing-basics""#));
        assert!(!updated.contains("~> 1.0"));
    }
}
