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

fn write_package(root: &Path, leaf: &str, version: &str, optional: Option<(&str, &str)>) {
    let package_root = root.join(format!("packages/{leaf}"));
    fs::create_dir_all(&package_root).expect("package dir");
    fs::write(package_root.join("README.md"), format!("{leaf}\n")).expect("README");
    let dependency = optional
        .map(|(name, constraint)| {
            format!("  spec.add_optional_dependency \"{name}\", \"{constraint}\"\n")
        })
        .unwrap_or_default();
    fs::write(
        package_root.join(format!("{leaf}.prayspec")),
        format!(
            "Package::Specification.new do |spec|\n  spec.name = \"sample/{leaf}\"\n  spec.version = \"{version}\"\n  spec.files = [\"README.md\"]\n{dependency}end\n"
        ),
    )
    .expect("prayspec");
}

fn write_manifest(root: &Path, packages: &[&str]) {
    let declarations = packages
        .iter()
        .map(|name| format!("pray \"sample/{name}\", path: \"packages/{name}\""))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        root.join("Prayfile"),
        format!("prayfile \"1\"\n{declarations}\n"),
    )
    .expect("Prayfile");
}

#[test]
fn resolve_accepts_mutual_optional_referrals() {
    let root = temporary_directory("pray-mutual-optional");
    write_package(&root, "alpha", "1.0.0", Some(("sample/beta", "~> 1.0")));
    write_package(&root, "beta", "1.0.0", Some(("sample/alpha", "~> 1.0")));
    write_manifest(&root, &["alpha", "beta"]);

    let project = resolve_project(&root.join("Prayfile")).expect("mutual optional referrals");
    assert_eq!(project.packages.len(), 2);
}

#[test]
fn resolve_checks_optional_constraint_when_target_is_selected() {
    let root = temporary_directory("pray-optional-constraint");
    write_package(&root, "alpha", "1.0.0", Some(("sample/beta", "~> 2.0")));
    write_package(&root, "beta", "1.0.0", None);
    write_manifest(&root, &["alpha", "beta"]);

    let error = resolve_project(&root.join("Prayfile")).expect_err("optional constraint mismatch");
    assert!(
        error
            .to_string()
            .contains("optional referral sample/alpha -> sample/beta"),
        "unexpected error: {error}"
    );
    assert!(error.to_string().contains("requires ~> 2.0"));
    assert!(error.to_string().contains("selected version is 1.0.0"));
}

#[test]
fn resolve_does_not_select_an_optional_target() {
    let root = temporary_directory("pray-optional-absent");
    write_package(&root, "alpha", "1.0.0", Some(("sample/missing", "~> 1.0")));
    write_manifest(&root, &["alpha"]);

    let project = resolve_project(&root.join("Prayfile")).expect("absent optional referral");
    assert_eq!(project.packages.len(), 1);
}
