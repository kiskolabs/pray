use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use pray_core::auth::RegistryAuthStore;
use pray_core::trust::EmailConfirmationPolicy;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
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
fn federation_endpoints_expose_discovery_and_sync_metadata() {
    let registry_root = temporary_directory("pray-federation-http");
    fs::create_dir_all(registry_root.join("v1/packages/sample")).expect("registry directories");
    fs::write(
        registry_root.join("v1/index.json"),
        r#"{
            "spec": "prayfile-distribution-1",
            "packages": ["sample/base"]
        }"#,
    )
    .expect("write index");
    fs::write(
        registry_root.join("v1/peers.json"),
        r#"[
            {
                "name": "seed-one",
                "url": "https://seed-one.example",
                "public": true
            }
        ]"#,
    )
    .expect("write peers");
    fs::write(
        registry_root.join("v1/packages/sample/base.json"),
        r#"{
            "name": "sample/base",
            "versions": [
                {
                    "version": "1.0.0",
                    "artifact": "v1/artifacts/sample/base/1.0.0/sample-base-1.0.0.praypkg",
                    "artifact_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "tree_hash": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                    "yanked": false,
                    "targets": ["tool_a"],
                    "exports": ["basics"],
                    "published_at": "1711111111",
                    "signer": "publisher-a",
                    "signature": "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                }
            ]
        }"#,
    )
    .expect("write package metadata");

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

    let base_url = format!("http://127.0.0.1:{port}");

    let discovery = fetch_http_get(&format!("{base_url}/.well-known/pray-federation.json"));
    assert_eq!(discovery.status, 200);
    let discovery_json: Value = serde_json::from_str(&discovery.body).expect("discovery json");
    assert_eq!(discovery_json["spec"], "pray-federation-v1");
    assert_eq!(discovery_json["server"]["name"], "pray");
    assert!(discovery_json["server"]["capabilities"]
        .as_array()
        .expect("capabilities")
        .iter()
        .any(|value| value.as_str() == Some("federation")));
    assert_eq!(discovery_json["peers"].as_array().expect("peers").len(), 1);

    let index = fetch_http_get(&format!("{base_url}/v1/sync/index"));
    assert_eq!(index.status, 200);
    let index_json: Value = serde_json::from_str(&index.body).expect("index json");
    assert_eq!(index_json["spec"], "prayfile-distribution-1");
    assert_eq!(
        index_json["packages"].as_array().expect("packages").len(),
        1
    );
    assert_eq!(index_json["packages"][0]["name"], "sample/base");
    assert_eq!(index_json["packages"][0]["updated_at"], "1711111111");

    let filtered_index = fetch_http_get(&format!("{base_url}/v1/sync/index?since=1711111111"));
    assert_eq!(filtered_index.status, 200);
    let filtered_json: Value =
        serde_json::from_str(&filtered_index.body).expect("filtered index json");
    assert_eq!(
        filtered_json["packages"]
            .as_array()
            .expect("filtered packages")
            .len(),
        0
    );

    let package = fetch_http_get(&format!("{base_url}/v1/sync/package/sample/base"));
    assert_eq!(package.status, 200);
    let package_json: Value = serde_json::from_str(&package.body).expect("package json");
    assert_eq!(package_json["name"], "sample/base");
    assert_eq!(package_json["versions"][0]["published_at"], "1711111111");
    assert_eq!(
        package_json["versions"][0]["publisher"]["id"],
        "publisher-a"
    );

    let pushed = fetch_http_post(
        &format!("{base_url}/v1/sync/push"),
        r#"{
            "name": "sample/extra",
            "versions": [
                {
                    "version": "2.0.0",
                    "artifact": "v1/artifacts/sample/extra/2.0.0/sample-extra-2.0.0.praypkg",
                    "artifact_hash": "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
                    "tree_hash": "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                    "yanked": false,
                    "targets": ["tool_a"],
                    "exports": ["extra"],
                    "published_at": "1712222222",
                    "publisher": {
                        "id": "publisher-b",
                        "key_fingerprint": "publisher-b"
                    },
                    "signature": {
                        "algorithm": "sha256",
                        "signature": "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                        "public_key": "publisher-b"
                    },
                    "origin": {
                        "server": "peer-one",
                        "first_seen": "1712222222"
                    }
                }
            ]
        }"#,
    );
    assert_eq!(pushed.status, 201);

    let pushed_metadata_text =
        fs::read_to_string(registry_root.join("v1/packages/sample/extra.json"))
            .expect("pushed package metadata");
    let pushed_metadata: Value =
        serde_json::from_str(&pushed_metadata_text).expect("pushed metadata json");
    assert_eq!(pushed_metadata["name"], "sample/extra");
    assert_eq!(pushed_metadata["versions"][0]["signer"], "publisher-b");

    let updated_index_text =
        fs::read_to_string(registry_root.join("v1/index.json")).expect("updated index");
    let updated_index: Value =
        serde_json::from_str(&updated_index_text).expect("updated index json");
    assert!(updated_index["packages"]
        .as_array()
        .expect("updated packages")
        .iter()
        .any(|value| value.as_str() == Some("sample/extra")));

    let _ = server.kill();
    let _ = server.wait();
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
    let registry_root_mirror = workspace.join("registry-mirror");
    let client_a = workspace.join("client-a");
    let client_b = workspace.join("client-b");
    fs::create_dir_all(&source_repo).expect("source workspace");
    fs::create_dir_all(&registry_root).expect("registry workspace");
    fs::create_dir_all(&registry_root_mirror).expect("registry mirror workspace");
    fs::create_dir_all(registry_root.join("v1")).expect("registry v1 workspace");
    fs::write(
        registry_root.join("v1/index.json"),
        r#"{
            "spec": "prayfile-distribution-1",
            "packages": []
        }"#,
    )
    .expect("write registry index");
    fs::write(
        registry_root.join("v1/trust.json"),
        r#"{
            "email_confirmation": "required",
            "passkeys_enabled": true,
            "ssh_keys_enabled": true,
            "ssh_agent_signing_enabled": true
        }"#,
    )
    .expect("write trust settings");
    fs::create_dir_all(registry_root_mirror.join("v1")).expect("registry mirror v1 workspace");
    fs::write(
        registry_root_mirror.join("v1/index.json"),
        r#"{
            "spec": "prayfile-distribution-1",
            "packages": []
        }"#,
    )
    .expect("write mirror index");
    fs::write(
        registry_root_mirror.join("v1/trust.json"),
        r#"{
            "email_confirmation": "required",
            "passkeys_enabled": true,
            "ssh_keys_enabled": true,
            "ssh_agent_signing_enabled": true
        }"#,
    )
    .expect("write mirror trust settings");
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

    let auth_store = RegistryAuthStore::open(&registry_root).expect("open auth store");
    let publisher_email = "sample-agent-packages@example.com";
    let client_a_email = "client-a@example.com";
    let client_b_email = "client-b@example.com";

    let publisher_key = signing_key_from_seed(11);
    let client_a_key = signing_key_from_seed(12);
    let client_b_key = signing_key_from_seed(13);
    let publisher_public_key = ssh_public_key_text(&publisher_key);
    let client_a_public_key = ssh_public_key_text(&client_a_key);
    let client_b_public_key = ssh_public_key_text(&client_b_key);

    verify_email_registration(&auth_store, publisher_email);
    verify_email_registration(&auth_store, client_a_email);
    verify_email_registration(&auth_store, client_b_email);

    auth_store
        .enroll_passkey(
            publisher_email,
            "publisher-passkey",
            &publisher_public_key,
            Some("publisher passkey"),
        )
        .expect("enroll publisher passkey");
    auth_store
        .enroll_passkey(
            client_a_email,
            "client-a-passkey",
            &client_a_public_key,
            Some("client A passkey"),
        )
        .expect("enroll client A passkey");
    auth_store
        .enroll_ssh_key(
            client_b_email,
            &client_b_public_key,
            Some("client B workstation"),
        )
        .expect("enroll client B ssh key");

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

    let publisher_private_key_path =
        write_private_key_file(&source_repo, "publisher-passkey.bin", &publisher_key);
    let client_a_private_key_path =
        write_private_key_file(&client_a, "client-a-passkey.bin", &client_a_key);
    let client_b_public_key_path =
        write_public_key_file(&client_b, "client-b-public-key.pub", &client_b_public_key);

    run_pray_login_passkey(
        &source_repo,
        &server_url,
        publisher_email,
        "publisher-passkey",
        &publisher_private_key_path,
    );
    run_pray_login_passkey(
        &client_a,
        &server_url,
        client_a_email,
        "client-a-passkey",
        &client_a_private_key_path,
    );

    let ssh_agent_socket = PathBuf::from("/tmp/pray-ssh-agent.sock");
    let ssh_agent_handle = spawn_mock_ssh_agent(&ssh_agent_socket, client_b_key);
    run_pray_login_ssh_agent(
        &client_b,
        &server_url,
        client_b_email,
        &client_b_public_key_path,
        &ssh_agent_socket,
    );
    ssh_agent_handle.join().expect("ssh agent finished");

    let publish = run_pray(
        &source_repo,
        &[
            "publish",
            "--root",
            registry_root.to_str().expect("registry path"),
            "--root",
            registry_root_mirror.to_str().expect("registry mirror path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );

    let index_text =
        fs::read_to_string(registry_root.join("v1/index.json")).expect("registry index");
    assert!(index_text.contains("prayfile-distribution-1"));
    assert!(index_text.contains("sample/base"));

    let mirror_index_text = fs::read_to_string(registry_root_mirror.join("v1/index.json"))
        .expect("registry mirror index");
    assert!(mirror_index_text.contains("prayfile-distribution-1"));
    assert!(mirror_index_text.contains("sample/base"));

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
    assert_eq!(signer, publisher_email);
    assert!(signature.starts_with("sha256:"));
    assert!(registry_root.join(artifact_path).is_file());
    assert!(registry_root_mirror.join(artifact_path).is_file());

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
fn login_supports_multiple_auth_origins() {
    let workspace = temporary_directory("pray-login-multi-origin");
    let client_repo = workspace.join("client");
    let auth_root_a = workspace.join("auth-a");
    let auth_root_b = workspace.join("auth-b");
    fs::create_dir_all(&client_repo).expect("client workspace");
    fs::create_dir_all(&auth_root_a).expect("auth root A workspace");
    fs::create_dir_all(&auth_root_b).expect("auth root B workspace");
    write_auth_registry_fixture(&auth_root_a);
    write_auth_registry_fixture(&auth_root_b);

    let auth_store_a = RegistryAuthStore::open(&auth_root_a).expect("open auth store A");
    let auth_store_b = RegistryAuthStore::open(&auth_root_b).expect("open auth store B");
    let login_email = "multi-origin@example.com";
    let signing_key = signing_key_from_seed(21);
    let public_key = ssh_public_key_text(&signing_key);

    verify_email_registration(&auth_store_a, login_email);
    verify_email_registration(&auth_store_b, login_email);
    auth_store_a
        .enroll_passkey(
            login_email,
            "multi-origin-passkey",
            &public_key,
            Some("auth origin A"),
        )
        .expect("enroll passkey on auth origin A");
    auth_store_b
        .enroll_passkey(
            login_email,
            "multi-origin-passkey",
            &public_key,
            Some("auth origin B"),
        )
        .expect("enroll passkey on auth origin B");

    let port_a = find_free_port();
    let port_b = find_free_port();
    let mut server_a = spawn_server(&auth_root_a, port_a);
    let mut server_b = spawn_server(&auth_root_b, port_b);
    wait_for_server(port_a);
    wait_for_server(port_b);

    let server_url_a = format!("http://127.0.0.1:{port_a}");
    let server_url_b = format!("http://127.0.0.1:{port_b}");
    let private_key_path =
        write_private_key_file(&client_repo, "multi-origin-passkey.bin", &signing_key);

    let login = Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "login",
            "--server",
            &server_url_a,
            "--server",
            &server_url_b,
            "--email",
            login_email,
            "--credential-id",
            "multi-origin-passkey",
            "--passkey-key",
            private_key_path.to_str().expect("private key path"),
        ])
        .current_dir(&client_repo)
        .output()
        .expect("run multi-origin login");
    assert!(
        login.status.success(),
        "multi-origin login failed: {}",
        String::from_utf8_lossy(&login.stderr)
    );

    let session_text =
        fs::read_to_string(client_repo.join(".pray/session.json")).expect("session file");
    let session_json: Value = serde_json::from_str(&session_text).expect("session json");
    let server_urls = session_server_urls(&session_json);
    assert!(server_urls.contains(&server_url_a));
    assert!(server_urls.contains(&server_url_b));

    let _ = server_a.kill();
    let _ = server_a.wait();
    let _ = server_b.kill();
    let _ = server_b.wait();
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

fn run_pray_login_passkey(
    repo: &Path,
    server_url: &str,
    email: &str,
    credential_id: &str,
    private_key_path: &Path,
) {
    let login = Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "login",
            "--server",
            server_url,
            "--email",
            email,
            "--credential-id",
            credential_id,
            "--passkey-key",
            private_key_path.to_str().expect("private key path"),
        ])
        .current_dir(repo)
        .output()
        .expect("run passkey login");
    assert!(
        login.status.success(),
        "passkey login failed: {}",
        String::from_utf8_lossy(&login.stderr)
    );
}

