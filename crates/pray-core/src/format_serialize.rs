use crate::manifest::{
    format_package_declaration, DestinationEntry, DestinationMode, Manifest, ManifestPackage,
    ManifestSource, ManifestTarget, RenderPolicy,
};

pub(crate) fn serialize_recommended(manifest: &Manifest) -> String {
    let mut lines = Vec::new();
    lines.push(format!("prayfile \"{}\"", manifest.prayfile_version));

    if !manifest.sources.is_empty() {
        lines.push(String::new());
        for source in &manifest.sources {
            lines.push(format_source(source));
        }
    }

    for target in &manifest.targets {
        if !target.scoped {
            continue;
        }
        lines.push(String::new());
        match target.mode {
            DestinationMode::Compose => {
                let path = target.outputs.first().map(String::as_str).unwrap_or("");
                lines.push(format!("compose \"{path}\" do"));
                for entry in &target.entries {
                    lines.push(format!("  {}", format_destination_entry(entry, manifest)));
                }
                lines.push("end".to_string());
            }
            DestinationMode::Tree => {
                let path = target.skills.first().map(String::as_str).unwrap_or("");
                lines.push(format!("tree \"{path}\" do"));
                for entry in &target.entries {
                    if let DestinationEntry::Package { name } = entry {
                        if let Some(package) = find_package(manifest, name) {
                            lines.push(format!("  {}", format_package_declaration(package)));
                        }
                    }
                }
                lines.push("end".to_string());
            }
            DestinationMode::Legacy => {}
        }
    }

    let file_packages: Vec<&ManifestPackage> = manifest
        .packages
        .iter()
        .filter(|package| package.file.is_some())
        .collect();
    if !file_packages.is_empty() {
        lines.push(String::new());
        for package in file_packages {
            lines.push(format_package_declaration(package));
        }
    }

    let unbound: Vec<&ManifestPackage> = manifest
        .packages
        .iter()
        .filter(|package| !package.bound && package.file.is_none() && package.groups.is_empty())
        .collect();
    if !unbound.is_empty() {
        lines.push(String::new());
        for package in unbound {
            lines.push(format_package_declaration(package));
        }
    }

    for (group_names, packages) in grouped_packages(manifest) {
        lines.push(String::new());
        lines.push(format!(
            "group {} do",
            group_names
                .iter()
                .map(|name| format!(":{name}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        for package in packages {
            lines.push(format!("  {}", format_package_declaration(package)));
        }
        lines.push("end".to_string());
    }

    for target in &manifest.targets {
        if target.scoped || !target_has_extras(target) {
            continue;
        }
        lines.push(String::new());
        lines.push(format!("target :{} do", target.name));
        for command in &target.commands {
            lines.push(format!("  commands \"{command}\""));
        }
        for rule in &target.rules {
            lines.push(format!("  rules \"{rule}\""));
        }
        if let Some(max_bytes) = target.max_bytes {
            lines.push(format!("  max_bytes {max_bytes}"));
        }
        lines.push("end".to_string());
    }

    if manifest.render != RenderPolicy::default() {
        lines.push(String::new());
        lines.push(format!(
            "render mode: :{}, conflict: :{}, churn: :{}",
            manifest.render.mode, manifest.render.conflict, manifest.render.churn
        ));
    }

    lines.push(String::new());
    lines.join("\n")
}

pub(crate) fn target_has_extras(target: &ManifestTarget) -> bool {
    !target.commands.is_empty() || !target.rules.is_empty() || target.max_bytes.is_some()
}

fn format_source(source: &ManifestSource) -> String {
    let mut parts = vec![format!("source \"{}\"", source.name)];
    match source.kind.as_str() {
        "path" => parts.push(format!("path: \"{}\"", source.url)),
        "git" => {
            let url = source.url.strip_prefix("git+").unwrap_or(&source.url);
            parts.push(format!("git: \"{url}\""));
        }
        _ => parts.push(format!("\"{}\"", source.url)),
    }
    if let Some(subdir) = &source.subdir {
        parts.push(format!("distribution: \"{subdir}\""));
    }
    if let Some(tag) = &source.tag {
        parts.push(format!("tag: \"{tag}\""));
    }
    if let Some(rev) = &source.rev {
        parts.push(format!("rev: \"{rev}\""));
    }
    parts.join(", ")
}

fn format_destination_entry(entry: &DestinationEntry, manifest: &Manifest) -> String {
    match entry {
        DestinationEntry::Local { path } => format!("pray \"{path}\""),
        DestinationEntry::Package { name } => find_package(manifest, name)
            .map(format_package_declaration)
            .unwrap_or_else(|| format!("pray \"{name}\"")),
    }
}

fn find_package<'a>(manifest: &'a Manifest, name: &str) -> Option<&'a ManifestPackage> {
    manifest.packages.iter().find(|package| package.name == name)
}

fn grouped_packages(manifest: &Manifest) -> Vec<(Vec<String>, Vec<&ManifestPackage>)> {
    let mut groups: std::collections::BTreeMap<Vec<String>, Vec<&ManifestPackage>> =
        std::collections::BTreeMap::new();
    for package in &manifest.packages {
        if package.groups.is_empty() {
            continue;
        }
        groups
            .entry(package.groups.clone())
            .or_default()
            .push(package);
    }
    groups.into_iter().collect()
}
