/// Prayfile keywords retained for compatibility and scheduled for removal in version 2.
pub const DEPRECATED_TARGET: &str = "target";
pub const DEPRECATED_OUTPUT: &str = "output";
pub const DEPRECATED_AGENT: &str = "agent";

pub fn is_deprecated_prayfile_keyword(keyword: &str) -> bool {
    matches!(
        keyword,
        DEPRECATED_TARGET | DEPRECATED_OUTPUT | DEPRECATED_AGENT
    )
}

pub fn deprecation_warning(keyword: &str) -> Option<String> {
    let replacement = match keyword {
        DEPRECATED_TARGET => "compose` / `tree",
        DEPRECATED_OUTPUT => "compose",
        DEPRECATED_AGENT => "pray",
        _ => return None,
    };
    Some(format!(
        "warning: `{keyword}` is deprecated and will be removed in version 2; prefer `{replacement}`"
    ))
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
        ]);
        assert_eq!(warnings.len(), 3);
        assert!(warnings[0].contains("`target`"));
        assert!(warnings[0].contains("version 2"));
        assert!(warnings[1].contains("`output`"));
        assert!(warnings[2].contains("`agent`"));
        assert!(warnings[2].contains("`pray`"));
    }
}
