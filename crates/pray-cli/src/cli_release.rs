use pray_core::cli_release::{
    build_upgrade_notice, format_upgrade_notice, parse_github_latest_release_tag_name,
    parse_workspace_package_version, should_check_upgrade, upgrade_available,
    version_check_cache_path, version_check_ttl_seconds, DEFAULT_REPOSITORY,
    DEFAULT_UPGRADE_COMMAND,
};
use pray_core::client_trust::env_truthy;
use pray_core::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct VersionCheckCache {
    checked_at: u64,
    latest_version: Option<String>,
}

pub fn maybe_print_upgrade_notice(arguments: &[String]) {
    if !version_check_enabled() {
        return;
    }
    let command = arguments.first().map(String::as_str).unwrap_or("");
    let offline = command == "install" && arguments.iter().any(|argument| argument == "--offline");
    if !should_check_upgrade(command, offline) {
        return;
    }
    let current_version = env!("CARGO_PKG_VERSION");
    let Ok(Some(latest_version)) = resolve_latest_version() else {
        return;
    };
    let Ok(true) = upgrade_available(current_version, &latest_version) else {
        return;
    };
    let notice = build_upgrade_notice(
        current_version,
        &latest_version,
        DEFAULT_REPOSITORY,
        DEFAULT_UPGRADE_COMMAND,
    );
    let _ = writeln!(std::io::stderr(), "{}", format_upgrade_notice(&notice));
}

pub fn upgrade_command() -> PrayResult<()> {
    let status = Command::new("cargo")
        .args([
            "install",
            "--git",
            DEFAULT_REPOSITORY,
            "--locked",
            "--force",
            "pray",
        ])
        .status()
        .map_err(|error| {
            PrayError::Unsupported(format!(
                "failed to run cargo install: {error} (is cargo on PATH?)"
            ))
        })?;
    if status.success() {
        return Ok(());
    }
    Err(PrayError::Unsupported(
        "pray upgrade failed: cargo install returned a non-zero exit status".into(),
    ))
}

fn version_check_enabled() -> bool {
    if env_truthy("PRAY_NO_VERSION_CHECK") || env_truthy("PRAY_OFFLINE") {
        return false;
    }
    if env_truthy("CI") {
        return false;
    }
    !matches!(env::var("CI").as_deref(), Ok("true") | Ok("1"))
}

fn resolve_latest_version() -> PrayResult<Option<String>> {
    if let Ok(version) = env::var("PRAY_TEST_LATEST_VERSION") {
        let version = version.trim().to_string();
        return Ok(if version.is_empty() {
            None
        } else {
            Some(version)
        });
    }
    if let Some(cached) = read_cached_latest_version()? {
        return Ok(Some(cached));
    }
    let latest_version = fetch_latest_version()?;
    write_cached_latest_version(latest_version.as_deref())?;
    Ok(latest_version)
}

fn read_cached_latest_version() -> PrayResult<Option<String>> {
    let Some(cache_path) = version_check_cache_file() else {
        return Ok(None);
    };
    if !cache_path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(&cache_path)?;
    let cache: VersionCheckCache = serde_json::from_str(&text).map_err(|error| {
        PrayError::Parse {
            kind: "cli-version-check",
            message: format!("{}: {error}", cache_path.display()),
        }
    })?;
    let now = unix_timestamp()?;
    if now.saturating_sub(cache.checked_at) > version_check_ttl_seconds() {
        return Ok(None);
    }
    Ok(cache.latest_version)
}

fn write_cached_latest_version(latest_version: Option<&str>) -> PrayResult<()> {
    let Some(cache_path) = version_check_cache_file() else {
        return Ok(());
    };
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let cache = VersionCheckCache {
        checked_at: unix_timestamp()?,
        latest_version: latest_version.map(str::to_string),
    };
    let text = serde_json::to_string(&cache).map_err(|error| PrayError::Parse {
        kind: "cli-version-check",
        message: error.to_string(),
    })?;
    fs::write(cache_path, text)?;
    Ok(())
}

fn fetch_latest_version() -> PrayResult<Option<String>> {
    if let Some(version) = fetch_latest_release_version()? {
        return Ok(Some(version));
    }
    fetch_workspace_package_version()
}

fn fetch_latest_release_version() -> PrayResult<Option<String>> {
    let url = "https://api.github.com/repos/kiskolabs/pray/releases/latest";
    let body = http_get(url)?;
    Ok(parse_github_latest_release_tag_name(&body))
}

fn fetch_workspace_package_version() -> PrayResult<Option<String>> {
    let url = "https://raw.githubusercontent.com/kiskolabs/pray/main/Cargo.toml";
    let body = http_get(url)?;
    Ok(parse_workspace_package_version(&body))
}

fn http_get(url: &str) -> PrayResult<String> {
    let mut response = ureq::get(url)
        .header(
            "User-Agent",
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
        )
        .call()
        .map_err(|error| PrayError::Unsupported(format!("HTTP request failed for {url}: {error}")))?;
    let status = response.status();
    let body = response.body_mut().read_to_string().unwrap_or_default();
    if !status.is_success() {
        return Err(PrayError::Unsupported(format!(
            "HTTP request for {url} returned {}",
            status.as_u16()
        )));
    }
    Ok(body)
}

fn version_check_cache_file() -> Option<PathBuf> {
    version_check_cache_root().map(|root| version_check_cache_path(&root))
}

fn version_check_cache_root() -> Option<PathBuf> {
    if let Ok(path) = env::var("PRAY_CACHE") {
        return Some(PathBuf::from(path));
    }
    if let Ok(home) = env::var("PRAY_HOME") {
        return Some(PathBuf::from(home).join("cache"));
    }
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache").join("pray"))
}

fn unix_timestamp() -> PrayResult<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| PrayError::Unsupported(format!("system clock error: {error}")))
}
