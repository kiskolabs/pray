mod install_support;

use install_support::{create_grouped_fixture, run_pray, temporary_directory};
use std::env;
use std::fs;
use std::sync::{Mutex, OnceLock};

fn environment_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn global_path_and_environment_flags_apply_before_install() {
    let _guard = environment_test_lock()
        .lock()
        .expect("environment test lock");
    let workspace = temporary_directory("pray-cli-env");
    let project_root = workspace.join("project");
    fs::create_dir_all(&project_root).expect("project dir");
    create_grouped_fixture(&project_root);

    let output = run_pray(
        &workspace,
        &[
            "--path",
            project_root.to_str().expect("path"),
            "--env",
            "development",
            "install",
        ],
    );
    assert!(
        output.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lock_text = fs::read_to_string(project_root.join("Prayfile.lock")).expect("lockfile");
    assert!(lock_text.contains("environment = \"development\""));
}

#[test]
fn dotenv_values_apply_when_cli_is_omitted() {
    let _guard = environment_test_lock()
        .lock()
        .expect("environment test lock");
    env::remove_var("PRAY_ENV");
    let workspace = temporary_directory("pray-cli-dotenv");
    let project_root = workspace.join("project");
    fs::create_dir_all(&project_root).expect("project dir");
    create_grouped_fixture(&project_root);
    fs::write(project_root.join(".env"), "PRAY_ENV=development\n").expect("dotenv");

    let output = run_pray(&project_root, &["install"]);
    assert!(
        output.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lock_text = fs::read_to_string(project_root.join("Prayfile.lock")).expect("lockfile");
    assert!(lock_text.contains("environment = \"development\""));
}

#[test]
fn process_environment_does_not_override_cli_environment() {
    let _guard = environment_test_lock()
        .lock()
        .expect("environment test lock");
    struct EnvironmentVariableGuard(&'static str);
    impl Drop for EnvironmentVariableGuard {
        fn drop(&mut self) {
            env::remove_var(self.0);
        }
    }

    let workspace = temporary_directory("pray-cli-precedence");
    let project_root = workspace.join("project");
    fs::create_dir_all(&project_root).expect("project dir");
    create_grouped_fixture(&project_root);
    env::set_var("PRAY_ENV", "ignored");
    let _guard = EnvironmentVariableGuard("PRAY_ENV");

    let output = run_pray(
        &workspace,
        &[
            "--path",
            project_root.to_str().expect("path"),
            "--environment",
            "development",
            "install",
        ],
    );
    assert!(output.status.success());
    let lock_text = fs::read_to_string(project_root.join("Prayfile.lock")).expect("lockfile");
    assert!(lock_text.contains("environment = \"development\""));
}