fn run_pray_login_ssh_agent(
    repo: &Path,
    server_url: &str,
    email: &str,
    public_key_path: &Path,
    ssh_auth_sock: &Path,
) {
    let login = Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "login",
            "--server",
            server_url,
            "--email",
            email,
            "--public-key",
            public_key_path.to_str().expect("public key path"),
            "--ssh-agent",
        ])
        .current_dir(repo)
        .env("SSH_AUTH_SOCK", ssh_auth_sock)
        .output()
        .expect("run ssh-agent login");
    assert!(
        login.status.success(),
        "ssh-agent login failed: {}",
        String::from_utf8_lossy(&login.stderr)
    );
}

fn spawn_mock_ssh_agent(socket_path: &Path, signing_key: SigningKey) -> thread::JoinHandle<()> {
    if socket_path.exists() {
        fs::remove_file(socket_path).expect("remove stale ssh agent socket");
    }
    let listener = UnixListener::bind(socket_path).expect("bind mock ssh agent");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept ssh agent connection");
        let (message_type, payload) =
            read_ssh_message(&mut stream).expect("read ssh agent request");
        assert_eq!(message_type, 13);
        let mut cursor = payload.as_slice();
        let _public_key_blob = read_ssh_string(&mut cursor).expect("read public key blob");
        let message = read_ssh_string(&mut cursor).expect("read message");
        let _flags = read_u32(&mut cursor).expect("read flags");
        let signature = signing_key.sign(&message).to_bytes();
        let mut signature_blob = Vec::new();
        write_ssh_string(&mut signature_blob, b"ssh-ed25519");
        write_ssh_string(&mut signature_blob, &signature);
        let mut response_payload = Vec::new();
        write_ssh_string(&mut response_payload, &signature_blob);
        write_ssh_message(&mut stream, 14, &response_payload).expect("write ssh agent response");
    })
}

