use crate::literal::{find_top_level, parse_literal, split_top_level, LiteralValue};
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;

pub(super) fn parse_call(
    rest: &str,
) -> PrayResult<(Vec<LiteralValue>, BTreeMap<String, LiteralValue>)> {
    let mut positional = Vec::new();
    let mut keywords = BTreeMap::new();
    for segment in split_top_level(rest.trim().trim_end_matches(','), ',') {
        if let Some((key, value)) = parse_keyword_segment(&segment)? {
            keywords.insert(key, value);
        } else if !segment.is_empty() {
            positional.push(parse_literal(&segment)?);
        }
    }
    Ok((positional, keywords))
}

pub(super) fn parse_keyword_segment(segment: &str) -> PrayResult<Option<(String, LiteralValue)>> {
    if let Some(index) = find_top_level(segment, "=>") {
        let key = string_from_literal(segment[..index].trim())?;
        return Ok(Some((key, parse_literal(segment[index + 2..].trim())?)));
    }
    if let Some(index) = find_top_level(segment, ":") {
        let left = segment[..index].trim();
        let right = segment[index + 1..].trim();
        if left.is_empty() {
            return Ok(None);
        }
        let key = left.trim().trim_start_matches(':').to_string();
        return Ok(Some((key, parse_literal(right)?)));
    }
    Ok(None)
}

pub(super) fn keyword_array(keywords: &BTreeMap<String, LiteralValue>, key: &str) -> Vec<String> {
    keywords
        .get(key)
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_string().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn string_from_value(value: &LiteralValue) -> PrayResult<String> {
    value
        .as_string()
        .map(str::to_string)
        .ok_or_else(|| PrayError::Parse {
            kind: "manifest",
            message: format!("expected string-like literal, found {:?}", value),
        })
}

pub(super) fn string_from_literal(input: &str) -> PrayResult<String> {
    string_from_value(&parse_literal(input)?)
}
