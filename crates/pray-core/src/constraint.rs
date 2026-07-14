use crate::{PrayError, PrayResult};
use semver::{Version, VersionReq};

/// Normalizes a Prayfile version constraint per SPEC §16.
///
/// Bare semver strings such as `1.0.0` are exact pins (`=1.0.0`), not caret ranges.
pub fn normalize_version_constraint(constraint: &str) -> String {
    let trimmed = constraint.trim();
    if trimmed.is_empty() || trimmed == "*" {
        return trimmed.to_string();
    }
    if trimmed.starts_with("~>")
        || trimmed.starts_with('~')
        || trimmed.starts_with('^')
        || trimmed.starts_with('=')
        || trimmed.starts_with('>')
        || trimmed.starts_with('<')
        || trimmed.contains('*')
    {
        return trimmed.to_string();
    }
    if Version::parse(trimmed).is_ok() {
        return format!("={trimmed}");
    }
    trimmed.to_string()
}

pub fn version_satisfies(version: &str, constraint: &str) -> PrayResult<bool> {
    let normalized = normalize_version_constraint(constraint);
    if normalized.is_empty() || normalized == "*" {
        return Ok(true);
    }
    let version =
        Version::parse(version).map_err(|error| PrayError::Resolution(error.to_string()))?;
    let req = if normalized.trim_start().starts_with("~>") {
        VersionReq::parse(&ruby_pessimistic_to_semver(&normalized)?)
            .map_err(|error| PrayError::Resolution(error.to_string()))?
    } else {
        VersionReq::parse(normalized.trim())
            .map_err(|error| PrayError::Resolution(error.to_string()))?
    };
    Ok(req.matches(&version))
}

/// Builds a Ruby pessimistic constraint (`~>`) that allows the given release line.
pub fn pessimistic_constraint_for_version(version: &str) -> PrayResult<String> {
    let parsed =
        Version::parse(version).map_err(|error| PrayError::Resolution(error.to_string()))?;
    if parsed.minor == 0 && parsed.patch == 0 {
        Ok(format!("~> {}.0", parsed.major))
    } else {
        Ok(format!("~> {}.{}", parsed.major, parsed.minor))
    }
}

/// Derives a Prayfile constraint that admits `latest_version`, preserving operator style.
pub fn latest_constraint_for_package(
    current_constraint: &str,
    latest_version: &str,
) -> PrayResult<String> {
    let normalized = normalize_version_constraint(current_constraint);
    if normalized == "*" {
        return Ok("*".to_string());
    }
    if normalized.starts_with("~") {
        return pessimistic_constraint_for_version(latest_version);
    }
    if normalized.starts_with('^') {
        let parsed = Version::parse(latest_version)
            .map_err(|error| PrayError::Resolution(error.to_string()))?;
        return Ok(format!("^{}.{}", parsed.major, parsed.minor));
    }
    if normalized.starts_with('=') || Version::parse(current_constraint.trim()).is_ok() {
        return Ok(format!("={latest_version}"));
    }
    pessimistic_constraint_for_version(latest_version)
}

fn ruby_pessimistic_to_semver(constraint: &str) -> PrayResult<String> {
    let text = constraint.trim().trim_start_matches("~>").trim();
    let parts: Vec<&str> = text.split('.').collect();
    if parts.is_empty() || parts.len() > 3 {
        return Err(PrayError::Resolution(format!(
            "unsupported Ruby pessimistic constraint: {constraint}"
        )));
    }
    let mut numbers = [0u64; 3];
    for (index, part) in parts.iter().enumerate() {
        numbers[index] = part
            .parse::<u64>()
            .map_err(|error| PrayError::Resolution(error.to_string()))?;
    }
    let lower = format!("{}.{}.{}", numbers[0], numbers[1], numbers[2]);
    let upper = match parts.len() {
        1 => format!("{}.0.0", numbers[0] + 1),
        2 => format!("{}.{}.0", numbers[0], numbers[1] + 1),
        _ => format!("{}.{}.0", numbers[0], numbers[1] + 1),
    };
    Ok(format!(">={}, <{}", lower, upper))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_semver_is_exact_pin() {
        assert_eq!(normalize_version_constraint("1.0.0"), "=1.0.0");
        assert_eq!(normalize_version_constraint("  2.3.4  "), "=2.3.4");
    }

    #[test]
    fn explicit_operators_are_preserved() {
        assert_eq!(normalize_version_constraint("~> 1.0"), "~> 1.0");
        assert_eq!(normalize_version_constraint("^2.0"), "^2.0");
        assert_eq!(normalize_version_constraint("= 1.2.3"), "= 1.2.3");
        assert_eq!(normalize_version_constraint("*"), "*");
    }

    #[test]
    fn bare_semver_matches_only_exact_version() {
        assert!(version_satisfies("1.0.0", "1.0.0").expect("matches"));
        assert!(!version_satisfies("1.0.1", "1.0.0").expect("does not match"));
        assert!(version_satisfies("1.0.1", "~> 1.0").expect("pessimistic matches"));
    }

    #[test]
    fn pessimistic_constraint_uses_major_minor_line() {
        assert_eq!(
            pessimistic_constraint_for_version("2.0.1").expect("constraint"),
            "~> 2.0"
        );
        assert_eq!(
            pessimistic_constraint_for_version("1.4.3").expect("constraint"),
            "~> 1.4"
        );
        assert!(version_satisfies("2.0.0", "~> 2.0").expect("matches"));
    }

    #[test]
    fn latest_constraint_preserves_operator_family() {
        assert_eq!(
            latest_constraint_for_package("~> 1.0", "2.0.0").expect("constraint"),
            "~> 2.0"
        );
        assert_eq!(
            latest_constraint_for_package("1.0.0", "2.0.0").expect("constraint"),
            "=2.0.0"
        );
        assert_eq!(
            latest_constraint_for_package("^1.0", "2.1.0").expect("constraint"),
            "^2.1"
        );
    }
}
