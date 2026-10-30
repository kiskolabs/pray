use crate::{PrayError, PrayResult};

pub(crate) fn canonicalize(references: &[String]) -> Vec<String> {
    let mut canonical: Vec<_> = references
        .iter()
        .map(|reference| reference.trim().to_string())
        .collect();
    canonical.sort();
    canonical.dedup();
    canonical
}

pub(crate) fn validate(references: &[String]) -> PrayResult<()> {
    for reference in references {
        if !valid(reference) {
            return Err(PrayError::Parse {
                kind: "prayspec",
                message: "invalid related prayer reference".to_string(),
            });
        }
    }
    Ok(())
}

fn valid(reference: &str) -> bool {
    if reference.is_empty() || reference.contains('\\') || reference.chars().any(char::is_control) {
        return false;
    }
    if absolute_uri(reference) {
        return !uri_has_userinfo(reference);
    }
    if let Some(rest) = reference.strip_prefix('/') {
        let path = locator_path(rest);
        return !path.is_empty() && !path.starts_with('/');
    }
    if reference.starts_with("./") || reference.starts_with("../") {
        return locator_path(reference)
            .split('/')
            .any(|segment| !segment.is_empty() && segment != "." && segment != "..");
    }
    let mut segments = reference.split('/');
    if reference.contains(['?', '#']) {
        return false;
    }
    matches!(
        (segments.next(), segments.next(), segments.next()),
        (Some(namespace), Some(name), None)
            if !namespace.is_empty()
                && !name.is_empty()
                && namespace != "."
                && namespace != ".."
                && name != "."
                && name != ".."
    )
}

fn locator_path(reference: &str) -> &str {
    reference.split(['?', '#']).next().unwrap_or_default()
}

fn absolute_uri(reference: &str) -> bool {
    let Some((scheme, rest)) = reference.split_once(':') else {
        return false;
    };
    !rest.is_empty()
        && scheme
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic())
        && scheme.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        })
}

fn uri_has_userinfo(reference: &str) -> bool {
    let Some((_, after_scheme)) = reference.split_once("://") else {
        return false;
    };
    after_scheme
        .split(['/', '?', '#'])
        .next()
        .is_some_and(|authority| authority.contains('@'))
}
