use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn run_pray(repo: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_pray"))
        .args(arguments)
        .current_dir(repo)
        .env_remove("PRAY_PATH")
        .env_remove("PRAY_FILE_PATH")
        .env_remove("PRAY_ENV")
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

#[test]
fn missing_prayfile_exits_nonzero() {
    let repo = temporary_directory("pray-missing-manifest");
    let output = run_pray(&repo, &["install"]);
    assert!(!output.status.success());
    assert!(matches!(output.status.code(), Some(1) | Some(3)));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Prayfile") || stderr.contains("manifest") || stderr.contains("No such"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn unknown_command_exits_usage() {
    let repo = temporary_directory("pray-unknown-command");
    let output = run_pray(&repo, &["not-a-real-command"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("usage error:"));
}

#[test]
fn legacy_target_output_agent_emit_deprecation_warnings() {
    let repo = temporary_directory("pray-deprecation-warn");
    fs::create_dir_all(repo.join("packages/base/exports")).expect("dirs");
    fs::write(
        repo.join("packages/base/base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.0.0"
  spec.files = ["README.md", "exports/basics.md"]
  spec.exports = {
    "basics" => { type: "fragment", path: "exports/basics.md", summary: "basics" }
  }
end
"#,
    )
    .expect("prayspec");
    fs::write(repo.join("packages/base/README.md"), "readme\n").expect("readme");
    fs::write(repo.join("packages/base/exports/basics.md"), "basics\n").expect("export");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", path: "packages/base"
"#,
    )
    .expect("Prayfile");

    let output = run_pray(&repo, &["tree"]);
    assert!(
        output.status.success(),
        "tree failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("`target` is deprecated"));
    assert!(stderr.contains("`output` is deprecated"));
    assert!(stderr.contains("`agent` is deprecated"));
    assert!(stderr.contains("version 2"));
}

#[test]
fn dependency_cycle_exits_resolution() {
    let repo = temporary_directory("pray-cli-cycle");
    for name in ["alpha", "beta"] {
        let other = if name == "alpha" { "beta" } else { "alpha" };
        let package_root = repo.join(format!("packages/{name}"));
        fs::create_dir_all(package_root.join("exports")).expect("dirs");
        fs::write(
            package_root.join(format!("{name}.prayspec")),
            format!(
                r#"
Package::Specification.new do |spec|
  spec.name = "sample/{name}"
  spec.version = "1.0.0"
  spec.files = ["README.md", "exports/{name}.md"]
  spec.exports = {{
    "{name}" => {{ type: "fragment", path: "exports/{name}.md", summary: "{name}" }}
  }}
  spec.add_dependency "sample/{other}", "~> 1.0"
end
"#
            ),
        )
        .expect("prayspec");
        fs::write(package_root.join("README.md"), "readme\n").expect("readme");
        fs::write(package_root.join(format!("exports/{name}.md")), "body\n").expect("export");
    }
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
pray "sample/alpha", path: "packages/alpha"
pray "sample/beta", path: "packages/beta"
"#,
    )
    .expect("Prayfile");

    let output = run_pray(&repo, &["tree"]);
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("dependency cycle detected"),
        "unexpected stderr: {stderr}"
    );
}
