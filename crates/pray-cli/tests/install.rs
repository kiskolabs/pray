use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[test]
fn installs_renders_and_verifies_a_local_package() {
    let repo = temporary_directory("pray-install");
    create_fixture(&repo);

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let rendered = fs::read_to_string(repo.join("INSTRUCTIONS.md")).expect("rendered file exists");
    assert!(rendered.contains("<!-- pray:"));

    let lockfile = repo.join("Prayfile.lock");
    let initial_modified = fs::metadata(&lockfile)
        .expect("lockfile exists")
        .modified()
        .expect("lockfile modified time");
    sleep(Duration::from_secs(1));

    let reinstall = run_pray(&repo, &["install"]);
    assert!(
        reinstall.status.success(),
        "reinstall failed: {}",
        String::from_utf8_lossy(&reinstall.stderr)
    );

    let next_modified = fs::metadata(&lockfile)
        .expect("lockfile exists")
        .modified()
        .expect("lockfile modified time");
    assert_eq!(initial_modified, next_modified);

    let verify = run_pray(&repo, &["verify"]);
    assert!(
        verify.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
}

#[test]
fn plan_reports_changes_without_writing_files() {
    let repo = temporary_directory("pray-plan");
    create_fixture(&repo);

    let plan = run_pray(&repo, &["plan"]);
    assert!(
        plan.status.success(),
        "plan failed: {}",
        String::from_utf8_lossy(&plan.stderr)
    );
    assert!(!repo.join("Prayfile.lock").exists());
    assert!(!repo.join("INSTRUCTIONS.md").exists());
    let stdout = String::from_utf8_lossy(&plan.stdout);
    assert!(stdout.contains("Prayfile.lock"));
    assert!(stdout.contains("INSTRUCTIONS.md"));
}

#[test]
fn apply_materializes_like_install() {
    let repo = temporary_directory("pray-apply");
    create_fixture(&repo);

    let apply = run_pray(&repo, &["apply"]);
    assert!(
        apply.status.success(),
        "apply failed: {}",
        String::from_utf8_lossy(&apply.stderr)
    );
    assert!(repo.join("Prayfile.lock").exists());
    assert!(repo.join("INSTRUCTIONS.md").exists());

    let verify = run_pray(&repo, &["verify"]);
    assert!(
        verify.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
}

#[test]
fn add_remove_and_update_package_declaration() {
    let repo = temporary_directory("pray-add-remove-update");
    create_add_fixture(&repo);

    let add = run_pray(&repo, &["add", "sample/base", "--path", "packages/base"]);
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let manifest = fs::read_to_string(repo.join("Prayfile")).expect("manifest exists");
    assert!(manifest.contains("agent \"sample/base\", path: \"packages/base\""));

    let install = run_pray(&repo, &["install"]);
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    fs::write(
        repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.4"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
end
"#,
    )
    .expect("rewrite prayspec");

    let update = run_pray(&repo, &["update", "sample/base"]);
    assert!(
        update.status.success(),
        "update failed: {}",
        String::from_utf8_lossy(&update.stderr)
    );
    let stdout = String::from_utf8_lossy(&update.stdout);
    assert!(stdout.contains("Update summary"));
    assert!(stdout.contains("sample/base 1.4.3 -> 1.4.4"));
    assert!(stdout.contains("source: path:packages/base"));
    assert!(stdout.contains("exports affected: testing-basics"));
    assert!(stdout.contains("targets affected: tool_a"));
    assert!(stdout.contains("rendered files affected: INSTRUCTIONS.md"));
    assert!(stdout.contains("warnings: none"));
    let lockfile = fs::read_to_string(repo.join("Prayfile.lock")).expect("lockfile exists");
    assert!(lockfile.contains("1.4.4"));

    let remove = run_pray(&repo, &["remove", "sample/base"]);
    assert!(
        remove.status.success(),
        "remove failed: {}",
        String::from_utf8_lossy(&remove.stderr)
    );
    let manifest = fs::read_to_string(repo.join("Prayfile")).expect("manifest exists");
    assert!(!manifest.contains("sample/base"));
    let lockfile = fs::read_to_string(repo.join("Prayfile.lock")).expect("lockfile exists");
    assert!(!lockfile.contains("sample/base"));
    let rendered = fs::read_to_string(repo.join("INSTRUCTIONS.md")).expect("rendered file exists");
    assert!(!rendered.contains("Testing guidance"));
}

#[test]
fn tree_reports_dependency_graph() {
    let repo = temporary_directory("pray-tree");
    create_tree_fixture(&repo);

    let tree = run_pray(&repo, &["tree"]);
    assert!(
        tree.status.success(),
        "tree failed: {}",
        String::from_utf8_lossy(&tree.stderr)
    );

    let stdout = String::from_utf8_lossy(&tree.stdout);
    assert!(stdout.contains("Dependency tree"));
    assert!(stdout.contains("sample/base 1.4.3"));
    assert!(stdout.contains("sample/common 1.0.0"));
    assert!(stdout.contains("  sample/common 1.0.0"));
}

#[test]
fn clean_removes_local_ephemeral_state() {
    let repo = temporary_directory("pray-clean");
    create_add_fixture(&repo);
    fs::create_dir_all(repo.join(".pray/cache")).expect("cache directory");
    fs::create_dir_all(repo.join(".pray/vendor")).expect("vendor directory");
    fs::write(repo.join(".pray/state.json"), "{}\n").expect("state file");
    fs::write(repo.join(".pray/cache/item.bin"), "cached\n").expect("cache file");
    fs::write(repo.join(".pray/vendor/item.bin"), "vendored\n").expect("vendor file");

    let clean = run_pray(&repo, &["clean"]);
    assert!(
        clean.status.success(),
        "clean failed: {}",
        String::from_utf8_lossy(&clean.stderr)
    );
    assert!(!repo.join(".pray/cache").exists());
    assert!(!repo.join(".pray/vendor").exists());
    assert!(!repo.join(".pray/state.json").exists());
}

#[test]
fn format_normalizes_pray_markers_and_line_endings() {
    let repo = temporary_directory("pray-format");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered
        .replace("<!-- pray:", "<!--  pray:")
        .replace(" -->", "   -->")
        .replace("\n", "\r\n");
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let format = run_pray(&repo, &["format"]);
    assert!(
        format.status.success(),
        "format failed: {}",
        String::from_utf8_lossy(&format.stderr)
    );

    let formatted = fs::read_to_string(&rendered_path).expect("formatted file exists");
    assert!(!formatted.contains("\r"));
    assert!(formatted.contains("<!-- pray:"));
    assert!(formatted.contains("<!-- pray:0 ignore-comments -->"));
    assert!(!formatted.contains("<!--  pray:"));
    assert!(!formatted.contains("   -->"));

    let verify = run_pray(&repo, &["verify"]);
    assert!(
        verify.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
}

#[test]
fn package_builds_a_tar_zst_archive_from_package_contents() {
    let repo = temporary_directory("pray-package");
    create_add_fixture(&repo);

    let add = run_pray(&repo, &["add", "sample/base", "--path", "packages/base"]);
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let package = run_pray(&repo, &["package"]);
    assert!(
        package.status.success(),
        "package failed: {}",
        String::from_utf8_lossy(&package.stderr)
    );

    let archive = repo.join("sample-base-1.4.3.praypkg");
    assert!(archive.is_file());

    let entries = read_package_archive(&archive);
    let metadata = entries.get("metadata.json").expect("metadata");
    assert!(metadata.contains("\"name\": \"sample/base\""));
    assert!(metadata.contains("\"version\": \"1.4.3\""));
    assert!(metadata.contains("\"files\": ["));
    assert!(metadata.contains("README.md"));
    assert!(metadata.contains("exports/testing-basics.md"));
    assert_eq!(
        entries.get("README.md").expect("archive readme"),
        "package readme\n"
    );
    assert_eq!(
        entries
            .get("exports/testing-basics.md")
            .expect("archive export"),
        "Testing guidance\n"
    );
}

#[test]
fn update_rejects_unknown_package_selection() {
    let repo = temporary_directory("pray-update-unknown");
    create_add_fixture(&repo);

    let update = run_pray(&repo, &["update", "missing/base"]);
    assert!(!update.status.success());
    assert_eq!(update.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&update.stderr);
    assert!(stderr.contains("package missing/base not found"));
}

#[test]
fn update_refreshes_only_the_selected_package_version() {
    let repo = temporary_directory("pray-update-selected");
    create_tree_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    fs::write(
        repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.4"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
  spec.add_dependency "sample/common", "~> 1.0"
end
"#,
    )
    .expect("rewrite base prayspec");
    fs::write(
        repo.join("packages/common/sample-common.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/common"
  spec.version = "1.1.0"
  spec.summary = "common guidance"
  spec.files = ["README.md", "exports/common-basics.md"]
  spec.exports = {
    "common-basics" => {
      type: "fragment",
      path: "exports/common-basics.md",
      summary: "Common guidance"
    }
  }
end
"#,
    )
    .expect("rewrite common prayspec");

    let update = run_pray(&repo, &["update", "sample/base"]);
    assert!(
        update.status.success(),
        "update failed: {}",
        String::from_utf8_lossy(&update.stderr)
    );
    let stdout = String::from_utf8_lossy(&update.stdout);
    assert!(stdout.contains("sample/base 1.4.3 -> 1.4.4"));
    assert!(!stdout.contains("dependent packages affected"));

    let lockfile = fs::read_to_string(repo.join("Prayfile.lock")).expect("lockfile exists");
    assert!(lockfile.contains("sample/base"));
    assert!(lockfile.contains("1.4.4"));
    assert!(lockfile.contains("sample/common"));
    assert!(lockfile.contains("1.0.0"));
    assert!(!lockfile.contains("1.1.0"));
}

#[test]
fn update_reports_dependent_packages_affected() {
    let repo = temporary_directory("pray-update-dependent");
    create_tree_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    fs::write(
        repo.join("packages/common/sample-common.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/common"
  spec.version = "1.1.0"
  spec.summary = "common guidance"
  spec.files = ["README.md", "exports/common-basics.md"]
  spec.exports = {
    "common-basics" => {
      type: "fragment",
      path: "exports/common-basics.md",
      summary: "Common guidance"
    }
  }
end
"#,
    )
    .expect("rewrite common prayspec");

    let update = run_pray(&repo, &["update", "sample/common"]);
    assert!(
        update.status.success(),
        "update failed: {}",
        String::from_utf8_lossy(&update.stderr)
    );
    let stdout = String::from_utf8_lossy(&update.stdout);
    assert!(stdout.contains("sample/common 1.0.0 -> 1.1.0"));
    assert!(stdout.contains("dependent packages affected: sample/base"));
    assert!(stdout.contains("\"updated_packages\""));
    assert!(stdout.contains("\"dependent_packages_affected\""));

    let lockfile = fs::read_to_string(repo.join("Prayfile.lock")).expect("lockfile exists");
    assert!(lockfile.contains("sample/common"));
    assert!(lockfile.contains("1.1.0"));
    assert!(lockfile.contains("sample/base"));
    assert!(lockfile.contains("1.4.3"));
}

#[test]
fn vendor_copies_package_contents_into_pray_vendor() {
    let repo = temporary_directory("pray-vendor");
    create_add_fixture(&repo);

    let add = run_pray(&repo, &["add", "sample/base", "--path", "packages/base"]);
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let vendor = run_pray(&repo, &["vendor"]);
    assert!(
        vendor.status.success(),
        "vendor failed: {}",
        String::from_utf8_lossy(&vendor.stderr)
    );

    let vendored = repo.join(".pray/vendor/sample-base/1.4.3");
    assert!(vendored.is_dir());
    assert!(vendored.join("metadata.json").exists());
    assert_eq!(
        fs::read_to_string(vendored.join("README.md")).expect("vendored readme"),
        "package readme\n"
    );
    assert_eq!(
        fs::read_to_string(vendored.join("exports/testing-basics.md")).expect("vendored export"),
        "Testing guidance\n"
    );
}

#[test]
fn verify_reports_custom_implementation() {
    let repo = temporary_directory("pray-verify");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let mut rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    rendered = rendered.replace("Testing guidance", "Changed guidance");
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let verify = run_pray(&repo, &["verify"]);
    assert!(!verify.status.success());
    assert_eq!(verify.status.code(), Some(6));
    let stderr = String::from_utf8_lossy(&verify.stderr);
    assert!(stderr.contains("custom_implementation") || stderr.contains("verify error"));
}

#[test]
fn verify_warns_on_orphan_markers_and_strict_fails() {
    let repo = temporary_directory("pray-verify-orphan-marker");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let mut rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    rendered.push_str("<!-- pray:abc123 -->\nOrphan marker body\n<!-- pray:abc123 -->\n");
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let verify = run_pray(&repo, &["verify"]);
    assert!(verify.status.success());
    let stderr = String::from_utf8_lossy(&verify.stderr);
    assert!(stderr.contains("orphan_marker"));

    let strict_verify = run_pray(&repo, &["verify", "--strict"]);
    assert!(!strict_verify.status.success());
    assert_eq!(strict_verify.status.code(), Some(6));
    let strict_stderr = String::from_utf8_lossy(&strict_verify.stderr);
    assert!(strict_stderr.contains("orphan_marker"));
}

#[test]
fn install_locked_rejects_lockfile_drift() {
    let repo = temporary_directory("pray-install-locked");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    fs::write(
        repo.join("agent/local/project.md"),
        "Local guidance\nExtra local guidance\n",
    )
    .expect("rewrite local file");

    let locked = run_pray(&repo, &["install", "--locked"]);
    assert!(!locked.status.success());
    assert_eq!(locked.status.code(), Some(6));
    let stderr = String::from_utf8_lossy(&locked.stderr);
    assert!(stderr.contains("lockfile needs update") || stderr.contains("verify error"));
}

#[test]
fn install_frozen_rejects_stale_rendered_output() {
    let repo = temporary_directory("pray-install-frozen");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered.replace(
        "Do not edit managed blocks or managed skills.",
        "Managed blocks stay read-only.",
    );
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let frozen = run_pray(&repo, &["install", "--frozen"]);
    assert!(!frozen.status.success());
    assert_eq!(frozen.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&frozen.stderr);
    assert!(stderr.contains("stale") || stderr.contains("render error"));
}

#[test]
fn install_offline_accepts_explicit_local_paths() {
    let repo = temporary_directory("pray-install-offline");
    create_fixture(&repo);

    let offline = run_pray(&repo, &["install", "--offline"]);
    assert!(
        offline.status.success(),
        "offline install failed: {}",
        String::from_utf8_lossy(&offline.stderr)
    );
}

#[test]
fn install_offline_rejects_derived_package_paths() {
    let repo = temporary_directory("pray-install-offline-derived");
    create_derived_fixture(&repo);

    let offline = run_pray(&repo, &["install", "--offline"]);
    assert!(!offline.status.success());
    assert_eq!(offline.status.code(), Some(8));
    let stderr = String::from_utf8_lossy(&offline.stderr);
    assert!(stderr.contains("offline mode") || stderr.contains("unsupported feature"));
}

#[test]
fn publish_serve_install_and_confess_end_to_end_with_web_surface() {
    let workspace = temporary_directory("pray-publish-e2e");
    let source_repo = workspace.join("source");
    let registry_root = workspace.join("registry");
    let client_a = workspace.join("client-a");
    let client_b = workspace.join("client-b");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&registry_root).expect("registry workspace");
    fs::create_dir_all(&client_a).expect("client A workspace");
    fs::create_dir_all(&client_b).expect("client B workspace");

    create_add_fixture(&source_repo);
    let add = run_pray(
        &source_repo,
        &["add", "sample/base", "--path", "packages/base"],
    );
    assert!(
        add.status.success(),
        "add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let publish = Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "publish",
            "--root",
            registry_root.to_str().expect("registry path"),
        ])
        .current_dir(&source_repo)
        .env("PRAY_SIGNER", "sample-agent-packages-2026")
        .output()
        .expect("run publish");
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    let index_text =
        fs::read_to_string(registry_root.join("v1/index.json")).expect("registry index");
    assert!(index_text.contains("prayfile-distribution-1"));
    assert!(index_text.contains("sample/base"));

    let metadata_text = fs::read_to_string(registry_root.join("v1/packages/sample/base.json"))
        .expect("package metadata");
    let metadata: Value = serde_json::from_str(&metadata_text).expect("package metadata json");
    let version = metadata
        .get("versions")
        .and_then(Value::as_array)
        .and_then(|versions| versions.first())
        .expect("published version");
    let artifact_path = version
        .get("artifact")
        .and_then(Value::as_str)
        .expect("artifact path");
    let signature = version
        .get("signature")
        .and_then(Value::as_str)
        .expect("signature");
    let signer = version
        .get("signer")
        .and_then(Value::as_str)
        .expect("signer");
    assert_eq!(signer, "sample-agent-packages-2026");
    assert!(signature.starts_with("sha256:"));
    assert!(registry_root.join(artifact_path).is_file());

    let port = find_free_port();
    let mut server = Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "serve",
            "--root",
            registry_root.to_str().expect("registry path"),
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn server");
    wait_for_server(port);

    let server_url = format!("http://127.0.0.1:{port}");
    write_registry_client_fixture(&client_a, &server_url);
    write_registry_client_fixture(&client_b, &server_url);

    let ruby_script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/distribution_point_smoke.rb");
    let smoke = Command::new("ruby")
        .arg(ruby_script)
        .arg("--pray-bin")
        .arg(env!("CARGO_BIN_EXE_pray"))
        .arg("--server-url")
        .arg(&server_url)
        .arg("--client")
        .arg(&client_a)
        .arg("--client")
        .arg(&client_b)
        .output()
        .expect("run ruby smoke test");

    let _ = server.kill();
    let _ = server.wait();

    assert!(
        smoke.status.success(),
        "ruby smoke test failed: {}",
        String::from_utf8_lossy(&smoke.stderr)
    );
    let stdout = String::from_utf8_lossy(&smoke.stdout);
    assert!(stdout.contains("distribution point smoke test passed"));
}

#[test]
fn drift_reports_renderer_changes_in_sections() {
    let repo = temporary_directory("pray-drift-renderer");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    fs::write(
        repo.join("agent/local/project.md"),
        "Local guidance\nChanged local guidance\n",
    )
    .expect("rewrite local file");

    let drift = run_pray(&repo, &["drift"]);
    assert!(!drift.status.success());
    assert_eq!(drift.status.code(), Some(6));
    let stderr = String::from_utf8_lossy(&drift.stderr);
    assert!(stderr.contains("Rendered file changes"));
    assert!(stderr.contains("renderer_drift"));
}

#[test]
fn drift_reports_position_changes_in_sections() {
    let repo = temporary_directory("pray-drift-position");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered.replace(
        "## Shared instructions\n\n<!-- pray:",
        "## Shared instructions\n\n\n<!-- pray:",
    );
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let drift = run_pray(&repo, &["drift"]);
    assert!(!drift.status.success());
    assert_eq!(drift.status.code(), Some(6));
    let stderr = String::from_utf8_lossy(&drift.stderr);
    assert!(stderr.contains("Managed span changes"));
    assert!(stderr.contains("position_drift"));
}

#[test]
fn drift_semantic_summarizes_package_version_changes() {
    let repo = temporary_directory("pray-drift-semantic");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    fs::write(
        repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.4"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md", "exports/security-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    },
    "security-basics" => {
      type: "fragment",
      path: "exports/security-basics.md",
      summary: "Security guidance"
    }
  }
end
"#,
    )
    .expect("rewrite prayspec");
    fs::write(
        repo.join("packages/base/exports/security-basics.md"),
        "Security guidance\n",
    )
    .expect("write second export");

    let semantic = run_pray(&repo, &["drift", "--semantic"]);
    assert!(!semantic.status.success());
    assert_eq!(semantic.status.code(), Some(6));
    let stderr = String::from_utf8_lossy(&semantic.stderr);
    assert!(stderr.contains("Semantic diff"));
    assert!(stderr.contains("sample/base 1.4.3 -> 1.4.4 would change 2 managed spans"));
}

fn write_registry_client_fixture(repo: &Path, server_url: &str) {
    fs::create_dir_all(repo).expect("client repo");
    fs::write(
        repo.join("Prayfile"),
        format!(
            r#"
prayfile "1"
source "default", "{server_url}"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", "~> 1.4", source: "default"
render mode: :managed, conflict: :fail, churn: :minimal
"#,
        ),
    )
    .expect("write client Prayfile");
}

fn find_free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("reserve port");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);
    port
}

