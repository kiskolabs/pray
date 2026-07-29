//! Normalize Gemfile-like Ruby surface sugar into the canonical Prayfile statement form.
//!
//! Supported (static, non-executable):
//! - top-level `;` statement separators
//! - `{ … }` blocks as `do` / `end`
//! - optional call parentheses after keywords (`compose("x") do`)
//! - optional call parentheses on symbol assignments (`support_email("x")`)
//!
//! Not supported (intentional): interpolation, constants, variables, method chaining.

use crate::literal::{is_balanced, split_top_level};
use std::collections::VecDeque;

pub fn expand_statement_surface(statement: &str) -> Vec<String> {
    let trimmed = statement.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut parts = Vec::new();
    for segment in split_top_level(trimmed, ';') {
        parts.extend(expand_one_surface(&segment));
    }
    parts
}

fn expand_one_surface(statement: &str) -> Vec<String> {
    let trimmed = statement.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if let Some(expanded) = expand_brace_block(trimmed) {
        return expanded;
    }
    vec![normalize_keyword_call(trimmed)]
}

fn expand_brace_block(statement: &str) -> Option<Vec<String>> {
    let keyword = leading_identifier(statement)?;
    let after_keyword = statement[keyword.len()..].trim_start();
    // Only `keyword{…}`, `keyword(…){…}`, or `keyword "…"{…}` — not `spec.exports = {…}`.
    let (args, after_open) = split_brace_header(after_keyword)?;
    let close_offset = matching_close_brace(after_open)?;
    let trailing = after_open[close_offset + 1..].trim();
    if !trailing.is_empty() {
        return None;
    }
    let body = after_open[..close_offset].trim();
    if !is_balanced(body) {
        return None;
    }

    let header_args = unwrap_outer_parens(&args);
    let open = if header_args.is_empty() {
        format!("{keyword} do")
    } else {
        format!("{keyword} {header_args} do")
    };

    let mut output = vec![open];
    if !body.is_empty() {
        output.extend(expand_statement_surface(body));
    }
    output.push("end".to_string());
    Some(output)
}

fn split_brace_header(after_keyword: &str) -> Option<(String, &str)> {
    if let Some(rest) = after_keyword.strip_prefix('{') {
        return Some((String::new(), rest));
    }
    if after_keyword.starts_with('(') {
        let close = matching_close_paren(after_keyword)?;
        let trailing = after_keyword[close + 1..].trim_start();
        let rest = trailing.strip_prefix('{')?;
        let args = after_keyword[1..close].trim().to_string();
        return Some((args, rest));
    }
    let first = after_keyword.chars().next()?;
    if first != '"' && first != '\'' && first != ':' {
        return None;
    }
    let brace_offset = find_top_level_char(after_keyword, '{')?;
    let args = after_keyword[..brace_offset].trim().to_string();
    Some((args, &after_keyword[brace_offset + 1..]))
}

fn normalize_keyword_call(statement: &str) -> String {
    if let Some(normalized) = normalize_spaced_block_opener(statement) {
        return normalized;
    }
    let Some(keyword) = leading_identifier(statement) else {
        return statement.to_string();
    };
    let after_keyword = statement[keyword.len()..].trim_start();
    if !after_keyword.starts_with('(') {
        return statement.to_string();
    }
    let Some(close) = matching_close_paren(after_keyword) else {
        return statement.to_string();
    };
    let inner = after_keyword[1..close].trim();
    let trailing = after_keyword[close + 1..].trim();
    if trailing.is_empty() {
        format!("{keyword} {inner}")
    } else {
        format!("{keyword} {inner} {trailing}")
    }
}

fn normalize_spaced_block_opener(statement: &str) -> Option<String> {
    let trimmed = statement.trim();
    for keyword in ["pray", "template"] {
        let rest = trimmed.strip_prefix(keyword)?.trim_start();
        if rest == "do" {
            return Some(format!("{keyword} do"));
        }
    }
    None
}

pub fn split_symbol_assignment(statement: &str) -> Option<(String, String)> {
    let trimmed = statement.trim();
    if let Some((key, value)) = split_symbol_call(trimmed) {
        return Some((key, value));
    }
    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let key = parts.next()?.trim();
    let value = parts.next()?.trim();
    if key.is_empty() || value.is_empty() {
        return None;
    }
    Some((key.to_string(), value.to_string()))
}

fn split_symbol_call(statement: &str) -> Option<(String, String)> {
    let key = leading_identifier(statement)?;
    let after_key = statement[key.len()..].trim_start();
    if !after_key.starts_with('(') || !after_key.ends_with(')') {
        return None;
    }
    if matching_close_paren(after_key)? != after_key.len() - 1 {
        return None;
    }
    let inner = after_key[1..after_key.len() - 1].trim();
    if inner.is_empty() {
        return None;
    }
    Some((key.to_string(), inner.to_string()))
}

