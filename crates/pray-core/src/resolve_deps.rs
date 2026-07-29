use crate::resolve::ResolvedPackage;
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;

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
