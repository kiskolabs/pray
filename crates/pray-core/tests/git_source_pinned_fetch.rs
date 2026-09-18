#![allow(dead_code)]

#[path = "support/git_catalog.rs"]
mod git_catalog;

use git_catalog::pinned_shallow_cache_fixture;
use pray_core::embed::{resolve_project_with_options, ResolveOptions};

#[test]
fn install_fetches_a_locked_revision_missing_from_a_shallow_cache() {
    let fixture = pinned_shallow_cache_fixture(true);
    let project =
        resolve_project_with_options(&fixture.root.join("Prayfile"), &ResolveOptions::default())
            .expect("install should fetch the locked revision");
    assert_eq!(
        project.source_revisions.get("used").map(String::as_str),
        Some(fixture.pinned.as_str())
    );
    assert_eq!(project.packages[0].spec.version, "1.0.0");
}

#[test]
fn offline_install_refuses_a_locked_revision_missing_from_a_shallow_cache() {
    let fixture = pinned_shallow_cache_fixture(true);
    let error = resolve_project_with_options(
        &fixture.root.join("Prayfile"),
        &ResolveOptions {
            offline: true,
            ..ResolveOptions::default()
        },
    )
    .expect_err("offline install must not fetch");
    let message = error.to_string();
    assert!(
        message.contains(&fixture.pinned),
        "offline error should name the pin:\n{message}"
    );
    assert!(
        message.contains("offline"),
        "offline error should name offline mode:\n{message}"
    );
    assert!(
        !message.contains("--locked"),
        "offline error should not mention --locked:\n{message}"
    );
}
