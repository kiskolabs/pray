use super::PackageSpec;
use crate::constraint::{normalize_version_constraint, version_satisfies};
use crate::{PrayError, PrayResult};

impl PackageSpec {
    pub const LOCAL_VERSION: &'static str = "local";

    pub fn has_release_version(&self) -> bool {
        !self.version.is_empty() && self.version != Self::LOCAL_VERSION
    }

    pub fn recorded_version(&self) -> &str {
        if self.has_release_version() {
            &self.version
        } else {
            Self::LOCAL_VERSION
        }
    }

    pub fn require_release_version(&self) -> PrayResult<()> {
        if self.has_release_version() {
            Ok(())
        } else {
            Err(PrayError::Manifest(format!(
                "package {} needs a version before it can be packaged",
                self.name
            )))
        }
    }

    pub fn satisfy_constraint(&self, constraint: &str) -> PrayResult<()> {
        if !self.has_release_version() {
            let normalized = normalize_version_constraint(constraint);
            if normalized.is_empty() || normalized == "*" {
                return Ok(());
            }
            return Err(PrayError::Resolution(format!(
                "package {} has no version; add spec.version or omit the constraint",
                self.name
            )));
        }
        if version_satisfies(&self.version, constraint)? {
            Ok(())
        } else {
            Err(PrayError::Resolution(format!(
                "package {} version {} does not satisfy constraint {}",
                self.name, self.version, constraint
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec_with_version(version: &str) -> PackageSpec {
        PackageSpec {
            name: "project".to_string(),
            version: version.to_string(),
            ..PackageSpec::default()
        }
    }

    #[test]
    fn omitted_version_records_local_and_matches_star() {
        let spec = spec_with_version("");
        assert_eq!(spec.recorded_version(), "local");
        spec.satisfy_constraint("*").expect("star");
        let error = spec.satisfy_constraint("~> 1.0").expect_err("range");
        assert!(error.to_string().contains("has no version"));
        spec.require_release_version().expect_err("pack");
    }

    #[test]
    fn release_version_uses_semver_constraints() {
        let spec = spec_with_version("1.4.3");
        spec.satisfy_constraint("~> 1.4").expect("matches");
        spec.require_release_version().expect("pack");
    }
}
