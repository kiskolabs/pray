use crate::resolve::ResolvedPackage;
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;

pub(crate) fn reject_dependency_cycles(packages: &[ResolvedPackage]) -> PrayResult<()> {
    if packages.iter().all(|package| {
        package
            .spec
            .dependencies
            .iter()
            .all(|dependency| dependency.optional)
    }) {
        return Ok(());
    }
    let mut edges = BTreeMap::new();
    for package in packages {
        edges.insert(
            package.declaration.name.clone(),
            package
                .spec
                .dependencies
                .iter()
                .filter(|dependency| !dependency.optional)
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

pub(crate) fn validate_optional_referrals(packages: &[ResolvedPackage]) -> PrayResult<()> {
    if packages.iter().all(|package| {
        package
            .spec
            .dependencies
            .iter()
            .all(|dependency| !dependency.optional)
    }) {
        return Ok(());
    }
    let packages_by_name: BTreeMap<_, _> = packages
        .iter()
        .map(|package| (package.declaration.name.as_str(), package))
        .collect();
    for package in packages {
        for dependency in package
            .spec
            .dependencies
            .iter()
            .filter(|dependency| dependency.optional)
        {
            let Some(target) = packages_by_name.get(dependency.name.as_str()) else {
                continue;
            };
            if target
                .spec
                .satisfy_constraint(&dependency.constraint)
                .is_err()
            {
                return Err(PrayError::Resolution(format!(
                    "optional referral {} -> {} requires {}, but selected version is {}",
                    package.declaration.name,
                    dependency.name,
                    dependency.constraint,
                    target.spec.recorded_version()
                )));
            }
        }
    }
    Ok(())
}
