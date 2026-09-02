#[path = "install_support.rs"]
mod support;

use std::fs;
use std::path::Path;
use support::{run_pray, temporary_directory};

fn write_fixture(root: &Path, body: &str) {
    let package_root = root.join("packages/shell");
    fs::create_dir_all(package_root.join("exports")).expect("package directories");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
pray "sample/shell", "~> 1.0", path: "packages/shell", file: ".zshrc"
"#,
    )
    .expect("Prayfile");
    fs::write(
        package_root.join("shell.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/shell"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["exports/zshrc"]
  spec.exports = {
    "zshrc" => { type: "file", path: "exports/zshrc" }
  }
end
"#,
    )
    .expect("prayspec");
    fs::write(package_root.join("exports/zshrc"), body).expect("export");
}

#[test]
fn plan_refuses_an_unmanaged_provisioned_destination() {
    let root = temporary_directory("pray-plan-provisioned-refusal");
    write_fixture(&root, "package aliases\n");
    fs::write(root.join(".zshrc"), "operator aliases\n").expect("operator file");

    let plan = run_pray(&root, &["plan"]);

    assert!(!plan.status.success());
    assert!(String::from_utf8_lossy(&plan.stderr).contains("refusing to overwrite `.zshrc`"));
    assert_eq!(
        fs::read_to_string(root.join(".zshrc")).expect("operator file kept"),
        "operator aliases\n"
    );
}

#[test]
fn failed_update_keeps_the_previous_lockfile_for_retry() {
    let root = temporary_directory("pray-provisioned-retry");
    write_fixture(&root, "old aliases\n");
    assert!(run_pray(&root, &["install"]).status.success());
    let previous_lock = fs::read(root.join("Prayfile.lock")).expect("previous lock");

    fs::write(root.join("packages/shell/exports/zshrc"), "new aliases\n").expect("new export");
    let destination = root.join(".zshrc");
    let original_permissions = fs::metadata(&destination)
        .expect("destination metadata")
        .permissions();
    let mut read_only = original_permissions.clone();
    read_only.set_readonly(true);
    fs::set_permissions(&destination, read_only).expect("read-only destination");

    let failed = run_pray(&root, &["install"]);
    assert!(!failed.status.success());
    assert_eq!(
        fs::read(root.join("Prayfile.lock")).expect("lock after failure"),
        previous_lock
    );

    fs::set_permissions(&destination, original_permissions).expect("restore permissions");
    let retry = run_pray(&root, &["install"]);
    assert!(
        retry.status.success(),
        "retry failed: {}",
        String::from_utf8_lossy(&retry.stderr)
    );
    assert_eq!(
        fs::read_to_string(root.join(".zshrc")).expect("updated destination"),
        "new aliases\n"
    );
}
