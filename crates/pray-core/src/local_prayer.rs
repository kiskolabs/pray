use crate::{PrayError, PrayResult};

pub const DEFAULT_LOCAL_PRAYER_NAME: &str = "project";
pub const DEFAULT_PATH_SOURCE_NAME: &str = "local";
pub const DEFAULT_PATH_SOURCE_DIRECTORY: &str = "prayers";

pub fn validate_local_prayer_name(name: &str) -> PrayResult<&str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(PrayError::Usage("prayer name is missing".to_string()));
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(PrayError::Usage(
            "prayer name must be a single folder name".to_string(),
        ));
    }
    if trimmed == "." || trimmed == ".." {
        return Err(PrayError::Usage(
            "prayer name must be a single folder name".to_string(),
        ));
    }
    if is_reserved_distribution_layout_name(trimmed) {
        return Err(PrayError::Usage(format!(
            "{trimmed} is reserved for the distribution layout"
        )));
    }
    Ok(trimmed)
}

pub fn path_source_package_directory(source_name: &str, package_name: &str) -> String {
    let prefix = format!("{source_name}/");
    if let Some(rest) = package_name.strip_prefix(&prefix) {
        if !rest.is_empty() && !rest.contains('/') && !rest.contains('\\') {
            return rest.to_string();
        }
    }
    package_name.replace(['/', '\\'], "-")
}

fn is_reserved_distribution_layout_name(name: &str) -> bool {
    let mut characters = name.chars();
    if characters.next() != Some('v') {
        return false;
    }
    let rest = characters.as_str();
    !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_single_folder_name() {
        assert_eq!(
            validate_local_prayer_name("project").expect("name"),
            "project"
        );
        assert_eq!(validate_local_prayer_name("notes").expect("name"), "notes");
    }

    #[test]
    fn refuses_distribution_layout_names() {
        let error = validate_local_prayer_name("v1").expect_err("v1");
        assert!(error.to_string().contains("reserved"));
        assert!(validate_local_prayer_name("v12").is_err());
        assert!(validate_local_prayer_name("vault").is_ok());
    }

    #[test]
    fn refuses_path_segments() {
        assert!(validate_local_prayer_name("a/b").is_err());
        assert!(validate_local_prayer_name("..").is_err());
        assert!(validate_local_prayer_name("").is_err());
    }

    #[test]
    fn path_source_directory_strips_a_matching_source_handle() {
        assert_eq!(
            path_source_package_directory("local", "local/project"),
            "project"
        );
        assert_eq!(path_source_package_directory("local", "project"), "project");
        assert_eq!(
            path_source_package_directory("local", "amkisko/rules"),
            "amkisko-rules"
        );
        assert_eq!(
            path_source_package_directory("amkisko", "amkisko/rules"),
            "rules"
        );
    }
}
