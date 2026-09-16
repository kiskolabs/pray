use crate::package_spec::PackageSpec;
use crate::package_upstream::next_upstream_constraint;

#[path = "package_spec_serialize.rs"]
mod serialize;
pub use serialize::render_package_spec;

pub fn fork_spec_after_refresh(
    local: &PackageSpec,
    new_upstream: &PackageSpec,
    clean_replica: bool,
    merged_content_paths: &[String],
) -> PackageSpec {
    let mut spec = local.clone();
    spec.files = merged_content_paths.to_vec();
    if clean_replica {
        spec.exports = new_upstream.exports.clone();
        spec.templates = new_upstream.templates.clone();
    }
    if let Some(upstream) = spec.upstream.as_mut() {
        upstream.name = new_upstream.name.clone();
        upstream.constraint = next_upstream_constraint(&upstream.constraint, &new_upstream.version);
    }
    spec
}

#[cfg(test)]
#[path = "package_spec_render_tests.rs"]
mod tests;
