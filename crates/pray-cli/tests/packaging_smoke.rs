#[path = "install_network_support.rs"]
mod support;

use serde_json::Value;
use std::fs;
use support::{create_add_fixture, run_pray, temporary_directory};

#[test]
fn packaging_smoke_publish_install_verify_without_html() {
    let workspace = temporary_directory("pray-packaging-smoke");
    let source_repo = workspace.join("source");
    let registry_root = workspace.join("registry");
    let consumer_repo = workspace.join("consumer");
    let pray_home = workspace.join("pray-home");
    fs::create_dir_all(&source_repo).expect("source");
    fs::create_dir_all(&registry_root).expect("registry");
    fs::create_dir_all(&consumer_repo).expect("consumer");
    fs::create_dir_all(&pray_home).expect("pray home");

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

    let publish = run_pray(
        &source_repo,
        &[
            "publish",
            "--root",
            registry_root.to_str().expect("registry path"),
        ],
    );
    assert!(
        publish.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&publish.stderr)
    );
    assert!(registry_root.join("v1/index.json").is_file());
    assert!(registry_root.join("v1/packages/sample/base.json").is_file());
    assert!(registry_root
        .join("v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg")
        .is_file());

    let relative_registry = "../registry";
    fs::write(
        consumer_repo.join("Prayfile"),
        r#"
prayfile "1"
source "default", "https://example.invalid"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", "~> 1.4", source: "default"
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("consumer Prayfile");
    fs::write(
        pray_home.join("config.toml"),
        format!(
            r#"
[local.source]
default = "{relative_registry}"
"#
        ),
    )
    .expect("config");

    let install = std::process::Command::new(env!("CARGO_BIN_EXE_pray"))
        .args(["install"])
        .current_dir(&consumer_repo)
        .env("PRAY_HOME", &pray_home)
        .env("PRAY_CONFIG", pray_home.join("config.toml"))
        .output()
        .expect("install");
    assert!(
        install.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );
    assert!(consumer_repo.join("Prayfile.lock").is_file());
    assert!(consumer_repo.join("INSTRUCTIONS.md").is_file());

    let verify = std::process::Command::new(env!("CARGO_BIN_EXE_pray"))
        .args(["verify"])
        .current_dir(&consumer_repo)
        .env("PRAY_HOME", &pray_home)
        .env("PRAY_CONFIG", pray_home.join("config.toml"))
        .output()
        .expect("verify");
    assert!(
        verify.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&verify.stderr)
    );

    let yank = run_pray(
        &workspace,
        &[
            "yank",
            "sample/base",
            "1.4.3",
            "--root",
            registry_root.to_str().expect("registry path"),
        ],
    );
    assert!(
        yank.status.success(),
        "yank failed: {}",
        String::from_utf8_lossy(&yank.stderr)
    );
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(registry_root.join("v1/packages/sample/base.json")).expect("metadata"),
    )
    .expect("metadata json");
    assert_eq!(metadata["versions"][0]["yanked"], true);
    assert!(registry_root
        .join("v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg")
        .is_file());

    let strict = std::process::Command::new(env!("CARGO_BIN_EXE_pray"))
        .args(["install", "--strict"])
        .current_dir(&consumer_repo)
        .env("PRAY_HOME", &pray_home)
        .env("PRAY_CONFIG", pray_home.join("config.toml"))
        .output()
        .expect("strict install");
    assert!(
        !strict.status.success(),
        "strict install should fail on yanked lock"
    );
    let stderr = String::from_utf8_lossy(&strict.stderr);
    assert!(stderr.contains("yanked"), "unexpected stderr: {stderr}");
}
