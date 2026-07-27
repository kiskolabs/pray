use crate::destination::{
    bind_local_entry, bind_package_entry, export_kind_matches_role, new_destination_target,
};
use crate::format_serialize::{serialize_recommended, target_has_extras};
use crate::manifest::{
    DestinationMode, ExportRole, Manifest, ManifestLocal, ManifestPackage, ManifestTarget,
};
use crate::resolve::ResolvedProject;
use crate::{PrayError, PrayResult};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default)]
pub struct PackageFormatHint {
    pub roles: Vec<ExportRole>,
    pub file_path: Option<String>,
    /// Export names that must stay explicit after migration when a role is ambiguous.
    pub exports: Vec<String>,
}

pub fn uses_destination_dsl(manifest: &Manifest) -> bool {
    manifest
        .targets
        .iter()
        .any(|target| target.scoped || target.mode != DestinationMode::Legacy)
        || manifest
            .packages
            .iter()
            .any(|package| package.bound || package.file.is_some())
        || manifest.local.iter().any(|local| local.bound)
}

fn has_migratable_legacy_targets(manifest: &Manifest) -> bool {
    manifest.targets.iter().any(|target| {
        !target.scoped
            && target.mode == DestinationMode::Legacy
            && (!target.outputs.is_empty() || !target.skills.is_empty())
    })
}

pub fn classify_format_hints(project: &ResolvedProject) -> BTreeMap<String, PackageFormatHint> {
    let mut hints = BTreeMap::new();
    for package in &project.packages {
        let mut roles = Vec::new();
        let mut file_path = package.declaration.file.clone();
        for export_name in &package.selected_exports {
            let Some(export) = package.spec.exports.get(export_name) else {
                continue;
            };
            for role in [ExportRole::Fragment, ExportRole::Folder, ExportRole::File] {
                if export_kind_matches_role(&export.kind, role) && !roles.contains(&role) {
                    roles.push(role);
                }
            }
            if file_path.is_none() && export.kind == "file" {
                file_path = export
                    .default_path
                    .clone()
                    .or_else(|| Some(export_name.clone()));
            }
        }
        let exports =
            ambiguous_exports_for_roles(&package.selected_exports, &package.spec.exports, &roles);
        hints.insert(
            package.declaration.name.clone(),
            PackageFormatHint {
                roles,
                file_path,
                exports,
            },
        );
    }
    hints
}

fn ambiguous_exports_for_roles(
    selected_exports: &[String],
    exports: &BTreeMap<String, crate::package_spec::PackageExport>,
    roles: &[ExportRole],
) -> Vec<String> {
    let mut ambiguous = Vec::new();
    for role in roles {
        let matching: Vec<String> = selected_exports
            .iter()
            .filter(|export_name| {
                exports
                    .get(*export_name)
                    .is_some_and(|export| export_kind_matches_role(&export.kind, *role))
            })
            .cloned()
            .collect();
        if matching.len() > 1 {
            for export_name in matching {
                if !ambiguous.contains(&export_name) {
                    ambiguous.push(export_name);
                }
            }
        }
    }
    ambiguous
}

pub fn recommend_manifest(
    manifest: &Manifest,
    hints: &BTreeMap<String, PackageFormatHint>,
) -> Manifest {
    // Top-level file: bindings alone must not skip migration of legacy target blocks.
    let mut recommended = if has_migratable_legacy_targets(manifest) {
        migrate_legacy_manifest(manifest, hints)
    } else {
        manifest.clone()
    };
    omit_context_resolved_exports(&mut recommended);
    omit_default_sources(&mut recommended);
    recommended
}

fn omit_context_resolved_exports(manifest: &mut Manifest) {
    for package in &mut manifest.packages {
        if package.bound && package.exports.len() <= 1 {
            package.exports.clear();
        }
    }
}

fn package_namespace(name: &str) -> Option<&str> {
    name.split_once('/').map(|(namespace, _)| namespace)
}

fn omit_default_sources(manifest: &mut Manifest) {
    let sole_source = (manifest.sources.len() == 1).then(|| manifest.sources[0].name.clone());
    let source_names: BTreeSet<&str> = manifest
        .sources
        .iter()
        .map(|source| source.name.as_str())
        .collect();
    for package in &mut manifest.packages {
        let Some(source) = package.source.as_deref() else {
            continue;
        };
        let matches_sole = sole_source.as_deref() == Some(source);
        let matches_namespace =
            package_namespace(&package.name) == Some(source) && source_names.contains(source);
        if matches_sole || matches_namespace {
            package.source = None;
        }
    }
}

pub fn format_recommended(
    manifest: &Manifest,
    hints: &BTreeMap<String, PackageFormatHint>,
) -> PrayResult<String> {
    let recommended = recommend_manifest(manifest, hints);
    let text = serialize_recommended(&recommended);
    let reparsed = crate::manifest::parse_manifest(&text)?;
    if reparsed.canonicalized() != recommended.canonicalized() {
        return Err(PrayError::Manifest(
            "formatted Prayfile did not round-trip to an equivalent manifest".to_string(),
        ));
    }
    Ok(text)
}

