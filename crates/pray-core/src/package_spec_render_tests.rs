use super::*;
use crate::literal::LiteralValue;
use crate::package_spec::{PackageDependency, PackageExport, PackageSkill, PackageTemplate};
use crate::package_upstream::PackageUpstream;
use std::collections::BTreeMap;

fn complete_spec() -> PackageSpec {
    PackageSpec {
        name: "fork/base".to_string(),
        version: "1.0.0".to_string(),
        summary: Some("summary".to_string()),
        description: Some("description".to_string()),
        authors: vec!["Author".to_string()],
        license: Some("MIT".to_string()),
        homepage: Some("https://example.com".to_string()),
        source_code_uri: Some("https://example.com/source".to_string()),
        changelog_uri: Some("https://example.com/changelog".to_string()),
        prayfile_version: Some("1".to_string()),
        files: vec!["a.md".to_string(), "fork.prayspec".to_string()],
        exports: BTreeMap::from([(
            "a".to_string(),
            PackageExport {
                kind: "fragment".to_string(),
                path: "a.md".to_string(),
                summary: Some("A".to_string()),
                only: vec!["tool_a".to_string()],
                except: vec!["tool_b".to_string()],
                default_path: Some("A.md".to_string()),
            },
        )]),
        skills: BTreeMap::from([(
            "review".to_string(),
            PackageSkill {
                path: "skills/review".to_string(),
                summary: Some("Review".to_string()),
            },
        )]),
        templates: BTreeMap::from([(
            "note".to_string(),
            PackageTemplate {
                path: "templates/note.md".to_string(),
                summary: Some("Note".to_string()),
            },
        )]),
        adapters: BTreeMap::from([("tool".to_string(), "adapter".to_string())]),
        targets: vec!["tool_a".to_string()],
        dependencies: vec![PackageDependency {
            name: "sample/common".to_string(),
            constraint: "~> 1.0".to_string(),
            optional: true,
        }],
        metadata: BTreeMap::from([
            (
                "labels".to_string(),
                LiteralValue::Array(vec![
                    LiteralValue::String("stable".to_string()),
                    LiteralValue::Symbol("tool-a".to_string()),
                ]),
            ),
            (
                "policy".to_string(),
                LiteralValue::Map(BTreeMap::from([
                    ("enabled".to_string(), LiteralValue::Bool(true)),
                    ("fallback".to_string(), LiteralValue::Null),
                ])),
            ),
            ("priority".to_string(), LiteralValue::Integer(1)),
        ]),
        upstream: Some(PackageUpstream {
            name: "sample/base".to_string(),
            constraint: "~> 1.4".to_string(),
        }),
    }
}

#[test]
fn render_round_trips_every_supported_field() {
    let expected = complete_spec().canonicalized();
    let rendered = render_package_spec(&expected);
    let parsed = crate::package_spec::parse_package_spec(&rendered).expect("rendered spec parses");
    assert_eq!(parsed, expected);
}

#[test]
fn refresh_keeps_local_only_content_in_a_dirty_fork() {
    let mut local = complete_spec();
    local.files.push("extra.md".to_string());
    let mut upstream = complete_spec();
    upstream.name = "sample/base".to_string();
    upstream.files = vec!["a.md".to_string(), "sample.prayspec".to_string()];
    let merged = vec!["a.md".to_string(), "extra.md".to_string()];
    let refreshed = fork_spec_after_refresh(&local, &upstream, "fork.prayspec", false, &merged);
    assert!(refreshed.files.contains(&"fork.prayspec".to_string()));
    assert!(refreshed.files.contains(&"a.md".to_string()));
    assert!(refreshed.files.contains(&"extra.md".to_string()));
}
