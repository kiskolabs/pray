#[path = "install_network_support.rs"]
mod support;

use support::{create_add_fixture, run_pray, temporary_directory};

#[test]
fn publish_prints_published_and_unchanged_lines() {
    let repo = temporary_directory("pray-publish-feedback");
    let registry_root = temporary_directory("pray-publish-feedback-root");
    create_add_fixture(&repo);
    assert_success(run_pray(
        &repo,
        &["add", "sample/base", "--path", "packages/base"],
    ));

    let first = publish(&repo, &registry_root, &[]);
    let first_out = String::from_utf8_lossy(&first.stdout);
    assert!(
        first_out.contains("published sample/base"),
        "first publish should report published:\n{first_out}"
    );
    assert!(
        first_out.contains("1 published, 0 unchanged"),
        "first publish should summarize:\n{first_out}"
    );
    assert!(
        first.stderr.is_empty(),
        "stderr should stay empty on success"
    );

    let second = publish(&repo, &registry_root, &[]);
    let second_out = String::from_utf8_lossy(&second.stdout);
    assert!(
        second_out.contains("unchanged sample/base"),
        "republish should report unchanged:\n{second_out}"
    );
    assert!(
        second_out.contains("0 published, 1 unchanged"),
        "republish should summarize unchanged-only:\n{second_out}"
    );
}

#[test]
fn publish_quiet_suppresses_stdout_including_dry_run() {
    let repo = temporary_directory("pray-publish-quiet");
    let registry_root = temporary_directory("pray-publish-quiet-root");
    create_add_fixture(&repo);
    assert_success(run_pray(
        &repo,
        &["add", "sample/base", "--path", "packages/base"],
    ));

    let root = registry_root.to_str().expect("path");
    let dry = assert_success(run_pray(
        &repo,
        &["--quiet", "publish", "--dry-run", "--root", root],
    ));
    assert!(
        dry.stdout.is_empty(),
        "--quiet --dry-run should be silent:\n{}",
        String::from_utf8_lossy(&dry.stdout)
    );

    let live = assert_success(run_pray(&repo, &["--quiet", "publish", "--root", root]));
    assert!(
        live.stdout.is_empty(),
        "--quiet publish should be silent:\n{}",
        String::from_utf8_lossy(&live.stdout)
    );
    assert!(
        registry_root.join("v1/packages/sample/base.json").is_file(),
        "quiet publish must still write"
    );
}

#[test]
fn publish_verbose_includes_detail_lines() {
    let repo = temporary_directory("pray-publish-verbose");
    let registry_root = temporary_directory("pray-publish-verbose-root");
    create_add_fixture(&repo);
    assert_success(run_pray(
        &repo,
        &["add", "sample/base", "--path", "packages/base"],
    ));
    let root = registry_root.to_str().expect("path");

    let first = assert_success(run_pray(&repo, &["--verbose", "publish", "--root", root]));
    let out = String::from_utf8_lossy(&first.stdout);
    assert!(
        out.contains("v1/artifacts/"),
        "verbose publish should name artifact path:\n{out}"
    );

    let second = assert_success(run_pray(&repo, &["-v", "publish", "--root", root]));
    let second_out = String::from_utf8_lossy(&second.stdout);
    assert!(
        second_out.contains("stored publish matches"),
        "verbose unchanged should name skip reason:\n{second_out}"
    );
}

#[test]
fn quiet_and_verbose_are_documented_as_status_stdout() {
    let help = assert_success(run_pray_cwd(&["--help"]));
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(
        stdout.contains("suppress non-error status stdout"),
        "quiet help should name status stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("extra status detail on stdout"),
        "verbose help should name status detail:\n{stdout}"
    );
}

#[test]
fn publish_redacts_url_userinfo_in_dry_run() {
    let repo = temporary_directory("pray-publish-redact");
    create_add_fixture(&repo);
    assert_success(run_pray(
        &repo,
        &["add", "sample/base", "--path", "packages/base"],
    ));
    let output = assert_success(run_pray(
        &repo,
        &[
            "publish",
            "--dry-run",
            "--server",
            "https://user:secret@example.invalid/reg",
        ],
    ));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("user:secret"),
        "credentials must not appear:\n{stdout}"
    );
    assert!(
        stdout.contains("https://***@example.invalid/reg"),
        "redacted target missing:\n{stdout}"
    );
}

fn publish(
    repo: &std::path::Path,
    registry_root: &std::path::Path,
    global: &[&str],
) -> std::process::Output {
    let root = registry_root.to_str().expect("registry path");
    let mut arguments = global.to_vec();
    arguments.extend_from_slice(&["publish", "--root", root]);
    assert_success(run_pray(repo, &arguments))
}

fn assert_success(output: std::process::Output) -> std::process::Output {
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn run_pray_cwd(arguments: &[&str]) -> std::process::Output {
    let binary = std::path::PathBuf::from(env!("CARGO_BIN_EXE_pray"));
    let binary = binary.canonicalize().unwrap_or(binary);
    std::process::Command::new(binary)
        .args(arguments)
        .output()
        .expect("run pray")
}
