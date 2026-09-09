use super::{PackageExport, PackageSkill, PackageTemplate};
use crate::literal::{parse_literal, parse_literal_map, LiteralValue};
use crate::{PrayError, PrayResult};
use std::collections::BTreeMap;

pub(super) fn parse_exports(value: &str) -> PrayResult<BTreeMap<String, PackageExport>> {
    let map = parse_literal_map(value)?;
    let mut exports = BTreeMap::new();
    for (name, literal) in map {
        let entry = literal.as_map().ok_or_else(|| PrayError::Parse {
            kind: "prayspec",
            message: format!("export {name} must be a map"),
        })?;
        let missing_path_name = name.clone();
        exports.insert(
            name,
            PackageExport {
                kind: map_string(entry, "type").unwrap_or_else(|| "fragment".to_string()),
                path: map_string(entry, "path").ok_or_else(|| PrayError::Parse {
                    kind: "prayspec",
                    message: format!("export {missing_path_name} missing path"),
                })?,
                summary: map_string(entry, "summary"),
                only: map_string_array(entry, "only"),
                except: map_string_array(entry, "except"),
                default_path: map_string(entry, "default_path"),
            },
        );
    }
    Ok(exports)
}

pub(super) fn parse_skills(value: &str) -> PrayResult<BTreeMap<String, PackageSkill>> {
    let map = parse_literal_map(value)?;
    let mut output = BTreeMap::new();
    for (name, literal) in map {
        let entry = literal.as_map().ok_or_else(|| PrayError::Parse {
            kind: "prayspec",
            message: format!("skill {name} must be a map"),
        })?;
        output.insert(
            name,
            PackageSkill {
                path: map_string(entry, "path").ok_or_else(|| PrayError::Parse {
                    kind: "prayspec",
                    message: "skill missing path".to_string(),
                })?,
                summary: map_string(entry, "summary"),
            },
        );
    }
    Ok(output)
}

pub(super) fn parse_templates(value: &str) -> PrayResult<BTreeMap<String, PackageTemplate>> {
    let map = parse_literal_map(value)?;
    let mut output = BTreeMap::new();
    for (name, literal) in map {
        let entry = literal.as_map().ok_or_else(|| PrayError::Parse {
            kind: "prayspec",
            message: format!("template {name} must be a map"),
        })?;
        output.insert(
            name,
            PackageTemplate {
                path: map_string(entry, "path").ok_or_else(|| PrayError::Parse {
                    kind: "prayspec",
                    message: "template missing path".to_string(),
                })?,
                summary: map_string(entry, "summary"),
            },
        );
    }
    Ok(output)
}

pub(super) fn parse_string_map(value: &str) -> PrayResult<BTreeMap<String, String>> {
    let map = parse_literal_map(value)?;
    let mut output = BTreeMap::new();
    for (key, literal) in map {
        output.insert(key, string_from_value(&literal)?);
    }
    Ok(output)
}

pub(super) fn parse_metadata(value: &str) -> PrayResult<BTreeMap<String, LiteralValue>> {
    parse_literal_map(value)
}

fn map_string(map: &BTreeMap<String, LiteralValue>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|value| value.as_string().map(str::to_string))
}

fn map_string_array(map: &BTreeMap<String, LiteralValue>, key: &str) -> Vec<String> {
    map.get(key)
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_string().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn array_of_strings(value: &str) -> PrayResult<Vec<String>> {
    let array = parse_literal(value)?;
    let values = array.as_array().ok_or_else(|| PrayError::Parse {
        kind: "prayspec",
        message: "expected array".to_string(),
    })?;
    values.iter().map(string_from_value).collect()
}

pub(super) fn string_from_value(value: &LiteralValue) -> PrayResult<String> {
    value
        .as_string()
        .map(str::to_string)
        .ok_or_else(|| PrayError::Parse {
            kind: "prayspec",
            message: format!("expected string-like literal, found {:?}", value),
        })
}

pub(super) fn string_from_literal(value: &str) -> PrayResult<String> {
    string_from_value(&parse_literal(value)?)
}
