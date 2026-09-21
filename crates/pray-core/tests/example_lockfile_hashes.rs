use pray_core::lockfile::read_lockfile;
use pray_core::manifest::parse_manifest;
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn committed_lockfiles_match_the_current_prayfile_hash() {
    let root = workspace_root();
    for relative in [
        "examples/simple-project",
        "examples/customized-render",
        "examples/team-workflow",
        "",
    ] {
        let label = if relative.is_empty() {
            "repository root"
        } else {
            relative
        };
        let project = if relative.is_empty() {
            root.clone()
        } else {
            root.join(relative)
        };
        let text = fs::read_to_string(project.join("Prayfile")).expect("Prayfile");
        let hash = parse_manifest(&text)
            .expect("parse Prayfile")
            .manifest_hash()
            .expect("manifest hash");
        let lockfile = read_lockfile(&project.join("Prayfile.lock")).expect("lockfile");
        assert_eq!(
            lockfile.manifest_hash, hash,
            "{label} lockfile hash must match the current Prayfile"
        );
    }
}

#[test]
fn simple_project_manifest_hash_is_stable_across_clients() {
    let text = fs::read_to_string(workspace_root().join("examples/simple-project/Prayfile"))
        .expect("simple-project Prayfile");
    let hash = parse_manifest(&text)
        .expect("parse")
        .manifest_hash()
        .expect("hash");
    assert_eq!(
        hash, "sha256:52235a67dc445cbf8548de90999310ef42be3cafd81fe63a43a5ccd89ed047a1",
        "Ruby client pins this same simple-project hash"
    );
}
