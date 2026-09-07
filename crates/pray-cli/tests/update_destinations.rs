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
