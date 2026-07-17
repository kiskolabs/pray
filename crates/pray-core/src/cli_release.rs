use crate::registry::version_is_greater_than;
use crate::PrayResult;
use std::path::{Path, PathBuf};

pub const DEFAULT_REPOSITORY: &str = "https://github.com/kiskolabs/pray";
pub const DEFAULT_UPGRADE_COMMAND: &str = "pray upgrade";

const VERSION_CHECK_CACHE_FILE: &str = "cli-version-check.json";
const VERSION_CHECK_TTL_SECONDS: u64 = 86_400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeNotice {
    pub current_version: String,
    pub latest_version: String,
    pub upgrade_command: String,
    pub changelog_url: String,
}

pub fn normalize_release_version(tag: &str) -> String {
    tag.trim().trim_start_matches(['v', 'V']).trim().to_string()
}

pub fn changelog_url(repository: &str, _latest_version: &str) -> String {
    let repository = repository.trim_end_matches('/');
    format!("{repository}/blob/main/CHANGELOG.md")
}

pub fn display_version(version: &str) -> String {
    let normalized = normalize_release_version(version);
    if normalized.is_empty() {
        return "main".to_string();
    }
    format!("v{normalized}")
}

pub fn parse_github_latest_release_tag_name(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    value
        .get("tag_name")
        .and_then(|tag| tag.as_str())
        .map(normalize_release_version)
        .filter(|version| !version.is_empty())
}

pub fn parse_workspace_package_version(cargo_toml: &str) -> Option<String> {
    let mut in_workspace_package = false;
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_workspace_package = trimmed == "[workspace.package]";
            continue;
        }
        if !in_workspace_package {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if key.trim() != "version" {
            continue;
        }
        let version = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim()
            .to_string();
        if version.is_empty() {
            return None;
        }
        return Some(version);
    }
    None
}

pub fn upgrade_available(current_version: &str, latest_version: &str) -> PrayResult<bool> {
    version_is_greater_than(latest_version, current_version)
}

pub fn build_upgrade_notice(
    current_version: &str,
    latest_version: &str,
    repository: &str,
    upgrade_command: &str,
) -> UpgradeNotice {
    UpgradeNotice {
        current_version: current_version.to_string(),
        latest_version: latest_version.to_string(),
        upgrade_command: upgrade_command.to_string(),
        changelog_url: changelog_url(repository, latest_version),
    }
}

pub fn format_upgrade_notice(notice: &UpgradeNotice) -> String {
    format!(
        "A new version of pray is available\n  {} → {}\n  Run: {}\n  Changelog: {}",
        display_version(&notice.current_version),
        display_version(&notice.latest_version),
        notice.upgrade_command,
        notice.changelog_url
    )
}

pub fn should_check_upgrade(command: &str, offline: bool) -> bool {
    if offline {
        return false;
    }
    !matches!(
        command,
        "upgrade" | "version" | "help" | "-h" | "--help" | "-V" | "--version"
    )
}

pub fn version_check_cache_path(cache_root: &Path) -> PathBuf {
    cache_root.join(VERSION_CHECK_CACHE_FILE)
}

pub fn version_check_ttl_seconds() -> u64 {
    VERSION_CHECK_TTL_SECONDS
}

#[cfg(test)]
mod tests {
    use super::{
        build_upgrade_notice, changelog_url, format_upgrade_notice, normalize_release_version,
        parse_github_latest_release_tag_name, parse_workspace_package_version,
        should_check_upgrade, upgrade_available, DEFAULT_REPOSITORY, DEFAULT_UPGRADE_COMMAND,
    };

    #[test]
    fn normalizes_release_tags() {
        assert_eq!(normalize_release_version("v1.2.3"), "1.2.3");
        assert_eq!(normalize_release_version("1.2.3"), "1.2.3");
    }

    #[test]
    fn parses_github_latest_release_tag() {
        let body = r#"{"tag_name":"v1.2.0","name":"1.2.0"}"#;
        assert_eq!(
            parse_github_latest_release_tag_name(body),
            Some("1.2.0".to_string())
        );
    }

    #[test]
    fn parses_workspace_package_version_from_cargo_toml() {
        let cargo_toml = r#"
[workspace.package]
version = "1.2.0"
edition = "2021"
"#;
        assert_eq!(
            parse_workspace_package_version(cargo_toml),
            Some("1.2.0".to_string())
        );
    }

    #[test]
    fn upgrade_notice_points_to_changelog() {
        let notice = build_upgrade_notice(
            "1.1.0",
            "1.2.0",
            DEFAULT_REPOSITORY,
            DEFAULT_UPGRADE_COMMAND,
        );
        assert_eq!(
            notice.changelog_url,
            "https://github.com/kiskolabs/pray/blob/main/CHANGELOG.md"
        );
        let formatted = format_upgrade_notice(&notice);
        assert!(formatted.contains("A new version of pray is available"));
        assert!(formatted.contains("v1.1.0 → v1.2.0"));
        assert!(formatted.contains("Run: pray upgrade"));
        assert!(formatted
            .contains("Changelog: https://github.com/kiskolabs/pray/blob/main/CHANGELOG.md"));
    }

    #[test]
    fn detects_when_upgrade_is_available() {
        assert!(upgrade_available("1.1.0", "1.2.0").expect("compare versions"));
        assert!(!upgrade_available("1.2.0", "1.2.0").expect("compare versions"));
    }

    #[test]
    fn skips_version_check_for_upgrade_and_version_commands() {
        assert!(!should_check_upgrade("upgrade", false));
        assert!(!should_check_upgrade("version", false));
        assert!(should_check_upgrade("install", false));
        assert!(!should_check_upgrade("install", true));
    }

    #[test]
    fn changelog_url_points_to_main_branch() {
        assert_eq!(
            changelog_url(DEFAULT_REPOSITORY, "1.0.0"),
            "https://github.com/kiskolabs/pray/blob/main/CHANGELOG.md"
        );
    }
}