fn verify_email_registration(store: &RegistryAuthStore, email: &str) {
    let registration = store
        .register_email(email, EmailConfirmationPolicy::Required)
        .expect("register email");
    let verification_code = registration
        .verification_code
        .as_deref()
        .expect("verification code");
    store
        .verify_email(email, verification_code)
        .expect("verify email");
}

fn write_auth_registry_fixture(root: &Path) {
    fs::create_dir_all(root.join("v1")).expect("auth root directories");
    fs::write(
        root.join("v1/index.json"),
        r#"{
            "spec": "prayfile-distribution-1",
            "packages": []
        }"#,
    )
    .expect("write auth index");
    fs::write(
        root.join("v1/trust.json"),
        r#"{
            "email_confirmation": "required",
            "passkeys_enabled": true,
            "ssh_keys_enabled": true,
            "ssh_agent_signing_enabled": true
        }"#,
    )
    .expect("write auth trust settings");
}

fn session_server_urls(session_json: &Value) -> Vec<String> {
    if let Some(sessions) = session_json.get("sessions").and_then(Value::as_array) {
        return sessions
            .iter()
            .filter_map(|session| {
                session
                    .get("server_url")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
            .collect();
    }

    session_json
        .get("server_url")
        .and_then(Value::as_str)
        .map(|server_url| vec![server_url.to_string()])
        .unwrap_or_default()
}

fn spawn_server(root: &Path, port: u16) -> Child {
    Command::new(env!("CARGO_BIN_EXE_pray"))
        .args([
            "serve",
            "--root",
            root.to_str().expect("registry path"),
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn server")
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

fn write_private_key_file(repo: &Path, filename: &str, signing_key: &SigningKey) -> PathBuf {
    let path = repo.join(filename);
    fs::write(&path, signing_key.to_bytes()).expect("write private key file");
    path
}

fn write_public_key_file(repo: &Path, filename: &str, public_key: &str) -> PathBuf {
    let path = repo.join(filename);
    fs::write(&path, format!("{public_key}\n")).expect("write public key file");
    path
}

fn signing_key_from_seed(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn ssh_public_key_text(signing_key: &SigningKey) -> String {
    let mut blob = Vec::new();
    write_ssh_string(&mut blob, b"ssh-ed25519");
    write_ssh_string(&mut blob, &signing_key.verifying_key().to_bytes());
    format!("ssh-ed25519 {}", STANDARD.encode(blob))
}

fn write_ssh_string(buffer: &mut Vec<u8>, bytes: &[u8]) {
    buffer.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buffer.extend_from_slice(bytes);
}

fn read_ssh_string(cursor: &mut &[u8]) -> std::io::Result<Vec<u8>> {
    let length = read_u32(cursor)? as usize;
    if cursor.len() < length {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "truncated ssh string",
        ));
    }
    let (value, rest) = cursor.split_at(length);
    *cursor = rest;
    Ok(value.to_vec())
}

fn read_u32(cursor: &mut &[u8]) -> std::io::Result<u32> {
    if cursor.len() < 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "truncated ssh field",
        ));
    }
    let (length_bytes, rest) = cursor.split_at(4);
    *cursor = rest;
    Ok(u32::from_be_bytes(
        length_bytes.try_into().expect("length bytes"),
    ))
}

fn read_ssh_message(stream: &mut UnixStream) -> std::io::Result<(u8, Vec<u8>)> {
    let length = read_u32_from_stream(stream)? as usize;
    let mut buffer = vec![0u8; length];
    stream.read_exact(&mut buffer)?;
    let message_type = *buffer
        .first()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "empty response"))?;
    Ok((message_type, buffer[1..].to_vec()))
}

