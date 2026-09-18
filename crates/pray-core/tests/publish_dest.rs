use pray_core::manifest::ManifestPackage;
use pray_core::publish_remote::{
    allowed_publish_names, package_is_path_owned, path_owned_package_names,
    resolve_publish_destinations, ManifestPublishRemote, PublishCliDest,
};
use std::path::{Path, PathBuf};

fn remote(name: &str, path: Option<&str>, url: Option<&str>) -> ManifestPublishRemote {
    ManifestPublishRemote {
        name: name.to_string(),
        path: path.map(str::to_string),
        url: url.map(str::to_string),
        packages: Vec::new(),
    }
}

#[test]
fn undeclared_publish_still_requires_cli_dest() {
    let error =
        resolve_publish_destinations(&[], &PublishCliDest::default(), Path::new("/tmp/project"))
            .expect_err("dest");
    assert!(error.to_string().contains("--root PATH or --server URL"));
}

#[test]
fn declared_remotes_fill_in_without_flags() {
    let remotes = vec![
        remote("prayers", Some("prayers"), None),
        remote("public", None, Some("https://prayers.example")),
    ];
    let dests = resolve_publish_destinations(
        &remotes,
        &PublishCliDest::default(),
        Path::new("/tmp/project"),
    )
    .expect("dests");
    assert_eq!(dests.len(), 2);
    assert_eq!(
        dests[0].root.as_deref(),
        Some(Path::new("/tmp/project/prayers"))
    );
    assert_eq!(dests[1].server.as_deref(), Some("https://prayers.example"));
}

#[test]
fn to_selects_one_remote() {
    let remotes = vec![
        remote("prayers", Some("prayers"), None),
        remote("public", None, Some("https://prayers.example")),
    ];
    let dests = resolve_publish_destinations(
        &remotes,
        &PublishCliDest {
            to: vec!["prayers".to_string()],
            ..PublishCliDest::default()
        },
        Path::new("/tmp/project"),
    )
    .expect("dests");
    assert_eq!(dests.len(), 1);
    assert_eq!(dests[0].name, "prayers");
}

#[test]
fn to_cannot_combine_with_root() {
    let remotes = vec![remote("prayers", Some("prayers"), None)];
    let error = resolve_publish_destinations(
        &remotes,
        &PublishCliDest {
            to: vec!["prayers".to_string()],
            roots: vec![PathBuf::from("prayers")],
            ..PublishCliDest::default()
        },
        Path::new("/tmp/project"),
    )
    .expect_err("combine");
    assert!(error.to_string().contains("cannot be combined"));
}

#[test]
fn undeclared_root_is_a_usage_error() {
    let remotes = vec![remote("prayers", Some("prayers"), None)];
    let error = resolve_publish_destinations(
        &remotes,
        &PublishCliDest {
            roots: vec![PathBuf::from("other")],
            ..PublishCliDest::default()
        },
        Path::new("/tmp/project"),
    )
    .expect_err("allowlist");
    assert!(error.to_string().contains("not a declared publish remote"));
}

#[test]
fn relative_root_matches_declared_path() {
    let remotes = vec![remote("prayers", Some("prayers"), None)];
    let dests = resolve_publish_destinations(
        &remotes,
        &PublishCliDest {
            roots: vec![PathBuf::from("./prayers")],
            ..PublishCliDest::default()
        },
        Path::new("/tmp/project"),
    )
    .expect("dests");
    assert_eq!(dests.len(), 1);
    assert_eq!(dests[0].name, "prayers");
}

#[test]
fn path_package_is_path_owned() {
    let package = ManifestPackage {
        name: "local/project".to_string(),
        path: Some("guidance/project".to_string()),
        ..ManifestPackage::default()
    };
    assert!(package_is_path_owned(&package, &[]));
}

#[test]
fn git_declaration_is_not_path_owned() {
    let package = ManifestPackage {
        name: "amkisko/rules".to_string(),
        git: Some("https://example.com/rules.git".to_string()),
        ..ManifestPackage::default()
    };
    assert!(!package_is_path_owned(&package, &[]));
}

#[test]
fn mixed_manifest_publishes_path_packages_only() {
    let manifest = pray_core::manifest::parse_manifest(
        r#"
prayfile "1"
source "local", path: "guidance"
pray "local/project"
pray "amkisko/rules", git: "https://example.com/rules.git"
"#,
    )
    .expect("manifest");
    assert_eq!(
        path_owned_package_names(&manifest),
        vec!["local/project".to_string()]
    );
    assert_eq!(
        allowed_publish_names(&manifest, &[]),
        vec!["local/project".to_string()]
    );
}

#[test]
fn dest_selection_finds_named_remote_among_many() {
    let remotes: Vec<ManifestPublishRemote> = (0..200)
        .map(|index| {
            remote(
                &format!("remote-{index:03}"),
                Some(&format!("dest-{index:03}")),
                None,
            )
        })
        .collect();
    let dests = resolve_publish_destinations(
        &remotes,
        &PublishCliDest {
            to: vec!["remote-173".to_string()],
            ..PublishCliDest::default()
        },
        Path::new("/tmp/project"),
    )
    .expect("dests");
    assert_eq!(dests.len(), 1);
    assert_eq!(dests[0].name, "remote-173");
    assert_eq!(
        dests[0].root.as_deref(),
        Some(Path::new("/tmp/project/dest-173"))
    );
}
