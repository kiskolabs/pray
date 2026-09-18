use pray_core::manifest::parse_manifest;

#[test]
fn parses_path_and_url_publish_remotes() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
source "local", path: "guidance"
publish "prayers", path: "prayers"
publish "public", "https://prayers.example"
compose "AGENTS.md" do
  pray "local/project"
end
"#,
    )
    .expect("manifest parses");
    assert_eq!(manifest.publish_remotes.len(), 2);
    assert_eq!(manifest.publish_remotes[0].name, "prayers");
    assert_eq!(manifest.publish_remotes[0].path.as_deref(), Some("prayers"));
    assert_eq!(
        manifest.publish_remotes[1].url.as_deref(),
        Some("https://prayers.example")
    );
}

#[test]
fn parses_publish_package_block() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
publish "prayers", path: "prayers" do
  pray "local/project"
end
pray "local/project", path: "guidance/project"
"#,
    )
    .expect("manifest parses");
    assert_eq!(
        manifest.publish_remotes[0].packages,
        vec!["local/project".to_string()]
    );
}

#[test]
fn rejects_unknown_package_in_publish_block() {
    let error = parse_manifest(
        r#"
prayfile "1"
publish "prayers", path: "prayers" do
  pray "missing/package"
end
"#,
    )
    .expect_err("unknown package");
    assert!(error.to_string().contains("unknown package"));
}

#[test]
fn rejects_git_package_in_publish_block() {
    let error = parse_manifest(
        r#"
prayfile "1"
publish "prayers", path: "prayers" do
  pray "amkisko/rules"
end
pray "amkisko/rules", git: "https://example.com/rules.git"
"#,
    )
    .expect_err("git package");
    assert!(error.to_string().contains("not a path package"));
}

#[test]
fn rejects_escaping_publish_path() {
    let error = parse_manifest(
        r#"
prayfile "1"
publish "out", path: "../outside"
"#,
    )
    .expect_err("escape");
    assert!(error.to_string().contains("escapes repository root"));
}

#[test]
fn rejects_duplicate_publish_names() {
    let error = parse_manifest(
        r#"
prayfile "1"
publish "prayers", path: "prayers"
publish "prayers", "https://prayers.example"
"#,
    )
    .expect_err("duplicate");
    assert!(error.to_string().contains("duplicate publish remote"));
}

#[test]
fn parses_url_that_ends_with_do() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
publish "public", "https://prayers.example/do"
"#,
    )
    .expect("manifest parses");
    assert_eq!(
        manifest.publish_remotes[0].url.as_deref(),
        Some("https://prayers.example/do")
    );
    assert!(manifest.publish_remotes[0].packages.is_empty());
}

#[test]
fn rejects_signing_key_on_publish() {
    let error = parse_manifest(
        r#"
prayfile "1"
publish "prayers", path: "prayers", signing_key: "secret.pem"
"#,
    )
    .expect_err("secret");
    assert!(error.to_string().contains("does not take git:"));
}

#[test]
fn empty_publish_remotes_omit_from_manifest_hash() {
    let without = parse_manifest(
        r#"
prayfile "1"
source "local", path: "guidance"
pray "local/project"
"#,
    )
    .expect("without");
    let with_empty_field = parse_manifest(
        r#"
prayfile "1"
source "local", path: "guidance"
pray "local/project"
"#,
    )
    .expect("also without");
    assert_eq!(
        without.manifest_hash().expect("hash"),
        with_empty_field.manifest_hash().expect("hash")
    );
    assert!(without.publish_remotes.is_empty());
}
