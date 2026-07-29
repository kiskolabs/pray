use pray_core::lockfile::ManagedSpanRecord;
use pray_core::resolve::ResolvedLocalFile;
use pray_core::verify::position::{format_position_drift_message, summarize_position_drift};
use std::collections::BTreeMap;

fn span(id: &str, open: usize, close: usize, checksum: &str) -> ManagedSpanRecord {
    ManagedSpanRecord {
        id: id.to_string(),
        target: "AGENTS.md".to_string(),
        open_line: open,
        close_line: close,
        ideal_checksum: checksum.to_string(),
        package: "sample/base".to_string(),
        export: "guidance".to_string(),
        source_checksum: "sha256:source".to_string(),
        silenced: false,
    }
}

#[test]
fn groups_uniform_position_drift_with_local_cause() {
    let locked = [
        span("aaaa1111", 4, 6, "sha256:one"),
        span("bbbb2222", 8, 10, "sha256:two"),
    ];
    let spans: Vec<&ManagedSpanRecord> = locked.iter().collect();
    let mut markers = BTreeMap::new();
    markers.insert("aaaa1111".to_string(), (6, 8, "sha256:one".to_string()));
    markers.insert("bbbb2222".to_string(), (10, 12, "sha256:two".to_string()));
    let on_disk = [
        "# Title",
        "",
        "Local alpha",
        "Extra unmarked",
        "",
        "Local beta",
        "<!-- pray:aaaa1111 -->",
        "body one",
        "<!-- pray:aaaa1111 -->",
        "",
        "<!-- pray:bbbb2222 -->",
        "body two",
        "<!-- pray:bbbb2222 -->",
    ];
    let fresh = [
        "# Title",
        "",
        "Local alpha",
        "Local beta",
        "<!-- pray:aaaa1111 -->",
        "body one",
        "<!-- pray:aaaa1111 -->",
        "",
        "<!-- pray:bbbb2222 -->",
        "body two",
        "<!-- pray:bbbb2222 -->",
    ];
    let locals = [ResolvedLocalFile {
        path: std::path::PathBuf::from(".agents/project.md"),
        manifest_path: ".agents/project.md".to_string(),
        content: "Local alpha\nLocal beta\n".to_string(),
        position: "prepend".to_string(),
        optional: false,
    }];
    let summary = summarize_position_drift(
        "AGENTS.md",
        &spans,
        &markers,
        &on_disk,
        Some(&fresh),
        &locals,
    )
    .expect("summary");
    assert_eq!(summary.marker_count, 2);
    assert_eq!(summary.uniform_delta, Some(2));
    assert_eq!(summary.first_id, "aaaa1111");
    assert_eq!(summary.lock_open, 4);
    assert_eq!(summary.file_open, 6);
    let message = format_position_drift_message(&summary);
    assert!(message.contains("`AGENTS.md` position drift (+2 lines) across 2 markers"));
    assert!(message.contains("first marker `aaaa1111` lock 4:6, file 6:8"));
    assert!(
        message.contains("cause: `AGENTS.md:4` unmarked text differs from `.agents/project.md:2`")
    );
}

#[test]
fn skips_checksum_mismatched_spans() {
    let locked = [span("aaaa1111", 2, 4, "sha256:ideal")];
    let spans: Vec<&ManagedSpanRecord> = locked.iter().collect();
    let mut markers = BTreeMap::new();
    markers.insert("aaaa1111".to_string(), (3, 5, "sha256:edited".to_string()));
    let on_disk = [
        "text",
        "<!-- pray:aaaa1111 -->",
        "edited",
        "<!-- pray:aaaa1111 -->",
    ];
    assert!(
        summarize_position_drift("AGENTS.md", &spans, &markers, &on_disk, None, &[],).is_none()
    );
}
