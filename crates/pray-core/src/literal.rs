use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralValue {
    String(String),
    Symbol(String),
    Bool(bool),
    Null,
    Integer(i64),
    Array(Vec<LiteralValue>),
    Map(BTreeMap<String, LiteralValue>),
}

impl LiteralValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(value) | Self::Symbol(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[LiteralValue]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&BTreeMap<String, LiteralValue>> {
        match self {
            Self::Map(values) => Some(values),
            _ => None,
        }
    }
}

pub fn split_top_level(input: &str, separator: char) -> Vec<String> {
    let mut output = Vec::new();
    let mut start = 0usize;
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
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth -= 1,
            _ if character == separator && depth == 0 => {
                output.push(input[start..index].trim().to_string());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }

    if start < input.len() {
        output.push(input[start..].trim().to_string());
    }

    output
        .into_iter()
        .filter(|segment| !segment.is_empty())
        .collect()
}

pub fn find_top_level(input: &str, token: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let characters: Vec<(usize, char)> = input.char_indices().collect();

    let mut index = 0usize;
    while index < characters.len() {
        let (byte_index, character) = characters[index];
        if let Some(quote_char) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == quote_char {
                quote = None;
            }
            index += 1;
            continue;
        }

        match character {
            '"' | '\'' => quote = Some(character),
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth -= 1,
            _ if depth == 0 && input[byte_index..].starts_with(token) => return Some(byte_index),
            _ => {}
        }
        index += 1;
    }
    None
}

pub fn is_balanced(input: &str) -> bool {
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for character in input.chars() {
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
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth -= 1,
            _ => {}
        }
    }

    depth == 0 && quote.is_none()
}

pub fn parse_literal(input: &str) -> PrayResult<LiteralValue> {
    let mut parser = LiteralParser::new(input);
    let value = parser.parse_value()?;
    parser.skip_whitespace();
    if !parser.is_finished() {
        return Err(PrayError::Parse {
            kind: "literal",
            message: format!("unexpected trailing input near {:?}", parser.remaining()),
        });
    }
    Ok(value)
}

pub fn parse_literal_map(input: &str) -> PrayResult<BTreeMap<String, LiteralValue>> {
    match parse_literal(input)? {
        LiteralValue::Map(entries) => Ok(entries),
        other => Err(PrayError::Parse {
            kind: "literal",
            message: format!("expected map literal, found {:?}", other),
        }),
    }
}

pub fn parse_literal_array(input: &str) -> PrayResult<Vec<LiteralValue>> {
    match parse_literal(input)? {
        LiteralValue::Array(entries) => Ok(entries),
        other => Err(PrayError::Parse {
            kind: "literal",
            message: format!("expected array literal, found {:?}", other),
        }),
    }
}

struct LiteralParser<'a> {
    input: &'a str,
    cursor: usize,
}