fn leading_identifier(input: &str) -> Option<&str> {
    let trimmed = input.trim_start();
    let end = trimmed
        .char_indices()
        .find(|(_, character)| !character.is_ascii_alphanumeric() && *character != '_')
        .map(|(index, _)| index)
        .unwrap_or(trimmed.len());
    if end == 0 {
        return None;
    }
    let ident = &trimmed[..end];
    if !ident.chars().next()?.is_ascii_alphabetic() {
        return None;
    }
    Some(ident)
}

fn unwrap_outer_parens(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.starts_with('(') && matching_close_paren(trimmed) == Some(trimmed.len() - 1) {
        return trimmed[1..trimmed.len() - 1].trim().to_string();
    }
    trimmed.to_string()
}

fn matching_close_paren(input: &str) -> Option<usize> {
    matching_close_delimited(input, '(', ')')
}

fn matching_close_brace(input: &str) -> Option<usize> {
    // Body is scanned after the opening `{`, so find the matching `}` at depth 0
    // relative to that body (depth starts at 1 for the already-consumed opener).
    let mut depth = 1i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (index, character) in input.char_indices() {
        if let Some(quote_char) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == quote_char {
                quote = None;
            }
            continue;
        }
        match character {
            '"' | '\'' => quote = Some(character),
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn matching_close_delimited(input: &str, open: char, close: char) -> Option<usize> {
    if !input.starts_with(open) {
        return None;
    }
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (index, character) in input.char_indices() {
        if let Some(quote_char) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == quote_char {
                quote = None;
            }
            continue;
        }
        if character == open {
            depth += 1;
        } else if character == close {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn find_top_level_char(input: &str, needle: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (index, character) in input.char_indices() {
        if let Some(quote_char) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == quote_char {
                quote = None;
            }
            continue;
        }
        match character {
            '"' | '\'' => quote = Some(character),
            '(' | '[' | '{' => {
                if depth == 0 && character == needle {
                    return Some(index);
                }
                depth += 1;
            }
            ')' | ']' | '}' => depth -= 1,
            _ if depth == 0 && character == needle => return Some(index),
            _ => {}
        }
    }
    None
}

#[derive(Debug, Default)]
pub struct SurfaceStatementReader {
    pending: VecDeque<String>,
}

impl SurfaceStatementReader {
    pub fn push_raw(&mut self, statement: String) {
        for part in expand_statement_surface(&statement) {
            self.pending.push_back(part);
        }
    }

    pub fn next_pending(&mut self) -> Option<String> {
        self.pending.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_semicolon_one_liner() {
        let parts = expand_statement_surface(
            r#"pray do; support_email("a@example.com"); security_email("b@example.com"); end"#,
        );
        assert_eq!(
            parts,
            vec![
                "pray do".to_string(),
                r#"support_email "a@example.com""#.to_string(),
                r#"security_email "b@example.com""#.to_string(),
                "end".to_string(),
            ]
        );
    }

    #[test]
    fn expands_brace_block() {
        let parts = expand_statement_surface(
            r#"pray{support_email("a@example.com");security_email("b@example.com")}"#,
        );
        assert_eq!(
            parts,
            vec![
                "pray do".to_string(),
                r#"support_email "a@example.com""#.to_string(),
                r#"security_email "b@example.com""#.to_string(),
                "end".to_string(),
            ]
        );
    }

    #[test]
    fn unwraps_compose_call_parens() {
        let parts = expand_statement_surface(r#"compose("AGENTS.md") do"#);
        assert_eq!(parts, vec![r#"compose "AGENTS.md" do"#.to_string()]);
    }

    #[test]
    fn expands_compose_brace_block() {
        let parts =
            expand_statement_surface(r#"compose("AGENTS.md"){ pray "sample/base", "~> 1.0" }"#);
        assert_eq!(
            parts,
            vec![
                r#"compose "AGENTS.md" do"#.to_string(),
                r#"pray "sample/base", "~> 1.0""#.to_string(),
                "end".to_string(),
            ]
        );
    }

    #[test]
    fn splits_symbol_call_form() {
        let (key, value) =
            split_symbol_assignment(r#"support_email("contact@kiskolabs.com")"#).expect("split");
        assert_eq!(key, "support_email");
        assert_eq!(value, r#""contact@kiskolabs.com""#);
    }

    #[test]
    fn leaves_assignment_map_literals_alone() {
        let statement = r#"spec.exports = { "AGENTS.md" => "templates/agents.md" }"#;
        assert_eq!(
            expand_statement_surface(statement),
            vec![statement.to_string()]
        );
    }
}
