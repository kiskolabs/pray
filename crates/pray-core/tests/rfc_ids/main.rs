mod check;

use check::check_rfc_tree;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn scratch() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "pray-rfc-ids-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(&root).expect("scratch");
    root
}

fn write_files(root: &Path, files: &[(&str, &str)]) {
    for (relative, body) in files {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent");
        }
        fs::write(path, body).expect("write");
    }
}

#[test]
fn duplicate_draft_files_report_both_paths() {
    let root = scratch();
    write_files(
        &root,
        &[
            ("ids/0108", "alpha\n"),
            ("0108-alpha.md", "# a\n"),
            ("0108-beta.md", "# b\n"),
        ],
    );
    let errors = check_rfc_tree(&root);
    let _ = fs::remove_dir_all(&root);
    let joined = errors.join("\n");
    assert!(joined.contains("0108-alpha.md"), "{joined}");
    assert!(joined.contains("0108-beta.md"), "{joined}");
}

#[test]
fn draft_requires_id_claim() {
    let root = scratch();
    write_files(&root, &[("0108-alpha.md", "# a\n")]);
    let errors = check_rfc_tree(&root);
    let _ = fs::remove_dir_all(&root);
    assert!(
        errors.iter().any(|error| error.contains("rfcs/ids/0108")),
        "{errors:?}"
    );
}

#[test]
fn reserved_claim_without_draft_is_ok() {
    let root = scratch();
    write_files(&root, &[("ids/0105", "reserved trust-enrollment\n")]);
    let errors = check_rfc_tree(&root);
    let _ = fs::remove_dir_all(&root);
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn reserved_claim_rejects_existing_draft() {
    let root = scratch();
    write_files(
        &root,
        &[
            ("ids/0105", "reserved trust-enrollment\n"),
            ("0105-other.md", "# x\n"),
        ],
    );
    let errors = check_rfc_tree(&root);
    let _ = fs::remove_dir_all(&root);
    assert!(
        errors.iter().any(|error| error.contains("reserved")),
        "{errors:?}"
    );
}

#[test]
fn live_rfc_tree_matches_id_claims() {
    let rfc_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rfcs");
    let errors = check_rfc_tree(&rfc_root);
    assert!(errors.is_empty(), "{errors:?}");
}