impl<'a> LiteralParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, cursor: 0 }
    }

    fn is_finished(&self) -> bool {
        self.cursor >= self.input.len()
    }

    fn remaining(&self) -> &str {
        &self.input[self.cursor..]
    }

    fn skip_whitespace(&mut self) {
        while let Some(character) = self.peek() {
            if character.is_whitespace() {
                self.cursor += character.len_utf8();
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn next(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.cursor += character.len_utf8();
        Some(character)
    }

    fn parse_value(&mut self) -> PrayResult<LiteralValue> {
        self.skip_whitespace();
        match self.peek() {
            Some('"') | Some('\'') => self.parse_string(),
            Some(':') => self.parse_symbol(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_map(),
            Some(character) if character.is_ascii_digit() || character == '-' => {
                self.parse_integer_or_identifier()
            }
            Some(_) => self.parse_identifier(),
            None => Err(PrayError::Parse {
                kind: "literal",
                message: "unexpected end of input".to_string(),
            }),
        }
    }

    fn parse_string(&mut self) -> PrayResult<LiteralValue> {
        let quote = self.next().unwrap();
        let mut output = String::new();
        let mut escaped = false;
        while let Some(character) = self.next() {
            if escaped {
                output.push(match character {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    other => other,
                });
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if character == quote {
                return Ok(LiteralValue::String(output));
            }
            output.push(character);
        }
        Err(PrayError::Parse {
            kind: "literal",
            message: "unterminated string literal".to_string(),
        })
    }

    fn parse_symbol(&mut self) -> PrayResult<LiteralValue> {
        self.next();
        let mut output = String::new();
        while let Some(character) = self.peek() {
            if character.is_alphanumeric() || matches!(character, '_' | '-' | '.' | '/') {
                output.push(character);
                self.next();
            } else {
                break;
            }
        }
        if output.is_empty() {
            return Err(PrayError::Parse {
                kind: "literal",
                message: "empty symbol".to_string(),
            });
        }
        Ok(LiteralValue::Symbol(output))
    }

    fn parse_array(&mut self) -> PrayResult<LiteralValue> {
        self.next();
        let mut values = Vec::new();
        loop {
            self.skip_whitespace();
            if matches!(self.peek(), Some(']')) {
                self.next();
                break;
            }
            values.push(self.parse_value()?);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.next();
                }
                Some(']') => {
                    self.next();
                    break;
                }
                _ => {
                    return Err(PrayError::Parse {
                        kind: "literal",
                        message: "expected ',' or ']'".to_string(),
                    })
                }
            }
        }
        Ok(LiteralValue::Array(values))
    }

    fn parse_map(&mut self) -> PrayResult<LiteralValue> {
        self.next();
        let mut entries = BTreeMap::new();
        loop {
            self.skip_whitespace();
            if matches!(self.peek(), Some('}')) {
                self.next();
                break;
            }
            let key = self.parse_map_key()?;
            self.skip_whitespace();
            if self.remaining().starts_with("=>") {
                self.next();
                self.next();
            } else if matches!(self.peek(), Some(':')) {
                self.next();
            } else {
                return Err(PrayError::Parse {
                    kind: "literal",
                    message: "expected ':' or '=>' after map key".to_string(),
                });
            }
            let value = self.parse_value()?;
            entries.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.next();
                }
                Some('}') => {
                    self.next();
                    break;
                }
                _ => {
                    return Err(PrayError::Parse {
                        kind: "literal",
                        message: "expected ',' or '}'".to_string(),
                    })
                }
            }
        }
        Ok(LiteralValue::Map(entries))
    }

    fn parse_map_key(&mut self) -> PrayResult<String> {
        self.skip_whitespace();
        match self.peek() {
            Some('"') | Some('\'') => match self.parse_string()? {
                LiteralValue::String(value) => Ok(value),
                _ => unreachable!(),
            },
            Some(':') => match self.parse_symbol()? {
                LiteralValue::Symbol(value) => Ok(value),
                _ => unreachable!(),
            },
            Some(character) if is_identifier_start(character) => self.parse_identifier_name(),
            _ => Err(PrayError::Parse {
                kind: "literal",
                message: "invalid map key".to_string(),
            }),
        }
    }

    fn parse_integer_or_identifier(&mut self) -> PrayResult<LiteralValue> {
        let start = self.cursor;
        if matches!(self.peek(), Some('-')) {
            self.next();
        }
        while matches!(self.peek(), Some(character) if character.is_ascii_digit() || character == '_')
        {
            self.next();
        }
        if matches!(self.peek(), Some('.')) {
            return self.parse_identifier_from(start);
        }
        let text = self.input[start..self.cursor].replace('_', "");
        let parsed = text.parse::<i64>().map_err(|error| PrayError::Parse {
            kind: "literal",
            message: error.to_string(),
        })?;
        Ok(LiteralValue::Integer(parsed))
    }

    fn parse_identifier(&mut self) -> PrayResult<LiteralValue> {
        let identifier = self.parse_identifier_name()?;
        match identifier.as_str() {
            "true" => Ok(LiteralValue::Bool(true)),
            "false" => Ok(LiteralValue::Bool(false)),
            "nil" => Ok(LiteralValue::Null),
            other => Ok(LiteralValue::String(other.to_string())),
        }
    }

    fn parse_identifier_name(&mut self) -> PrayResult<String> {
        let start = self.cursor;
        if !matches!(self.peek(), Some(character) if is_identifier_start(character)) {
            return Err(PrayError::Parse {
                kind: "literal",
                message: "expected identifier".to_string(),
            });
        }
        self.next();
        while matches!(self.peek(), Some(character) if is_identifier_continue(character)) {
            self.next();
        }
        Ok(self.input[start..self.cursor].to_string())
    }

    fn parse_identifier_from(&mut self, start: usize) -> PrayResult<LiteralValue> {
        while matches!(self.peek(), Some(character) if is_identifier_continue(character) || character == '.')
        {
            self.next();
        }
        Ok(LiteralValue::String(
            self.input[start..self.cursor].to_string(),
        ))
    }
}

fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_identifier_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
}

pub fn prepare_parser_lines(text: &str) -> Vec<Cow<'_, str>> {
    text.lines().map(prepare_parser_line).collect()
}

fn prepare_parser_line(line: &str) -> Cow<'_, str> {
    trim_end_cow(strip_line_comment(line))
}

pub fn strip_line_comment(line: &str) -> Cow<'_, str> {
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (index, character) in line.char_indices() {
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
            '#' => return Cow::Borrowed(&line[..index]),
            _ => {}
        }
    }
    Cow::Borrowed(line)
}

fn trim_end_cow<'a>(value: Cow<'a, str>) -> Cow<'a, str> {
    match value {
        Cow::Borrowed(slice) => {
            let trimmed = slice.trim_end();
            if trimmed.len() == slice.len() {
                Cow::Borrowed(slice)
            } else {
                Cow::Owned(trimmed.to_string())
            }
        }
        Cow::Owned(mut owned) => {
            let trimmed_len = owned.trim_end().len();
            owned.truncate(trimmed_len);
            Cow::Owned(owned)
        }
    }
}
