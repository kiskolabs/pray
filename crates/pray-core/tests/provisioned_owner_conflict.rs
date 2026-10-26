use pray_core::render::write_rendered_targets;
use pray_core::resolve::resolve_project_in_context;
use pray_core::resolve_context::ResolveOptions;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_directory() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("pray-provisioned-owner-conflict-{stamp}"))
}

fn write_package(root: &Path, directory: &str, package: &str, export: &str) {
    let package_root = root.join("packages").join(directory);
    fs::create_dir_all(package_root.join("exports")).expect("package directory");
    fs::write(
        package_root.join(format!("{directory}.prayspec")),
        format!(
            r#"
Package::Specification.new do |spec|
  spec.name = "{package}"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["exports/shared"]
  spec.exports = {{
    "{export}" => {{ type: "file", path: "exports/shared" }}
  }}
end
"#,
        ),
    )
    .expect("prayspec");
    fs::write(package_root.join("exports/shared"), package).expect("export");
}

#[test]
fn different_owners_cannot_provision_the_same_destination() {
    let root = temporary_directory();
    write_package(&root, "alpha", "sample/alpha", "alpha-file");
    write_package(&root, "beta", "sample/beta", "beta-file");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
pray "sample/alpha", "~> 1.0", path: "packages/alpha", file: ".shared"
pray "sample/beta", "~> 1.0", path: "packages/beta", file: ".shared"
"#,
    )
    .expect("Prayfile");
    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect("resolve");

    let error = write_rendered_targets(&project, &[]).expect_err("owner conflict");
    let message = error.to_string();
    assert!(message.contains("provisioned destination `.shared` has multiple owners"));
    assert!(message.contains("sample/alpha"));
    assert!(message.contains("alpha-file"));
    assert!(message.contains("sample/beta"));
    assert!(message.contains("beta-file"));
    assert!(!root.join(".shared").exists());
}