fn wait_for_server(port: u16) {
    for _ in 0..50 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        sleep(Duration::from_millis(100));
    }
    panic!("server did not start on port {port}");
}

fn create_fixture(repo: &Path) {
    fs::create_dir_all(repo.join("packages/base/exports")).expect("fixture directories");
    fs::create_dir_all(repo.join("agent/local")).expect("local directories");

    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", "~> 1.4", path: "packages/base"
local "agent/local/project.md"
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");

    fs::write(
        repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md", "exports/security-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    },
    "security-basics" => {
      type: "fragment",
      path: "exports/security-basics.md",
      summary: "Security guidance"
    }
  }
end
"#,
    )
    .expect("write prayspec");

    fs::write(repo.join("packages/base/README.md"), "package readme\n").expect("write readme");
    fs::write(
        repo.join("packages/base/exports/testing-basics.md"),
        "Testing guidance\n",
    )
    .expect("write export");
    fs::write(
        repo.join("packages/base/exports/security-basics.md"),
        "Security guidance\n",
    )
    .expect("write export");
    fs::write(repo.join("agent/local/project.md"), "Local guidance\n").expect("write local");
}

fn create_add_fixture(repo: &Path) {
    fs::create_dir_all(repo.join("packages/base/exports")).expect("fixture directories");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");

    fs::write(
        repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
end
"#,
    )
    .expect("write prayspec");

    fs::write(repo.join("packages/base/README.md"), "package readme\n").expect("write readme");
    fs::write(
        repo.join("packages/base/exports/testing-basics.md"),
        "Testing guidance\n",
    )
    .expect("write export");
}

