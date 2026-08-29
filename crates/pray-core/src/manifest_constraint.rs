use crate::{PrayError, PrayResult};

const PACKAGE_KEYWORDS: [&str; 5] = ["pray", "use", "include", "agent", "package"];

pub fn rewrite_constraint_on_line(line: &str, constraint: &str) -> PrayResult<String> {
    let indent_len = line.len() - line.trim_start().len();
    let indent = &line[..indent_len];
    let trimmed = line.trim_start();
    let after_keyword = skip_package_keyword(trimmed).ok_or_else(|| {
        PrayError::Manifest("package declaration is missing a keyword".to_string())
    })?;
    let after_keyword = after_keyword.trim_start();
    let (name, after_name) = parse_quoted(after_keyword).ok_or_else(|| {
        PrayError::Manifest("package declaration is missing a quoted name".to_string())
    })?;
    let quoted_constraint = format!("\"{constraint}\"");
    let keyword_and_name = &trimmed[..trimmed.len() - after_name.len()];
    let remainder = after_name.trim_start();
    if remainder.is_empty() {
        return Ok(format!("{indent}{keyword_and_name}, {quoted_constraint}"));
    }
    if !remainder.starts_with(',') {
        return Err(PrayError::Manifest(format!(
            "package {name} declaration is missing a comma after the name"
        )));
    }
    let after_comma = remainder[1..].trim_start();
    if after_comma.starts_with('"') || after_comma.starts_with('\'') {
        let (_, after_constraint) = parse_quoted(after_comma).ok_or_else(|| {
            PrayError::Manifest(format!(
                "package {name} declaration has an unclosed constraint"
            ))
        })?;
        return Ok(format!(
            "{indent}{keyword_and_name}, {quoted_constraint}{after_constraint}"
        ));
    }
    Ok(format!(
        "{indent}{keyword_and_name}, {quoted_constraint}, {after_comma}"
    ))
}

fn skip_package_keyword(input: &str) -> Option<&str> {
    for keyword in PACKAGE_KEYWORDS {
        if let Some(rest) = input.strip_prefix(keyword) {
            if rest.starts_with(char::is_whitespace)
                || rest.starts_with('"')
                || rest.starts_with('\'')
            {
                return Some(rest);
            }
        }
    }
    None
}

fn parse_quoted(input: &str) -> Option<(&str, &str)> {
    let quote = input.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = input.get(quote.len_utf8()..)?;
    let end = rest.find(quote)?;
    Some((&rest[..end], &rest[end + quote.len_utf8()..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_existing_constraint_and_keeps_keywords() {
        let line = r#"  pray "sample/base", "~> 1.0", export: "testing-basics""#;
        let rewritten = rewrite_constraint_on_line(line, "~> 1.1").expect("rewrite");
        assert_eq!(
            rewritten,
            r#"  pray "sample/base", "~> 1.1", export: "testing-basics""#
        );
    }

    #[test]
    fn inserts_constraint_before_keyword_arguments() {
        let line = r#"    pray "sample/base", path: "packages/base""#;
        let rewritten = rewrite_constraint_on_line(line, "~> 1.1").expect("rewrite");
        assert_eq!(
            rewritten,
            r#"    pray "sample/base", "~> 1.1", path: "packages/base""#
        );
    }

    #[test]
    fn appends_constraint_when_the_line_has_only_a_name() {
        let line = r#"pray "sample/base""#;
        let rewritten = rewrite_constraint_on_line(line, "~> 1.1").expect("rewrite");
        assert_eq!(rewritten, r#"pray "sample/base", "~> 1.1""#);
    }
}
