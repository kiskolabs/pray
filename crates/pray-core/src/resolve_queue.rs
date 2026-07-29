use crate::constraint::version_satisfies;
use crate::manifest::{ManifestPackage, ManifestSource};
use crate::package_spec::PackageDependency;
use crate::resolve::ResolvedPackage;
use crate::{PrayError, PrayResult};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(crate) struct ResolveAllOutcome {
    pub packages: Vec<ResolvedPackage>,
    pub errors: Vec<String>,
    pub saw_network_error: bool,
}

#[derive(Debug, Default)]
pub(crate) struct ResolveQueue {
    pub order: VecDeque<String>,
    pub declarations: BTreeMap<String, ManifestPackage>,
    pub explicit: BTreeSet<String>,
    pub constraints: BTreeMap<String, String>,
}

impl ResolveQueue {
    pub fn seed(packages: &[ManifestPackage]) -> PrayResult<Self> {
        let mut queue = Self::default();
        for declaration in packages {
            queue.push_declaration(declaration.clone(), true)?;
        }
        Ok(queue)
    }

    pub fn push_declaration(
        &mut self,
        declaration: ManifestPackage,
        explicit: bool,
    ) -> PrayResult<()> {
        let name = declaration.name.clone();
        let constraint = merge_constraint(
            self.constraints
                .get(&name)
                .map(String::as_str)
                .unwrap_or("*"),
            &declaration.constraint,
        )?;
        self.constraints.insert(name.clone(), constraint.clone());
        if explicit {
            self.explicit.insert(name.clone());
        }
        if let Some(existing) = self.declarations.get_mut(&name) {
            existing.constraint = constraint;
            if existing.source.is_none() {
                existing.source = declaration.source;
            }
            if existing.path.is_none() {
                existing.path = declaration.path;
            }
        } else {
            let mut declaration = declaration;
            declaration.constraint = constraint;
            self.declarations.insert(name.clone(), declaration);
            self.order.push_back(name);
        }
        Ok(())
    }

    pub fn resolve_all(
        &mut self,
        manifest_packages: &[ManifestPackage],
        sources: &BTreeMap<String, ManifestSource>,
        mut resolve_one: impl FnMut(&ManifestPackage) -> PrayResult<ResolvedPackage>,
    ) -> ResolveAllOutcome {
        let mut resolved_by_name = BTreeMap::new();
        let mut errors = Vec::new();
        let mut saw_network_error = false;
        while let Some(name) = self.order.pop_front() {
            let Some(declaration) = self.declarations.get(&name).cloned() else {
                continue;
            };
            if let Some(existing) = resolved_by_name.get(&name) {
                if let Some(constraint) = self.constraints.get(&name) {
                    if let Err(error) = ensure_resolved_satisfies(existing, constraint) {
                        errors.push(format!("{name}: {error}"));
                    }
                }
                continue;
            }
            match resolve_one(&declaration) {
                Ok(mut package) => {
                    package.explicit = self.explicit.contains(&name);
                    if let Err(error) = self.enqueue_required_dependencies(&package, sources) {
                        errors.push(format!("{name}: {error}"));
                        continue;
                    }
                    resolved_by_name.insert(name, package);
                }
                Err(error) => {
                    if matches!(error, PrayError::Network(_)) {
                        saw_network_error = true;
                    }
                    errors.push(format!("{name}: {error}"));
                }
            }
        }
        ResolveAllOutcome {
            packages: order_packages(manifest_packages, resolved_by_name),
            errors,
            saw_network_error,
        }
    }

    pub fn enqueue_required_dependencies(
        &mut self,
        parent: &ResolvedPackage,
        sources: &BTreeMap<String, ManifestSource>,
    ) -> PrayResult<()> {
        for dependency in &parent.spec.dependencies {
            if dependency.optional {
                continue;
            }
            self.enqueue_dependency(parent, dependency, sources)?;
        }
        Ok(())
    }

    fn enqueue_dependency(
        &mut self,
        parent: &ResolvedPackage,
        dependency: &PackageDependency,
        sources: &BTreeMap<String, ManifestSource>,
    ) -> PrayResult<()> {
        if self.declarations.contains_key(&dependency.name) {
            // Prayfile-declared packages keep their manifest constraint.
            if self.explicit.contains(&dependency.name) {
                return Ok(());
            }
            let merged = merge_constraint(
                self.constraints
                    .get(&dependency.name)
                    .map(String::as_str)
                    .unwrap_or("*"),
                &dependency.constraint,
            )?;
            self.constraints
                .insert(dependency.name.clone(), merged.clone());
            if let Some(existing) = self.declarations.get_mut(&dependency.name) {
                existing.constraint = merged;
            }
            return Ok(());
        }
        self.push_declaration(
            synthetic_dependency_declaration(parent, dependency, sources)?,
            false,
        )
    }
}

fn order_packages(
    manifest_packages: &[ManifestPackage],
    mut resolved_by_name: BTreeMap<String, ResolvedPackage>,
) -> Vec<ResolvedPackage> {
    let mut packages = Vec::with_capacity(resolved_by_name.len());
    for declaration in manifest_packages {
        if let Some(package) = resolved_by_name.remove(&declaration.name) {
            packages.push(package);
        }
    }
    packages.extend(resolved_by_name.into_values());
    packages
}

pub(crate) fn ensure_resolved_satisfies(
    package: &ResolvedPackage,
    constraint: &str,
) -> PrayResult<()> {
    if version_satisfies(&package.spec.version, constraint)? {
        return Ok(());
    }
    Err(PrayError::Resolution(format!(
        "package {} version {} does not satisfy merged constraint {}",
        package.declaration.name, package.spec.version, constraint
    )))
}

fn merge_constraint(existing: &str, incoming: &str) -> PrayResult<String> {
    if existing == "*" || existing.is_empty() {
        return Ok(incoming.to_string());
    }
    if incoming == "*" || incoming.is_empty() {
        return Ok(existing.to_string());
    }
    if existing == incoming {
        return Ok(existing.to_string());
    }
    Ok(format!("{existing},{incoming}"))
}

fn synthetic_dependency_declaration(
    parent: &ResolvedPackage,
    dependency: &PackageDependency,
    sources: &BTreeMap<String, ManifestSource>,
) -> PrayResult<ManifestPackage> {
    if let Some(source) = &parent.declaration.source {
        return Ok(ManifestPackage {
            name: dependency.name.clone(),
            constraint: dependency.constraint.clone(),
            source: Some(source.clone()),
            ..ManifestPackage::default()
        });
    }
    if let Some(parent_path) = &parent.declaration.path {
        let leaf = dependency
            .name
            .rsplit('/')
            .next()
            .unwrap_or(dependency.name.as_str());
        let sibling = std::path::Path::new(parent_path)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(leaf);
        return Ok(ManifestPackage {
            name: dependency.name.clone(),
            constraint: dependency.constraint.clone(),
            path: Some(sibling.to_string_lossy().into_owned()),
            ..ManifestPackage::default()
        });
    }
    if sources.len() == 1 {
        return Ok(ManifestPackage {
            name: dependency.name.clone(),
            constraint: dependency.constraint.clone(),
            source: sources.keys().next().cloned(),
            ..ManifestPackage::default()
        });
    }
    Err(PrayError::Resolution(format!(
        "cannot resolve transitive dependency {} of {}; declare a source or path for the parent",
        dependency.name, parent.declaration.name
    )))
}
