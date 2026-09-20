#[path = "install_network_support.rs"]
mod support;

use std::fs;
use support::{run_pray, temporary_directory};

#[test]
fn prayer_init_and_repo_init_share_one_prayers_tree() {
    let repository = temporary_directory("pray-hybrid-init");
    fs::write(repository.join("Prayfile"), "prayfile \"1\"\n").expect("Prayfile");

    let prayer = run_pray(&repository, &["prayer", "init", "notes"]);
    assert!(
        prayer.status.success(),
        "prayer init failed: {}",
        String::from_utf8_lossy(&prayer.stderr)
    );
    let repo = run_pray(&repository, &["repo", "init"]);
    assert!(
        repo.status.success(),
        "repo init failed: {}",
        String::from_utf8_lossy(&repo.stderr)
    );

    assert!(repository.join("prayers/notes/notes.prayspec").is_file());
    assert!(repository.join("prayers/v1/index.json").is_file());
    assert!(!repository.join("packages").exists());

    let manifest = fs::read_to_string(repository.join("Prayfile")).expect("Prayfile");
    assert!(manifest.contains("source \"local\", path: \"prayers\""));
    assert!(manifest.contains("pray \"local/notes\""));
    assert!(manifest.contains("publish \"prayers\", path: \"prayers\""));
}

#[test]
fn product_hybrid_publishes_path_source_into_prayers_v1() {
    let repository = temporary_directory("pray-hybrid-publish");
    fs::write(repository.join("Prayfile"), "prayfile \"1\"\n").expect("Prayfile");

    let prayer = run_pray(&repository, &["prayer", "init", "notes"]);
    assert!(
        prayer.status.success(),
        "prayer init failed: {}",
        String::from_utf8_lossy(&prayer.stderr)
    );
    let repo = run_pray(&repository, &["repo", "init"]);
    assert!(
        repo.status.success(),
        "repo init failed: {}",
        String::from_utf8_lossy(&repo.stderr)
    );

    let prayspec_path = repository.join("prayers/notes/notes.prayspec");
    let prayspec = fs::read_to_string(&prayspec_path).expect("prayspec");
    fs::write(
        &prayspec_path,
        prayspec.replace(
            "spec.name = \"local/notes\"\n",
            "spec.name = \"local/notes\"\n  spec.version = \"0.1.0\"\n",
        ),
    )
    .expect("version");

    let publish = run_pray(&repository, &["publish"]);
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );
    assert!(repository
        .join("prayers/v1/packages/local/notes.json")
        .is_file());
    assert!(!repository.join("packages").exists());
}
