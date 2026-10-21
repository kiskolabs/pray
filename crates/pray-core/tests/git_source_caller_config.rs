#![allow(dead_code)]

#[path = "support/git_catalog.rs"]
mod git_catalog;

use git_catalog::{pin_cache, unique_temp, write_git_catalog, write_prayfile};
use pray_core::embed::{resolve_project_with_options, ResolveOptions};
use std::fs;

const CONFIG_VARIABLES: [&str; 3] = ["GIT_CONFIG_COUNT", "GIT_CONFIG_KEY_0", "GIT_CONFIG_VALUE_0"];

struct CallerConfig;

impl Drop for CallerConfig {
    fn drop(&mut self) {
        for name in CONFIG_VARIABLES {
            std::env::remove_var(name);
        }
    }
}

#[test]
fn install_applies_git_config_the_caller_passes_in_the_environment() {
    let root = unique_temp("pray-caller-git-config");
    let _cache_env = pin_cache(&root);
    let origin = write_git_catalog(&root.join("origin"), "sample/base");
    let alias = "file:///pray-caller-config-alias";
    write_prayfile(
        &root,
        &format!(
            "source \"used\", \"git+{alias}\"\npray \"sample/base\", \"~> 1.0\", source: \"used\"\n"
        ),
    );

    let result = {
        let _config = CallerConfig;
        std::env::set_var("GIT_CONFIG_COUNT", "1");
        std::env::set_var(
            "GIT_CONFIG_KEY_0",
            format!("url.file://{}.insteadOf", origin.display()),
        );
        std::env::set_var("GIT_CONFIG_VALUE_0", alias);
        resolve_project_with_options(&root.join("Prayfile"), &ResolveOptions::default())
    };

    let project = result.expect("caller git config should rewrite the source URL");
    assert_eq!(project.packages[0].spec.version, "1.0.0");
    let _ = fs::remove_dir_all(&root);
}
