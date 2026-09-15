use pray_core::embed::{
    default_lockfile_path, default_manifest_path, lockfiles_equivalent, parse_lockfile,
    parse_manifest, serialize_lockfile, Lockfile,
};
use std::path::Path;

#[test]
fn embed_module_parses_manifest_and_lockfile() {
    let manifest = parse_manifest(
        r#"
prayfile "1"
compose "INSTRUCTIONS.md" do
end
"#,
    )
    .expect("manifest");
    assert_eq!(manifest.prayfile_version, "1");
    assert_eq!(manifest.targets.len(), 1);

    let lockfile = Lockfile {
        prayfile_lock: "1".to_string(),
        spec: "0.1".to_string(),
        generated_by: "pray test".to_string(),
        manifest_hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        ..Lockfile::default()
    };
    let serialized = serialize_lockfile(&lockfile).expect("serialize");
    let parsed = parse_lockfile(&serialized).expect("parse");
    assert!(lockfiles_equivalent(&lockfile.canonicalized(), &parsed));
}

#[test]
fn embed_module_default_paths() {
    let root = Path::new(".");
    assert_eq!(default_manifest_path(root), root.join("Prayfile"));
    assert_eq!(default_lockfile_path(root), root.join("Prayfile.lock"));
}
