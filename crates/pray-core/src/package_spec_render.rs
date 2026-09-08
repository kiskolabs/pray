use crate::package_spec::PackageSpec;
use crate::package_upstream::next_upstream_constraint;

#[path = "package_spec_serialize.rs"]
mod serialize;
pub use serialize::render_package_spec;

pub fn fork_spec_after_refresh(
    local: &PackageSpec,
    new_upstream: &PackageSpec,
    local_prayspec_file: &str,
    clean_replica: bool,
    merged_content_paths: &[String],
) -> PackageSpec {
    let mut spec = local.clone();
    let mut files = vec![local_prayspec_file.to_string()];
    files.extend(merged_content_paths.iter().cloned());
    spec.files = files;
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
