use pray_core::render::{render_project, write_rendered_targets};
use pray_core::resolve::resolve_project_in_context;
use pray_core::resolve_context::ResolveOptions;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{stamp}"))
}

fn write_package(
    root: &Path,
    directory: &str,
    package_name: &str,
    export_name: &str,
    export_kind: &str,
    export_path: &str,
    body: &[u8],
) {
    let package_root = root.join(format!("packages/{directory}"));
    if let Some(parent) = Path::new(export_path).parent() {
        fs::create_dir_all(package_root.join(parent)).expect("export dirs");
    } else {
        fs::create_dir_all(&package_root).expect("package dir");
    }
    fs::write(
        package_root.join(format!("{directory}.prayspec")),
        format!(
            r#"
Package::Specification.new do |spec|
  spec.name = "{package_name}"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["{export_path}"]
  spec.exports = {{
    "{export_name}" => {{
      type: "{export_kind}",
      path: "{export_path}",
      summary: "{export_name}"
    }}
  }}
end
"#
        ),
    )
    .expect("prayspec");
    fs::write(package_root.join(export_path), body).expect("export body");
}

fn resolve_and_render(
    root: &Path,
) -> pray_core::PrayResult<Vec<pray_core::render::RenderedTarget>> {
    let project =
        resolve_project_in_context(&root.join("Prayfile"), root, &ResolveOptions::default())?;
    render_project(&project)
}

#[test]
fn compose_inlines_utf8_file_export_as_marked_span() {
    let root = unique_temp_dir("pray-compose-file");
    write_package(
        &root,
        "community",
        "sample/community",
        "contributing",
        "file",
        "exports/CONTRIBUTING.md",
        b"Be kind.\n",
    );
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
compose "CONTRIBUTING.md" do
  pray "sample/community", "~> 1.0", path: "packages/community"
end
"#,
    )
    .expect("prayfile");

    let rendered = resolve_and_render(&root).expect("render");
    assert_eq!(rendered.len(), 1);
    assert!(rendered[0].content.contains("<!-- pray:"));
    assert!(rendered[0].content.contains("Be kind."));
    assert!(!rendered[0].content.contains("# Agent context"));
    assert!(!rendered[0].managed_spans.is_empty());
}

#[test]
fn exclusive_file_stays_unmarked() {
    let root = unique_temp_dir("pray-exclusive-file");
    write_package(
        &root,
        "community",
        "sample/community",
        "contributing",
        "file",
        "exports/CONTRIBUTING.md",
        b"Be kind.\n",
    );
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
pray "sample/community", "~> 1.0", path: "packages/community", file: "CONTRIBUTING.md"
"#,
    )
    .expect("prayfile");

    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .expect("resolve");
    let rendered = render_project(&project).expect("render");
    assert!(rendered.is_empty());
    write_rendered_targets(&project, &rendered).expect("write");
    let dest = fs::read_to_string(root.join("CONTRIBUTING.md")).expect("dest");
    assert_eq!(dest, "Be kind.\n");
    assert!(!dest.contains("<!-- pray:"));
}

#[test]
fn compose_prefers_fragment_when_file_also_exists() {
    let root = unique_temp_dir("pray-compose-prefer-fragment");
    let package_root = root.join("packages/mixed");
    fs::create_dir_all(package_root.join("exports")).expect("dirs");
    fs::write(
        package_root.join("mixed.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/mixed"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["exports/notes.md", "exports/CONTRIBUTING.md"]
  spec.exports = {
    "notes" => { type: "fragment", path: "exports/notes.md" },
    "contributing" => { type: "file", path: "exports/CONTRIBUTING.md" }
  }
end
"#,
    )
    .expect("prayspec");
    fs::write(package_root.join("exports/notes.md"), "Fragment notes\n").expect("fragment");
    fs::write(
        package_root.join("exports/CONTRIBUTING.md"),
        "File contributing\n",
    )
    .expect("file");
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
compose "AGENTS.md" do
  pray "sample/mixed", "~> 1.0", path: "packages/mixed"
end
"#,
    )
    .expect("prayfile");

    let rendered = resolve_and_render(&root).expect("render");
    assert!(rendered[0].content.contains("Fragment notes"));
    assert!(!rendered[0].content.contains("File contributing"));
    assert!(rendered[0].content.contains("# Agent context"));
    assert!(rendered[0].content.contains(".agents/"));
}

