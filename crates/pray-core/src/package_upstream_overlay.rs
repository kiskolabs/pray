use std::collections::BTreeMap;

pub enum OverlayFileChange {
    Changed,
    LocalOnly,
    Missing,
}

pub fn overlay_file_changes(
    local_content: &BTreeMap<String, Vec<u8>>,
    upstream_content: &BTreeMap<String, Vec<u8>>,
) -> Vec<(String, OverlayFileChange)> {
    let mut paths = BTreeMap::new();
    for path in local_content.keys().chain(upstream_content.keys()) {
        paths.insert(path.clone(), ());
    }
    let mut changes = Vec::new();
    for path in paths.keys() {
        match (local_content.get(path), upstream_content.get(path)) {
            (Some(local), Some(upstream)) if local != upstream => {
                changes.push((path.clone(), OverlayFileChange::Changed));
            }
            (Some(_), None) => changes.push((path.clone(), OverlayFileChange::LocalOnly)),
            (None, Some(_)) => changes.push((path.clone(), OverlayFileChange::Missing)),
            _ => {}
        }
    }
    changes
}

pub fn overlay_drift_line(
    fork: &str,
    upstream_name: &str,
    upstream_version: &str,
    path: &str,
    change: &OverlayFileChange,
) -> String {
    match change {
        OverlayFileChange::Changed => {
            format!("{fork} {path} differs from {upstream_name} {upstream_version}")
        }
        OverlayFileChange::LocalOnly => format!("{fork} {path} local"),
        OverlayFileChange::Missing => format!("{fork} {path} missing from fork"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_file_changes_name_local_and_missing_paths() {
        let local = BTreeMap::from([
            ("README.md".to_string(), b"edit".to_vec()),
            ("overlays/note.md".to_string(), b"local".to_vec()),
        ]);
        let upstream = BTreeMap::from([
            ("README.md".to_string(), b"up".to_vec()),
            ("exports/a.md".to_string(), b"a".to_vec()),
        ]);
        let changes = overlay_file_changes(&local, &upstream);
        assert!(changes.iter().any(|(path, change)| {
            path == "README.md" && matches!(change, OverlayFileChange::Changed)
        }));
        assert!(changes.iter().any(|(path, change)| {
            path == "overlays/note.md" && matches!(change, OverlayFileChange::LocalOnly)
        }));
        assert!(changes.iter().any(|(path, change)| {
            path == "exports/a.md" && matches!(change, OverlayFileChange::Missing)
        }));
    }
}