fn migrate_legacy_manifest(
    manifest: &Manifest,
    hints: &BTreeMap<String, PackageFormatHint>,
) -> Manifest {
    let mut next = Manifest {
        prayfile_version: manifest.prayfile_version.clone(),
        sources: manifest.sources.clone(),
        targets: Vec::new(),
        packages: manifest.packages.clone(),
        local: manifest.local.clone(),
        symbols: manifest.symbols.clone(),
        render: manifest.render.clone(),
    };

    apply_format_hints(&mut next.packages, hints);

    let compose_paths = unique_paths(manifest.targets.iter().flat_map(|target| {
        target
            .outputs
            .iter()
            .map(move |path| (path.clone(), target.name.clone()))
    }));
    let tree_paths = unique_paths(manifest.targets.iter().flat_map(|target| {
        target
            .skills
            .iter()
            .map(move |path| (path.clone(), target.name.clone()))
    }));

    for (path, target_names) in &compose_paths {
        let mut target = new_destination_target(DestinationMode::Compose, path);
        for local in locals_for_compose(&next.local) {
            bind_local_entry(&mut target, &local.path);
            if let Some(entry) = next
                .local
                .iter_mut()
                .find(|candidate| candidate.path == local.path)
            {
                entry.bound = true;
            }
        }
        for package in packages_for_role(&next.packages, ExportRole::Fragment, target_names) {
            bind_package_entry(&mut target, &package.name);
            mark_package_bound(&mut next.packages, &package.name, ExportRole::Fragment);
        }
        next.targets.push(target);
    }

    for (path, target_names) in &tree_paths {
        let mut target = new_destination_target(DestinationMode::Tree, path);
        for package in packages_for_role(&next.packages, ExportRole::Folder, target_names) {
            bind_package_entry(&mut target, &package.name);
            mark_package_bound(&mut next.packages, &package.name, ExportRole::Folder);
        }
        next.targets.push(target);
    }

    for package in &mut next.packages {
        if package.file.is_some() {
            package.bound = true;
            if !package.roles.contains(&ExportRole::File) {
                package.roles.push(ExportRole::File);
            }
        }
    }
    for local in &mut next.local {
        if local.bound {
            // Compose entry order replaces legacy position keywords.
            local.position = "after".to_string();
        }
    }

    for target in &manifest.targets {
        if target_has_extras(target) {
            next.targets.push(ManifestTarget {
                name: target.name.clone(),
                outputs: Vec::new(),
                skills: Vec::new(),
                commands: target.commands.clone(),
                rules: target.rules.clone(),
                max_bytes: target.max_bytes,
                mode: DestinationMode::Legacy,
                scoped: false,
                entries: Vec::new(),
            });
        }
    }

    next
}

fn apply_format_hints(
    packages: &mut [ManifestPackage],
    hints: &BTreeMap<String, PackageFormatHint>,
) {
    for package in packages {
        if let Some(hint) = hints.get(&package.name) {
            for role in &hint.roles {
                if !package.roles.contains(role) {
                    package.roles.push(*role);
                }
            }
            if package.file.is_none() {
                package.file = hint.file_path.clone();
            }
            if package.exports.is_empty() && !hint.exports.is_empty() {
                package.exports = hint.exports.clone();
            }
        }
        if package.file.is_some() && !package.roles.contains(&ExportRole::File) {
            package.roles.push(ExportRole::File);
        }
    }
}

fn unique_paths(items: impl Iterator<Item = (String, String)>) -> Vec<(String, BTreeSet<String>)> {
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (path, target_name) in items {
        map.entry(path).or_default().insert(target_name);
    }
    map.into_iter().collect()
}

fn locals_for_compose(locals: &[ManifestLocal]) -> Vec<ManifestLocal> {
    let mut before = Vec::new();
    let mut after = Vec::new();
    for local in locals {
        if local.bound {
            continue;
        }
        match local.position.as_str() {
            "start" | "before" => before.push(local.clone()),
            _ => after.push(local.clone()),
        }
    }
    before.extend(after);
    before
}

fn packages_for_role(
    packages: &[ManifestPackage],
    role: ExportRole,
    target_names: &BTreeSet<String>,
) -> Vec<ManifestPackage> {
    packages
        .iter()
        .filter(|package| {
            if package.file.is_some() {
                return false;
            }
            if !package.targets.is_empty()
                && !package
                    .targets
                    .iter()
                    .any(|name| target_names.contains(name))
            {
                return false;
            }
            package.roles.contains(&role)
        })
        .cloned()
        .collect()
}

fn mark_package_bound(packages: &mut [ManifestPackage], name: &str, role: ExportRole) {
    if let Some(package) = packages.iter_mut().find(|package| package.name == name) {
        package.bound = true;
        if !package.roles.contains(&role) {
            package.roles.push(role);
        }
    }
}