fn write_ssh_message(
    stream: &mut UnixStream,
    message_type: u8,
    payload: &[u8],
) -> std::io::Result<()> {
    let mut buffer = Vec::new();
    buffer.push(message_type);
    buffer.extend_from_slice(payload);
    write_u32_to_stream(stream, buffer.len() as u32)?;
    stream.write_all(&buffer)
}

fn read_u32_from_stream(stream: &mut UnixStream) -> std::io::Result<u32> {
    let mut buffer = [0u8; 4];
    stream.read_exact(&mut buffer)?;
    Ok(u32::from_be_bytes(buffer))
}

fn write_u32_to_stream(stream: &mut UnixStream, value: u32) -> std::io::Result<()> {
    stream.write_all(&value.to_be_bytes())
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

fn fetch_http_get(url: &str) -> HttpResponse {
    fetch_http_request("GET", url, None)
}

fn fetch_http_post(url: &str, body: &str) -> HttpResponse {
    fetch_http_request("POST", url, Some(body))
}

struct HttpResponse {
    status: u16,
    body: String,
}

fn fetch_http_request(method: &str, url: &str, body: Option<&str>) -> HttpResponse {
    let url = url.strip_prefix("http://").expect("http url");
    let (host_port, path) = url.split_once('/').unwrap_or((url, ""));
    let (host, port) = host_port.split_once(':').expect("host and port");
    let mut stream =
        TcpStream::connect((host, port.parse::<u16>().expect("port"))).expect("connect");
    let request_path = format!("/{}", path);
    let body = body.unwrap_or("");
    let content_length = body.len();
    write!(
        stream,
        "{} {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        method,
        request_path,
        host_port,
        content_length,
        body
    )
    .expect("write request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read response");
    let mut sections = response.splitn(2, "\r\n\r\n");
    let header = sections.next().unwrap_or_default();
    let body = sections.next().unwrap_or_default().to_string();
    let status = header
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .expect("status code");
    HttpResponse { status, body }
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
