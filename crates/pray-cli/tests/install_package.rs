#[path = "install_support.rs"]
mod support;

use std::fs;
use std::path::PathBuf;

use support::{create_add_fixture, read_package_archive, run_pray, temporary_directory};

#[test]
fn package_builds_a_tar_zst_archive_from_package_contents() {
    let repo = temporary_directory("pray-package");
    create_add_fixture(&repo);

    let add = run_pray(&repo, &["add", "sample/base", "--path", "packages/base"]);
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let package = run_pray(&repo, &["package"]);
    assert!(
        package.status.success(),
        "package failed: {}",
        String::from_utf8_lossy(&package.stderr)
    );

    let archive = repo.join("sample-base-1.4.3.praypkg");
    assert!(archive.is_file());

    let entries = read_package_archive(&archive);
    let metadata = entries.get("metadata.json").expect("metadata");
    assert!(metadata.contains("\"name\": \"sample/base\""));
    assert!(metadata.contains("\"version\": \"1.4.3\""));
    assert!(metadata.contains("\"files\": ["));
    assert!(metadata.contains("README.md"));
    assert!(metadata.contains("exports/testing-basics.md"));
    assert_eq!(
        entries.get("README.md").expect("archive readme"),
        "package readme\n"
    );
    assert_eq!(
        entries
            .get("exports/testing-basics.md")
            .expect("archive export"),
        "Testing guidance\n"
    );
}

#[test]
fn vendor_copies_package_contents_into_pray_vendor() {
    let repo = temporary_directory("pray-vendor");
    create_add_fixture(&repo);

    let add = run_pray(&repo, &["add", "sample/base", "--path", "packages/base"]);
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let vendor = run_pray(&repo, &["vendor"]);
    assert!(
        vendor.status.success(),
        "vendor failed: {}",
        String::from_utf8_lossy(&vendor.stderr)
    );

    let vendored = repo.join(".pray/vendor/sample-base/1.4.3");
    assert!(vendored.is_dir());
    assert!(vendored.join("metadata.json").exists());
    assert_eq!(
        fs::read_to_string(vendored.join("README.md")).expect("vendored readme"),
        "package readme\n"
    );
    assert_eq!(
        fs::read_to_string(vendored.join("exports/testing-basics.md")).expect("vendored export"),
        "Testing guidance\n"
    );
}
