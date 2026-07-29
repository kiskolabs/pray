use crate::resolve::ResolvedPackage;
use crate::{PrayError, PrayResult};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn reject_undeclared_dependencies(packages: &[ResolvedPackage]) -> PrayResult<()> {
    let declared: BTreeSet<String> = packages
        .iter()
        .map(|package| package.declaration.name.clone())
        .collect();
    let mut missing = BTreeSet::new();
    for package in packages {
        for dependency in &package.spec.dependencies {
            if dependency.optional {
                continue;
            }
            if !declared.contains(&dependency.name) {
                missing.insert(format!(
                    "{} -> {}",
                    package.declaration.name, dependency.name
                ));
            }
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    Err(PrayError::Resolution(format!(
        "undeclared package dependencies (declare them in Prayfile or make them optional): {}",
        missing.into_iter().collect::<Vec<_>>().join(", ")
    )))
}

pub(crate) fn reject_dependency_cycles(packages: &[ResolvedPackage]) -> PrayResult<()> {
    let mut edges = BTreeMap::new();
    for package in packages {
        edges.insert(
            package.declaration.name.clone(),
            package
                .spec
                .dependencies
                .iter()
                .map(|dependency| dependency.name.clone())
                .collect(),
        );
    }
    if let Some(cycle) = crate::dependency_graph::find_dependency_cycle(&edges) {
        return Err(PrayError::Resolution(format!(
            "dependency cycle detected: {}",
            cycle.join(" -> ")
        )));
    }
    Ok(())
}
