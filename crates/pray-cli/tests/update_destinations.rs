#[path = "support/update_fixture.rs"]
mod support;

use std::fs;
use support::UpdateFixture;

#[test]
fn latest_collision_preserves_recipe_and_reports_all_paths() {
    let fixture = UpdateFixture::new("~> 1.0");
    fixture.omit_ledger();
    let manifest = fs::read(fixture.consumer.join("Prayfile")).unwrap();
    let lockfile = fs::read(fixture.consumer.join("Prayfile.lock")).unwrap();
    let compose = fs::read(fixture.consumer.join("INSTRUCTIONS.md")).unwrap();
    fixture.publish("2.0.0");
    fs::write(fixture.consumer.join("rules/rules.md"), "new rules\n").unwrap();

    for flags in [
        vec!["update", "--latest"],
        vec!["update", "--latest", "--json"],
    ] {
        let output = fixture.run(&flags);
        assert_eq!(output.status.code(), Some(5));
        assert_eq!(
            fs::read(fixture.consumer.join("Prayfile")).unwrap(),
            manifest
        );
        assert_eq!(
            fs::read(fixture.consumer.join("Prayfile.lock")).unwrap(),
            lockfile
        );
        assert_eq!(
            fs::read(fixture.consumer.join("INSTRUCTIONS.md")).unwrap(),
            compose
        );
        let error = String::from_utf8_lossy(&output.stderr);
        for expected in ["a.md", "b.md", "sample/files", "move"] {
            assert!(error.contains(expected), "missing {expected}: {error}");
        }
    }
    fixture.success(&["install"]);
    fixture.success(&["update", "--latest"]);
    let updated =
        pray_core::lockfile::read_lockfile(&fixture.consumer.join("Prayfile.lock")).unwrap();
    assert_eq!(
        updated
            .package
            .iter()
            .find(|package| package.name == "sample/files")
            .unwrap()
            .version,
        "2.0.0"
    );
}

#[test]
fn latest_dry_run_checks_candidate_destinations_without_writes() {
    let fixture = UpdateFixture::new("~> 1.0");
    fixture.omit_ledger();
    let manifest = fs::read(fixture.consumer.join("Prayfile")).unwrap();
    let lockfile = fs::read(fixture.consumer.join("Prayfile.lock")).unwrap();
    fixture.publish("2.0.0");
    let output = fixture.run(&["update", "--latest", "--dry-run"]);
    assert_eq!(output.status.code(), Some(5));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("a.md") && error.contains("b.md"), "{error}");
    assert_eq!(
        fs::read(fixture.consumer.join("Prayfile")).unwrap(),
        manifest
    );
    assert_eq!(
        fs::read(fixture.consumer.join("Prayfile.lock")).unwrap(),
        lockfile
    );
}

#[test]
fn latest_json_updates_versions_already_allowed_by_the_constraint() {
    let fixture = UpdateFixture::new(">= 1.0");
    fixture.publish("2.0.0");
    let output = fixture.success(&["update", "--latest", "--json"]);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "updated");
    let lockfile =
        pray_core::lockfile::read_lockfile(&fixture.consumer.join("Prayfile.lock")).unwrap();
    assert_eq!(
        lockfile
            .package
            .iter()
            .find(|package| package.name == "sample/files")
            .unwrap()
            .version,
        "2.0.0"
    );
}

#[test]
fn verify_gives_recovery_that_preserves_an_edited_destination() {
    let fixture = UpdateFixture::new("~> 1.0");
    let lockfile =
        pray_core::lockfile::read_lockfile(&fixture.consumer.join("Prayfile.lock")).unwrap();
    let destination = fixture.consumer.join(&lockfile.provisioned[0].path);
    fs::write(&destination, "operator changes").unwrap();
    let output = fixture.run(&["verify"]);
    assert_eq!(output.status.code(), Some(6));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("move"), "{error}");
    fs::rename(&destination, destination.with_extension("saved")).unwrap();
    fixture.success(&["install"]);
    fixture.success(&["verify"]);
    assert_eq!(
        fs::read_to_string(destination.with_extension("saved")).unwrap(),
        "operator changes"
    );
}

#[test]
fn oversized_destination_is_rejected_before_output_changes() {
    let fixture = UpdateFixture::new("~> 1.0");
    let lockfile_path = fixture.consumer.join("Prayfile.lock");
    let lockfile = pray_core::lockfile::read_lockfile(&lockfile_path).unwrap();
    let destination = fixture.consumer.join(&lockfile.provisioned[0].path);
    fs::OpenOptions::new()
        .write(true)
        .open(&destination)
        .unwrap()
        .set_len(32 * 1024 * 1024 + 1)
        .unwrap();
    let previous_lock = fs::read(&lockfile_path).unwrap();
    let previous_compose = fs::read(fixture.consumer.join("INSTRUCTIONS.md")).unwrap();
    fs::write(fixture.consumer.join("rules/rules.md"), "new rules\n").unwrap();
    let output = fixture.run(&["install"]);
    assert_eq!(output.status.code(), Some(5));
    assert!(String::from_utf8_lossy(&output.stderr).contains("32 MiB"));
    assert_eq!(
        fs::metadata(destination).unwrap().len(),
        32 * 1024 * 1024 + 1
    );
    assert_eq!(fs::read(lockfile_path).unwrap(), previous_lock);
    assert_eq!(
        fs::read(fixture.consumer.join("INSTRUCTIONS.md")).unwrap(),
        previous_compose
    );
}

#[cfg(unix)]
#[test]
fn late_lock_failure_restores_manifest_outputs_and_pruned_files() {
    let fixture = UpdateFixture::new("~> 1.0");
    let lock_path = fixture.consumer.join("Prayfile.lock");
    let mut lockfile = pray_core::lockfile::read_lockfile(&lock_path).unwrap();
    fs::write(fixture.consumer.join("dropped.txt"), "old export").unwrap();
    lockfile
        .provisioned
        .push(pray_core::lockfile::ProvisionedFileRecord {
            path: "dropped.txt".into(),
            content_hash: pray_core::hashing::sha256_prefixed(b"old export"),
            package: "sample/files".into(),
            export: "files".into(),
        });
    pray_core::lockfile::write_lockfile(&lock_path, &lockfile).unwrap();
    let mut paths = vec![
        "Prayfile".to_owned(),
        "Prayfile.lock".to_owned(),
        "INSTRUCTIONS.md".to_owned(),
    ];
    paths.extend(lockfile.provisioned.iter().map(|file| file.path.clone()));
    let original: Vec<_> = paths
        .iter()
        .map(|path| fs::read(fixture.consumer.join(path)).unwrap())
        .collect();
    fs::rename(&lock_path, fixture.consumer.join("original.lock")).unwrap();
    std::os::unix::fs::symlink("original.lock", &lock_path).unwrap();
    fixture.publish("2.0.0");
    fs::write(fixture.consumer.join("rules/rules.md"), "new rules\n").unwrap();
    let output = fixture.run(&["update", "--latest"]);
    assert_eq!(
        output.status.code(),
        Some(5),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("symbolic link"));
    for (path, bytes) in paths.iter().zip(original) {
        assert_eq!(
            fs::read(fixture.consumer.join(path)).unwrap(),
            bytes,
            "{path}"
        );
    }
}
