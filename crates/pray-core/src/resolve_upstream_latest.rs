use super::super::{source_map, ResolvedPackage, ResolvedProject};
use super::lock::implied_upstream_source;
use super::resolution_context::UpstreamResolutionContext;
use crate::config::load_user_config;
use crate::constraint::{latest_constraint_for_package, version_satisfies};
use crate::lockfile::Lockfile;
use crate::package_spec::parse_package_spec;
use crate::package_spec_render::render_package_spec;
use crate::paths::find_prayspec_file;
use crate::resolve_context::ResolveOptions;
use crate::resolve_git_sources::prepare_git_sources;
use crate::{PrayError, PrayResult};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathUpstreamLatestConstraint {
    pub package_name: String,
    pub upstream_name: String,
    pub current_constraint: String,
    pub latest_version: String,
    pub new_constraint: String,
    pub package_root: PathBuf,
}

pub fn plan_path_upstream_latest_constraints(
    project: &ResolvedProject,
    previous: Option<&Lockfile>,
    selected: Option<&str>,
    options: &ResolveOptions,
) -> PrayResult<Vec<PathUpstreamLatestConstraint>> {
    let user_config = load_user_config()?;
    let git_sources = prepare_git_sources(
        &project.project_root,
        &project.manifest.sources,
        previous,
        options,
    )?;
    let sources = source_map(&project.manifest.sources);
    let context = UpstreamResolutionContext::new(
        &project.project_root,
        &sources,
        &git_sources,
        &user_config,
        previous,
        options,
    );
    let mut plans = Vec::new();
    for package in &project.packages {
        if selected.is_some_and(|name| name != package.declaration.name) {
            continue;
        }
        if let Some(plan) = plan_one_path_upstream(&context, package)? {
            plans.push(plan);
        }
    }
    Ok(plans)
}

pub fn apply_path_upstream_latest_constraints(
    plans: &[PathUpstreamLatestConstraint],
) -> PrayResult<()> {
    for plan in plans {
        let spec_path = find_prayspec_file(&plan.package_root)?;
        let text = fs::read_to_string(&spec_path)?;
        let mut spec = parse_package_spec(&text)?;
        let Some(upstream) = spec.upstream.as_mut() else {
            return Err(PrayError::Resolution(format!(
                "package {} has no upstream pin",
                plan.package_name
            )));
        };
        upstream.constraint = plan.new_constraint.clone();
        crate::transaction::write_file(&spec_path, render_package_spec(&spec))?;
    }
    Ok(())
}

fn plan_one_path_upstream(
    context: &UpstreamResolutionContext<'_>,
    package: &ResolvedPackage,
) -> PrayResult<Option<PathUpstreamLatestConstraint>> {
    let Some(upstream) = &package.spec.upstream else {
        return Ok(None);
    };
    if package.declaration.path.is_none() {
        return Ok(None);
    }
    let source = implied_upstream_source(&upstream.name, context.sources)?;
    let resolved = context.resolve(&upstream.name, "*", source.as_deref())?;
    if version_satisfies(&resolved.spec.version, &upstream.constraint)? {
        return Ok(None);
    }
    let new_constraint =
        latest_spec_upstream_constraint(&upstream.constraint, &resolved.spec.version)?;
    if !version_satisfies(&resolved.spec.version, &new_constraint)? {
        return Err(PrayError::Resolution(format!(
            "derived constraint {new_constraint} does not admit latest {} for {}",
            resolved.spec.version, package.declaration.name
        )));
    }
    Ok(Some(PathUpstreamLatestConstraint {
        package_name: package.declaration.name.clone(),
        upstream_name: upstream.name.clone(),
        current_constraint: upstream.constraint.clone(),
        latest_version: resolved.spec.version,
        new_constraint,
        package_root: package.root.clone(),
    }))
}

fn latest_spec_upstream_constraint(current: &str, latest_version: &str) -> PrayResult<String> {
    let derived = latest_constraint_for_package(current, latest_version)?;
    if derived.starts_with('=') {
        return Ok(format!("= {latest_version}"));
    }
    Ok(derived)
}

#[cfg(test)]
mod tests {
    use super::latest_spec_upstream_constraint;

    #[test]
    fn exact_pin_uses_spaced_equals() {
        assert_eq!(
            latest_spec_upstream_constraint("= 1.4.3", "1.4.4").expect("constraint"),
            "= 1.4.4"
        );
    }

    #[test]
    fn pessimistic_pin_follows_latest_family() {
        assert_eq!(
            latest_spec_upstream_constraint("~> 1.4", "2.0.0").expect("constraint"),
            "~> 2.0"
        );
    }
}
