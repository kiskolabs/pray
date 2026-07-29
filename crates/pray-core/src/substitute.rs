use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;

/// Strict placeholder form: `((pray:<path>))` with no spaces.
/// Resolver is fixed to `pray` in v1; grammar leaves room for other resolvers later.
const PLACEHOLDER_PREFIX: &str = "((pray:";
const PLACEHOLDER_SUFFIX: &str = "))";

pub fn is_pray_symbol_key(key: &str) -> bool {
    if key.is_empty() {
        return false;
    }
    key.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '/' | '-')
    })
}

pub fn substitute_pray_symbols(
    text: &str,
    symbols: &BTreeMap<String, String>,
) -> PrayResult<String> {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(start) = rest.find(PLACEHOLDER_PREFIX) {
        output.push_str(&rest[..start]);
        let after_prefix = &rest[start + PLACEHOLDER_PREFIX.len()..];
        let Some(end) = after_prefix.find(PLACEHOLDER_SUFFIX) else {
            return Err(PrayError::Render(
                "unclosed ((pray:...) placeholder".to_string(),
            ));
        };
        let path = &after_prefix[..end];
        if !is_pray_symbol_key(path) {
            return Err(PrayError::Render(format!(
                "invalid ((pray:...)) path `{path}`"
            )));
        }
        let Some(value) = symbols.get(path) else {
            return Err(PrayError::Render(format!(
                "unknown pray symbol `{path}`; declare it in `pray do ... end`"
            )));
        };
        output.push_str(value);
        rest = &after_prefix[end + PLACEHOLDER_SUFFIX.len()..];
    }

    output.push_str(rest);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_known_symbols() {
        let mut symbols = BTreeMap::new();
        symbols.insert("support_email".to_string(), "a@example.com".to_string());
        symbols.insert("security_email".to_string(), "b@example.com".to_string());
        let text = "write ((pray:support_email)) or ((pray:security_email))";
        assert_eq!(
            substitute_pray_symbols(text, &symbols).expect("ok"),
            "write a@example.com or b@example.com"
        );
    }

    #[test]
    fn rejects_unknown_symbol() {
        let symbols = BTreeMap::new();
        let error = substitute_pray_symbols("((pray:missing))", &symbols).unwrap_err();
        assert!(error.to_string().contains("unknown pray symbol"));
    }

    #[test]
    fn ignores_spaced_forms() {
        let mut symbols = BTreeMap::new();
        symbols.insert("email".to_string(), "a@example.com".to_string());
        let text = "(( pray:email )) ((pray : email))";
        assert_eq!(substitute_pray_symbols(text, &symbols).expect("ok"), text);
    }
}
