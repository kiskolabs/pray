use pray_core::embed::{inspect_locked_destinations, LockedPackage, Lockfile, ManagedSpanRecord};
use pray_core::hashing::checksum_managed_span_content;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "pray-locked-dest-{}-{}",
        std::process::id(),
        sequence
    ));
    fs::create_dir_all(&root).expect("scratch");
    root
}

fn lockfile_with_span(checksum: &str) -> Lockfile {
    Lockfile {
        prayfile_lock: "1".to_string(),
        spec: "0.1".to_string(),
        generated_by: "pray test".to_string(),
        manifest_hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        package: vec![LockedPackage {
            name: "sample/base".to_string(),
            version: "1.0.0".to_string(),
            source: None,
            path: "./packages/base".to_string(),
            tree_hash: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                .to_string(),
            artifact_hash:
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .to_string(),
            artifact: "path:./packages/base".to_string(),
            exports: vec!["testing-basics".to_string()],
            dependencies: Vec::new(),
            signer_fingerprint: None,
            upstream: None,
        }],
        managed_span: vec![ManagedSpanRecord {
            id: "abcd1234".to_string(),
            target: "INSTRUCTIONS.md".to_string(),
            open_line: 1,
            close_line: 3,
            ideal_checksum: checksum.to_string(),
            package: "sample/base".to_string(),
            export: "testing-basics".to_string(),
            source_checksum:
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_string(),
            silenced: false,
        }],
        ..Lockfile::default()
    }
}

#[test]
fn inspect_locked_destinations_accepts_matching_span() {
    let root = scratch();
    let checksum = checksum_managed_span_content("hello");
    fs::write(
        root.join("INSTRUCTIONS.md"),
        "<!-- pray:abcd1234 -->\nhello\n<!-- pray:abcd1234 -->\n",
    )
    .expect("write dest");
    let report =
        inspect_locked_destinations(&root, &lockfile_with_span(&checksum)).expect("inspect");
    let _ = fs::remove_dir_all(&root);
    assert!(report.is_clean(), "{:?}", report.findings);
}

#[test]
fn inspect_locked_destinations_reports_edited_span() {
    let root = scratch();
    let checksum = checksum_managed_span_content("hello");
    fs::write(
        root.join("INSTRUCTIONS.md"),
        "<!-- pray:abcd1234 -->\nedited\n<!-- pray:abcd1234 -->\n",
    )
    .expect("write dest");
    let report =
        inspect_locked_destinations(&root, &lockfile_with_span(&checksum)).expect("inspect");
    let _ = fs::remove_dir_all(&root);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.kind == "custom_implementation"),
        "{:?}",
        report.findings
    );
}

#[test]
fn inspect_locked_destinations_reports_missing_dest() {
    let root = scratch();
    let checksum = checksum_managed_span_content("hello");
    let report =
        inspect_locked_destinations(&root, &lockfile_with_span(&checksum)).expect("inspect");
    let _ = fs::remove_dir_all(&root);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.kind == "verify_error"),
        "{:?}",
        report.findings
    );
}
