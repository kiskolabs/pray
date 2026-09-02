use crate::package_spec::PackageSpec;

/// Prayfile keywords retained for compatibility and scheduled for removal in version 2.
pub const DEPRECATED_TARGET: &str = "target";
pub const DEPRECATED_OUTPUT: &str = "output";
pub const DEPRECATED_AGENT: &str = "agent";
pub const DEPRECATED_SKILLS: &str = "skills";
pub const DEPRECATED_SKILL: &str = "skill";
pub const DEPRECATED_SPEC_SKILLS: &str = "spec.skills";

pub fn is_deprecated_prayfile_keyword(keyword: &str) -> bool {
    matches!(
        keyword,
        DEPRECATED_TARGET | DEPRECATED_OUTPUT | DEPRECATED_AGENT | DEPRECATED_SKILLS
    )
}

pub fn deprecation_warning(keyword: &str) -> Option<String> {
    let replacement = match keyword {
        DEPRECATED_TARGET => "compose` / `tree",
        DEPRECATED_OUTPUT => "compose",
        DEPRECATED_AGENT => "pray",
        DEPRECATED_SKILLS => "tree` / `folder",
        DEPRECATED_SKILL => "folder",
        DEPRECATED_SPEC_SKILLS => "a folder export",
        _ => return None,
    };
    Some(format!(
        "warning: `{keyword}` is deprecated and will be removed in version 2; prefer `{replacement}`"
    ))
}

pub fn package_spec_deprecation_warnings(spec: &PackageSpec) -> Vec<String> {
    let mut keywords = Vec::new();
    if !spec.skills.is_empty() {
        keywords.push(DEPRECATED_SPEC_SKILLS.to_string());
    }
    if spec
        .exports
        .values()
        .any(|export| export.kind == DEPRECATED_SKILL)
    {
        keywords.push(DEPRECATED_SKILL.to_string());
    }
    deprecation_warnings_for(&keywords)
}

pub fn deprecation_warnings_for(keywords: &[String]) -> Vec<String> {
    let mut warnings = Vec::new();
    for keyword in keywords {
        if let Some(warning) = deprecation_warning(keyword) {
            warnings.push(warning);
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warns_for_legacy_keywords() {
        let warnings = deprecation_warnings_for(&[
            DEPRECATED_TARGET.to_string(),
            DEPRECATED_OUTPUT.to_string(),
            DEPRECATED_AGENT.to_string(),
            DEPRECATED_SKILLS.to_string(),
        ]);
        assert_eq!(warnings.len(), 4);
        assert!(warnings[0].contains("`target`"));
        assert!(warnings[0].contains("version 2"));
        assert!(warnings[1].contains("`output`"));
        assert!(warnings[2].contains("`agent`"));
        assert!(warnings[2].contains("`pray`"));
        assert!(warnings[3].contains("`skills`"));
        assert!(warnings[3].contains("`tree`"));
    }
}