#[test]
fn compose_of_binary_file_export_fails() {
    let root = unique_temp_dir("pray-compose-binary");
    write_package(
        &root,
        "blob",
        "sample/blob",
        "icon",
        "file",
        "exports/icon.md",
        &[0xff, 0xfe, 0x00],
    );
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
compose "ICON.md" do
  pray "sample/blob", "~> 1.0", path: "packages/blob"
end
"#,
    )
    .expect("prayfile");

    let error = resolve_and_render(&root).expect_err("binary");
    assert!(
        error.to_string().contains("binary") || error.to_string().contains("utf-8"),
        "{error}"
    );
}

#[test]
fn compose_json_names_file_destination() {
    let root = unique_temp_dir("pray-compose-json");
    write_package(
        &root,
        "rules",
        "sample/rules",
        "rules",
        "fragment",
        "exports/rules.md",
        b"Keep it small.\n",
    );
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
compose "config.json" do
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
"#,
    )
    .expect("prayfile");

    let error = resolve_and_render(&root).expect_err("json");
    let text = error.to_string();
    assert!(text.contains("JSON"), "{text}");
    assert!(text.contains("file: \"config.json\""), "{text}");
}

#[test]
fn compose_unknown_type_names_file_destination() {
    let root = unique_temp_dir("pray-compose-unknown");
    write_package(
        &root,
        "rules",
        "sample/rules",
        "rules",
        "fragment",
        "exports/rules.md",
        b"Keep it small.\n",
    );
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
compose ".zshrc" do
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
"#,
    )
    .expect("prayfile");

    let error = resolve_and_render(&root).expect_err("unknown");
    let text = error.to_string();
    assert!(text.contains("file: \".zshrc\""), "{text}");
}

#[test]
fn compose_header_false_suppresses_agents_banner() {
    let root = unique_temp_dir("pray-compose-header-off");
    write_package(
        &root,
        "rules",
        "sample/rules",
        "rules",
        "fragment",
        "exports/rules.md",
        b"Keep it small.\n",
    );
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
compose "AGENTS.md", header: false do
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
"#,
    )
    .expect("prayfile");

    let rendered = resolve_and_render(&root).expect("render");
    assert!(!rendered[0].content.contains("# Agent context"));
}

#[test]
fn compose_header_true_on_notes_omits_agents_path() {
    let root = unique_temp_dir("pray-compose-header-on");
    write_package(
        &root,
        "rules",
        "sample/rules",
        "rules",
        "fragment",
        "exports/rules.md",
        b"Keep it small.\n",
    );
    fs::write(
        root.join("Prayfile"),
        r#"
prayfile "1"
compose "NOTES.md", header: true do
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
"#,
    )
    .expect("prayfile");

    let rendered = resolve_and_render(&root).expect("render");
    assert!(rendered[0].content.contains("# Agent context"));
    assert!(!rendered[0].content.contains(".agents/"));
}

#[test]
fn unused_export_kinds_match_no_destination_role() {
    use pray_core::destination::export_kind_matches_role;
    use pray_core::manifest::ExportRole;
    for kind in ["template", "command", "rule", "asset", "bundle"] {
        assert!(!export_kind_matches_role(kind, ExportRole::Fragment));
        assert!(!export_kind_matches_role(kind, ExportRole::Folder));
        assert!(!export_kind_matches_role(kind, ExportRole::File));
    }
}

#[test]
fn adapters_parse_and_stay_unused() {
    let spec = pray_core::package_spec::parse_package_spec(
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/with-adapters"
  spec.version = "1.0.0"
  spec.files = ["exports/a.md"]
  spec.exports = { "a" => { type: "fragment", path: "exports/a.md" } }
  spec.adapters = { "tool_a" => "adapters/tool_a.toml" }
end
"#,
    )
    .expect("spec");
    assert_eq!(
        spec.adapters.get("tool_a").map(String::as_str),
        Some("adapters/tool_a.toml")
    );
}
