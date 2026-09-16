use crate::manifest::{
    DestinationEntry, DestinationMode, ExportRole, Manifest, ManifestLocal, ManifestPackage,
    ManifestTarget,
};
use crate::{PrayError, PrayResult};

pub fn is_local_path_form(value: &str) -> bool {
    value.starts_with('.')
        || value.starts_with('/')
        || value.ends_with(".md")
        || value.ends_with(".txt")
        || value.ends_with(".markdown")
}

pub fn destination_target_name(mode: DestinationMode, path: &str) -> String {
    let prefix = match mode {
        DestinationMode::Compose => "compose",
        DestinationMode::Tree => "tree",
        DestinationMode::Legacy => "legacy",
    };
    format!("{prefix}:{path}")
}

pub fn new_destination_target(mode: DestinationMode, path: &str) -> ManifestTarget {
    let mut target = ManifestTarget {
        name: destination_target_name(mode, path),
        mode,
        scoped: true,
        ..ManifestTarget::default()
    };
    match mode {
        DestinationMode::Compose => target.outputs.push(path.to_string()),
        DestinationMode::Tree => target.skills.push(path.to_string()),
        DestinationMode::Legacy => {}
    }
    target
}

pub fn upsert_package(manifest: &mut Manifest, package: ManifestPackage) -> PrayResult<()> {
    if let Some(existing) = manifest
        .packages
        .iter_mut()
        .find(|candidate| candidate.name == package.name)
    {
        if existing.constraint != package.constraint
            && existing.constraint != "*"
            && package.constraint != "*"
        {
            return Err(PrayError::Manifest(format!(
                "package {} declared with conflicting constraints ({} vs {})",
                package.name, existing.constraint, package.constraint
            )));
        }
        if existing.constraint == "*" && package.constraint != "*" {
            existing.constraint = package.constraint;
        }
        if existing.source.is_none() {
            existing.source = package.source;
        } else if package.source.is_some() && existing.source != package.source {
            return Err(PrayError::Manifest(format!(
                "package {} declared with conflicting sources",
                package.name
            )));
        }
        for export in package.exports {
            if !existing.exports.contains(&export) {
                existing.exports.push(export);
            }
        }
        for role in package.roles {
            if !existing.roles.contains(&role) {
                existing.roles.push(role);
            }
        }
        if package.file.is_some() {
            if existing.file.is_some() && existing.file != package.file {
                return Err(PrayError::Manifest(format!(
                    "package {} declared with conflicting file: destinations",
                    package.name
                )));
            }
            existing.file = package.file;
        }
        existing.bound = existing.bound || package.bound;
        existing.optional = existing.optional || package.optional;
        if existing.path.is_none() {
            existing.path = package.path;
        }
        if existing.git.is_none() {
            existing.git = package.git;
        }
        if existing.tag.is_none() {
            existing.tag = package.tag;
        }
        if existing.rev.is_none() {
            existing.rev = package.rev;
        }
        if existing.tarball.is_none() {
            existing.tarball = package.tarball;
        }
        if existing.oci.is_none() {
            existing.oci = package.oci;
        }
        for group in package.groups {
            if !existing.groups.contains(&group) {
                existing.groups.push(group);
            }
        }
        return Ok(());
    }
    manifest.packages.push(package);
    Ok(())
}

pub fn upsert_local(manifest: &mut Manifest, local: ManifestLocal) {
    if let Some(existing) = manifest
        .local
        .iter_mut()
        .find(|candidate| candidate.path == local.path)
    {
        existing.bound = existing.bound || local.bound;
        existing.optional = existing.optional || local.optional;
        if existing.position == "after" && local.position != "after" {
            existing.position = local.position;
        }
        if existing.file.is_none() {
            existing.file = local.file;
        }
        return;
    }
    manifest.local.push(local);
}

pub fn bind_package_entry(target: &mut ManifestTarget, package_name: &str) {
    let entry = DestinationEntry::Package {
        name: package_name.to_string(),
    };
    if !target.entries.contains(&entry) {
        target.entries.push(entry);
    }
}

pub fn bind_local_entry(target: &mut ManifestTarget, path: &str) {
    let entry = DestinationEntry::Local {
        path: path.to_string(),
    };
    if !target.entries.contains(&entry) {
        target.entries.push(entry);
    }
}

pub fn role_for_destination(mode: DestinationMode) -> Option<ExportRole> {
    match mode {
        DestinationMode::Compose => Some(ExportRole::Fragment),
        DestinationMode::Tree => Some(ExportRole::Folder),
        DestinationMode::Legacy => None,
    }
}

pub fn package_bound_to_compose(package: &ManifestPackage, target: &ManifestTarget) -> bool {
    if target.scoped && target.mode == DestinationMode::Compose {
        return target.entries.iter().any(|entry| match entry {
            DestinationEntry::Package { name } => name == &package.name,
            DestinationEntry::Local { .. } => false,
        });
    }
    if package.bound || package.file.is_some() {
        return false;
    }
    true
}

pub fn package_bound_to_tree(package: &ManifestPackage, target: &ManifestTarget) -> bool {
    if target.scoped && target.mode == DestinationMode::Tree {
        return target.entries.iter().any(|entry| match entry {
            DestinationEntry::Package { name } => name == &package.name,
            DestinationEntry::Local { .. } => false,
        });
    }
    if package.bound || package.file.is_some() {
        return false;
    }
    true
}

pub fn export_kind_matches_role(kind: &str, role: ExportRole) -> bool {
    match role {
        ExportRole::Fragment => kind == "fragment",
        ExportRole::Folder => matches!(kind, "folder" | "skill"),
        ExportRole::File => kind == "file",
    }
}

pub fn local_is_compose_embed(manifest: &Manifest, local: &ManifestLocal) -> bool {
    if local.file.is_some() {
        return false;
    }
    !manifest.targets.iter().any(|target| {
        target.mode == DestinationMode::Tree
            && target.entries.iter().any(|entry| match entry {
                DestinationEntry::Local { path } => path == &local.path,
                DestinationEntry::Package { .. } => false,
            })
    })
}

pub fn try_apply_local_pray(
    manifest: &mut Manifest,
    path: &str,
    file_dest: Option<&str>,
    other_package_signal: bool,
    destination_index: Option<usize>,
) -> PrayResult<bool> {
    if !is_local_path_form(path) {
        return Ok(false);
    }
    if file_dest.is_some() {
        if other_package_signal {
            return Ok(false);
        }
        return Err(PrayError::Parse {
            kind: "manifest",
            message: "a local file cannot use file:".to_string(),
        });
    }
    if other_package_signal {
        return Ok(false);
    }
    let Some(index) = destination_index else {
        return Err(PrayError::Parse {
            kind: "manifest",
            message: "local pray paths are only valid inside compose".to_string(),
        });
    };
    let mode = manifest.targets[index].mode;
    if mode != DestinationMode::Compose {
        return Err(PrayError::Parse {
            kind: "manifest",
            message: "local pray paths are only valid inside compose".to_string(),
        });
    }
    let local = ManifestLocal {
        path: path.to_string(),
        position: "after".to_string(),
        optional: false,
        bound: true,
        file: None,
    };
    bind_local_entry(&mut manifest.targets[index], &local.path);
    upsert_local(manifest, local);
    Ok(true)
}