fn create_tree_fixture(repo: &Path) {
    fs::create_dir_all(repo.join("packages/base/exports")).expect("base directories");
    fs::create_dir_all(repo.join("packages/common/exports")).expect("common directories");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", path: "packages/base"
agent "sample/common", path: "packages/common"
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");

    fs::write(
        repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
  spec.add_dependency "sample/common", "~> 1.0"
end
"#,
    )
    .expect("write base prayspec");

    fs::write(repo.join("packages/base/README.md"), "package readme\n").expect("write readme");
    fs::write(
        repo.join("packages/base/exports/testing-basics.md"),
        "Testing guidance\n",
    )
    .expect("write export");

    fs::write(
        repo.join("packages/common/sample-common.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/common"
  spec.version = "1.0.0"
  spec.summary = "common guidance"
  spec.files = ["README.md", "exports/common-basics.md"]
  spec.exports = {
    "common-basics" => {
      type: "fragment",
      path: "exports/common-basics.md",
      summary: "Common guidance"
    }
  }
end
"#,
    )
    .expect("write common prayspec");

    fs::write(repo.join("packages/common/README.md"), "package readme\n").expect("write readme");
    fs::write(
        repo.join("packages/common/exports/common-basics.md"),
        "Common guidance\n",
    )
    .expect("write export");
}

fn create_derived_fixture(repo: &Path) {
    fs::create_dir_all(repo.join("sample-base/exports")).expect("fixture directories");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", "~> 1.4"
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");

    fs::write(
        repo.join("sample-base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
end
"#,
    )
    .expect("write prayspec");

    fs::write(repo.join("sample-base/README.md"), "package readme\n").expect("write readme");
    fs::write(
        repo.join("sample-base/exports/testing-basics.md"),
        "Testing guidance\n",
    )
    .expect("write export");
}

fn read_package_archive(path: &Path) -> BTreeMap<String, String> {
    let file = fs::File::open(path).expect("open package archive");
    let decoder = zstd::stream::read::Decoder::new(file).expect("decode archive");
    let mut archive = tar::Archive::new(decoder);
    let mut entries = BTreeMap::new();
    for entry in archive.entries().expect("read archive entries") {
        let mut entry = entry.expect("archive entry");
        if entry.header().entry_type().is_dir() {
            continue;
        }
        let path = entry
            .path()
            .expect("entry path")
            .to_string_lossy()
            .to_string();
        let mut content = String::new();
        entry.read_to_string(&mut content).expect("entry contents");
        entries.insert(path, content);
    }
    entries
}

fn run_pray(repo: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_pray"))
        .args(arguments)
        .current_dir(repo)
        .output()
        .expect("run pray")
}

fn temporary_directory(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!("{prefix}-{stamp}-{suffix}"));
    fs::create_dir_all(&path).expect("temp dir");
    path
}
