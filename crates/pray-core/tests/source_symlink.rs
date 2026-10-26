use pray_core::package_spec::{parse_package_spec, PackageSpec};
use pray_core::resolve::resolve_project_in_context;
use pray_core::resolve_context::ResolveOptions;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{label}-{stamp}"));
    fs::create_dir_all(&path).expect("temp dir");
    path
}

fn write_package(root: &Path, body_path: &Path) {
    let package_root = root.join("packages/base");
    fs::create_dir_all(package_root.join("exports")).expect("package dirs");
    fs::write(
        package_root.join("base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["exports/a.md"]
  spec.exports = {
    "a" => { type: "fragment", path: "exports/a.md", summary: "a" }
  }
end
"#,
    )
    .expect("prayspec");
    symlink(body_path, package_root.join("exports/a.md")).expect("package symlink");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
compose "NOTES.md" do
  pray "sample/base", path: "packages/base"
end
"#,
    )
    .expect("prayfile");
}

fn write_regular_package(root: &Path) {
    let package_root = root.join("base");
    fs::create_dir_all(package_root.join("exports")).expect("package dirs");
    fs::write(
        package_root.join("base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["exports/a.md"]
  spec.exports = {
    "a" => { type: "fragment", path: "exports/a.md", summary: "a" }
  }
end
"#,
    )
    .expect("prayspec");
    fs::write(package_root.join("exports/a.md"), "outside package bytes\n").expect("package file");
}

#[test]
fn path_package_file_symlink_outside_package_is_rejected() {
    let root = unique_temp_dir("pray-source-symlink-package");
    let outside = unique_temp_dir("pray-source-symlink-outside");
    let secret = outside.join("secret.md");
    fs::write(&secret, "outside package bytes\n").expect("outside file");
    write_package(&root, &secret);

    let error =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect_err("symlink package file");
    assert!(
        error.to_string().contains("symbolic link"),
        "resolve should refuse the linked package file, got {error}"
    );

    let spec_text = fs::read_to_string(root.join("packages/base/base.prayspec")).expect("spec");
    let spec = parse_package_spec(&spec_text).expect("parse spec");
    let hash_error =
        PackageSpec::tree_hash_for_root(&spec, &root.join("packages/base")).expect_err("tree hash");
    assert!(
        hash_error.to_string().contains("symbolic link"),
        "tree hash should refuse the linked package file, got {hash_error}"
    );
}

#[test]
fn path_package_directory_symlink_outside_project_is_rejected() {
    let root = unique_temp_dir("pray-source-symlink-package-root");
    let outside = unique_temp_dir("pray-source-symlink-package-root-outside");
    write_regular_package(&outside);
    fs::create_dir_all(root.join("packages")).expect("package parent");
    symlink(outside.join("base"), root.join("packages/base")).expect("package symlink");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
compose "NOTES.md" do
  pray "sample/base", path: "packages/base"
end
"#,
    )
    .expect("prayfile");

    let error =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect_err("symlink package directory");
    assert!(
        error.to_string().contains("symbolic link"),
        "resolve should refuse the linked package directory, got {error}"
    );
}

#[test]
fn path_source_parent_directory_resolves_when_it_is_not_a_symlink() {
    let workspace = unique_temp_dir("pray-source-parent-package");
    let root = workspace.join("project");
    fs::create_dir_all(&root).expect("project");
    write_regular_package(&workspace.join("pkgs"));
    let spec_path = workspace.join("pkgs/base/base.prayspec");
    let spec = fs::read_to_string(&spec_path).expect("spec");
    fs::write(&spec_path, spec.replace("sample/base", "local/base")).expect("rename spec");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
source "local", path: "../pkgs"
compose "NOTES.md" do
  pray "local/base"
end
"#,
    )
    .expect("prayfile");

    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect("parent path package");
    let body = project.packages[0].export_bodies.get("a").expect("export");
    assert!(body.contains("outside package bytes"));
}

#[test]
fn compose_local_symlink_is_rejected() {
    let root = unique_temp_dir("pray-source-symlink-local");
    let outside = unique_temp_dir("pray-source-symlink-local-outside");
    let secret = outside.join("secret.md");
    fs::write(&secret, "outside local bytes\n").expect("outside file");
    symlink(&secret, root.join("link.md")).expect("local symlink");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
compose "NOTES.md" do
  local "./link.md"
end
"#,
    )
    .expect("prayfile");

    let error =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect_err("symlink local");
    assert!(
        error.to_string().contains("symbolic link"),
        "resolve should refuse the linked local, got {error}"
    );
}
