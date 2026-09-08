use crate::literal::LiteralValue;
use crate::package_spec::PackageSpec;
use std::collections::BTreeMap;

#[path = "package_spec_serialize_maps.rs"]
mod maps;

pub fn render_package_spec(spec: &PackageSpec) -> String {
    let mut lines = vec!["Package::Specification.new do |spec|".to_string()];
    push_assignment(&mut lines, "name", &spec.name);
    push_assignment(&mut lines, "version", &spec.version);
    push_optional(&mut lines, "summary", spec.summary.as_deref());
    push_optional(&mut lines, "description", spec.description.as_deref());
    if !spec.authors.is_empty() {
        push_array(&mut lines, "authors", &spec.authors);
    }
    push_optional(&mut lines, "license", spec.license.as_deref());
    push_optional(&mut lines, "homepage", spec.homepage.as_deref());
    push_optional(
        &mut lines,
        "source_code_uri",
        spec.source_code_uri.as_deref(),
    );
    push_optional(&mut lines, "changelog_uri", spec.changelog_uri.as_deref());
    push_optional(
        &mut lines,
        "prayfile_version",
        spec.prayfile_version.as_deref(),
    );
    push_array(&mut lines, "files", &spec.files);
    maps::push_exports(&mut lines, &spec.exports);
    maps::push_skills(&mut lines, &spec.skills);
    maps::push_templates(&mut lines, &spec.templates);
    maps::push_strings(&mut lines, "adapters", &spec.adapters);
    if !spec.targets.is_empty() {
        push_array(&mut lines, "targets", &spec.targets);
    }
    for dependency in &spec.dependencies {
        let method = if dependency.optional {
            "add_optional_dependency"
        } else {
            "add_dependency"
        };
        lines.push(format!(
            "  spec.{method} {}, {}",
            quote(&dependency.name),
            quote(&dependency.constraint)
        ));
    }
    if !spec.metadata.is_empty() {
        lines.push(format!("  spec.metadata = {}", literal_map(&spec.metadata)));
    }
    if let Some(upstream) = &spec.upstream {
        lines.push(format!(
            "  spec.upstream {}, {}",
            quote(&upstream.name),
            quote(&upstream.constraint)
        ));
    }
    lines.push("end".to_string());
    lines.push(String::new());
    lines.join("\n")
}

fn push_assignment(lines: &mut Vec<String>, field: &str, value: &str) {
    lines.push(format!("  spec.{field} = {}", quote(value)));
}

fn push_optional(lines: &mut Vec<String>, field: &str, value: Option<&str>) {
    if let Some(value) = value {
        push_assignment(lines, field, value);
    }
}

fn push_array(lines: &mut Vec<String>, field: &str, values: &[String]) {
    lines.push(format!("  spec.{field} = [{}]", string_array(values)));
}

pub(super) fn string_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| quote(value))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn quote(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

fn literal_map(entries: &BTreeMap<String, LiteralValue>) -> String {
    let values = entries
        .iter()
        .map(|(name, value)| format!("{} => {}", quote(name), literal(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {values} }}")
}

fn literal(value: &LiteralValue) -> String {
    match value {
        LiteralValue::String(value) => quote(value),
        LiteralValue::Symbol(value) => format!(":{value}"),
        LiteralValue::Bool(value) => value.to_string(),
        LiteralValue::Null => "nil".to_string(),
        LiteralValue::Integer(value) => value.to_string(),
        LiteralValue::Array(values) => {
            format!(
                "[{}]",
                values.iter().map(literal).collect::<Vec<_>>().join(", ")
            )
        }
        LiteralValue::Map(entries) => literal_map(entries),
    }
}
