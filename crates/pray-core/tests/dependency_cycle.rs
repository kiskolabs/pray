use pray_core::resolve::resolve_project;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_directory(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{nanos}"));
    fs::create_dir_all(&path).expect("temp dir");
    path
}

fn write_package(root: &Path, name: &str, version: &str, dependency: Option<&str>) {
    let package_root = root.join(format!("packages/{name}"));
    fs::create_dir_all(package_root.join("exports")).expect("package dirs");
    let dependency_line = dependency
        .map(|dep| format!("  spec.add_dependency \"{dep}\", \"~> 1.0\"\n"))
        .unwrap_or_default();
    fs::write(
        package_root.join(format!("{name}.prayspec")),
        format!(
            r#"
Package::Specification.new do |spec|
  spec.name = "sample/{name}"
  spec.version = "{version}"
  spec.summary = "{name}"
  spec.files = ["README.md", "exports/{name}.md"]
  spec.exports = {{
    "{name}" => {{
      type: "fragment",
      path: "exports/{name}.md",
      summary: "{name}"
    }}
  }}
{dependency_line}end
"#
        ),
    )
    .expect("prayspec");
    fs::write(package_root.join("README.md"), "readme\n").expect("readme");
    fs::write(
        package_root.join(format!("exports/{name}.md")),
        format!("{name}\n"),
    )
    .expect("export");
}

#[test]
fn resolve_rejects_two_package_dependency_cycle() {
    let root = temporary_directory("pray-cycle");
    write_package(&root, "alpha", "1.0.0", Some("sample/beta"));
    write_package(&root, "beta", "1.0.0", Some("sample/alpha"));
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
pray "sample/alpha", path: "packages/alpha"
pray "sample/beta", path: "packages/beta"
"#,
    )
    .expect("Prayfile");

    let error = resolve_project(&root.join("Prayfile")).expect_err("cycle");
    let message = error.to_string();
    assert!(
        message.contains("dependency cycle detected"),
        "unexpected error: {message}"
    );
    assert!(message.contains("sample/alpha") || message.contains("sample/beta"));
}

#[test]
fn resolve_accepts_acyclic_dependency_graph() {
    let root = temporary_directory("pray-acyclic");
    write_package(&root, "base", "1.0.0", Some("sample/common"));
    write_package(&root, "common", "1.0.0", None);
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
pray "sample/base", path: "packages/base"
pray "sample/common", path: "packages/common"
"#,
    )
    .expect("Prayfile");

    let project = resolve_project(&root.join("Prayfile")).expect("acyclic resolve");
    assert_eq!(project.packages.len(), 2);
}

#[test]
fn resolve_rejects_undeclared_required_dependency() {
    let root = temporary_directory("pray-undeclared-dep");
    write_package(&root, "base", "1.0.0", Some("sample/missing"));
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
pray "sample/base", path: "packages/base"
"#,
    )
    .expect("Prayfile");

    let error = resolve_project(&root.join("Prayfile")).expect_err("undeclared");
    let message = error.to_string();
    assert!(
        message.contains("undeclared package dependencies"),
        "unexpected error: {message}"
    );
    assert!(message.contains("sample/missing"));
}
