use crate::hashing::sha256_prefixed;
use crate::literal::{
    find_top_level, is_balanced, parse_literal, parse_literal_map, split_top_level, LiteralValue,
};
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PackageSpec {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub source_code_uri: Option<String>,
    pub changelog_uri: Option<String>,
    pub prayfile_version: Option<String>,
    pub files: Vec<String>,
    pub exports: BTreeMap<String, PackageExport>,
    pub skills: BTreeMap<String, PackageSkill>,
    pub templates: BTreeMap<String, PackageTemplate>,
    pub adapters: BTreeMap<String, String>,
    pub targets: Vec<String>,
    pub dependencies: Vec<PackageDependency>,
    pub metadata: BTreeMap<String, LiteralValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageExport {
    pub kind: String,
    pub path: String,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageSkill {
    pub path: String,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageTemplate {
    pub path: String,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageDependency {
    pub name: String,
    pub constraint: String,
    pub optional: bool,
}

impl PackageSpec {
    pub fn canonicalized(&self) -> Self {
        let mut package = self.clone();
        package.files.sort();
        package.authors.sort();
        package.targets.sort();
        package.dependencies.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then(left.constraint.cmp(&right.constraint))
                .then(left.optional.cmp(&right.optional))
        });
        package
    }

    pub fn tree_hash_for_root(&self, root: &std::path::Path) -> PrayResult<String> {
        let mut entries = Vec::new();
        for file in &self.files {
            let path = root.join(file);
            if !path.exists() {
                return Err(PrayError::Integrity(format!(
                    "package file missing: {}",
                    file
                )));
            }
            if path.is_dir() {
                return Err(PrayError::Integrity(format!(
                    "package file is a directory: {}",
                    file
                )));
            }
            let bytes = std::fs::read(&path)?;
            entries.push((file.clone(), sha256_prefixed(&bytes)));
        }
        entries.sort_by(|left, right| left.0.cmp(&right.0));

        let mut serialized = String::new();
        for (path, hash) in entries {
            serialized.push_str("file");
            serialized.push('\0');
            serialized.push_str("regular");
            serialized.push('\0');
            serialized.push_str(&path);
            serialized.push('\0');
            serialized.push_str(&hash);
            serialized.push('\n');
        }
        Ok(sha256_prefixed(serialized.as_bytes()))
    }
}

pub fn parse_package_spec(text: &str) -> PrayResult<PackageSpec> {
    let lines = prepare_lines(text);
    let mut parser = BlockParser::new(&lines);
    parser.parse_root()
}

fn prepare_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| strip_comment(line).trim_end().to_string())
        .collect()
}

fn strip_comment(line: &str) -> String {
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
            '#' => return line[..index].to_string(),
            _ => {}
        }
    }
    line.to_string()
}

struct BlockParser<'a> {
    lines: &'a [String],
    cursor: usize,
}

impl<'a> BlockParser<'a> {
    fn new(lines: &'a [String]) -> Self {
        Self { lines, cursor: 0 }
    }

    fn parse_root(&mut self) -> PrayResult<PackageSpec> {
        self.expect_start()?;
        let mut spec = PackageSpec::default();
        while let Some(statement) = self.next_statement()? {
            if statement == "end" {
                return Ok(spec.canonicalized());
            }
            self.apply_statement(&mut spec, statement)?;
        }
        Err(PrayError::Parse {
            kind: "prayspec",
            message: "missing 'end'".to_string(),
        })
    }

    fn expect_start(&mut self) -> PrayResult<()> {
        let statement = self.next_statement()?.ok_or_else(|| PrayError::Parse {
            kind: "prayspec",
            message: "empty package spec".to_string(),
        })?;
        if !statement.starts_with("Package::Specification.new") {
            return Err(PrayError::Parse {
                kind: "prayspec",
                message: "expected Package::Specification.new".to_string(),
            });
        }
        Ok(())
    }

    fn apply_statement(&mut self, spec: &mut PackageSpec, statement: String) -> PrayResult<()> {
        if let Some(rest) = statement.strip_prefix("spec.add_dependency ") {
            spec.dependencies.push(parse_dependency(rest, false)?);
            return Ok(());
        }
        if let Some(rest) = statement.strip_prefix("spec.add_optional_dependency ") {
            spec.dependencies.push(parse_dependency(rest, true)?);
            return Ok(());
        }
        if let Some(rest) = statement.strip_prefix("spec.") {
            if let Some((name, value)) = rest.split_once(" = ") {
                return self.apply_assignment(spec, name.trim(), value.trim());
            }
        }
        Err(PrayError::Parse {
            kind: "prayspec",
            message: format!("unrecognized statement: {statement}"),
        })
    }

    fn apply_assignment(&self, spec: &mut PackageSpec, field: &str, value: &str) -> PrayResult<()> {
        match field {
            "name" => spec.name = string_from_literal(value)?,
            "version" => spec.version = string_from_literal(value)?,
            "summary" => spec.summary = Some(string_from_literal(value)?),
            "description" => spec.description = Some(string_from_literal(value)?),
            "authors" => spec.authors = array_of_strings(value)?,
            "license" => spec.license = Some(string_from_literal(value)?),
            "homepage" => spec.homepage = Some(string_from_literal(value)?),
            "source_code_uri" => spec.source_code_uri = Some(string_from_literal(value)?),
            "changelog_uri" => spec.changelog_uri = Some(string_from_literal(value)?),
            "prayfile_version" => spec.prayfile_version = Some(string_from_literal(value)?),
            "files" => spec.files = array_of_strings(value)?,
            "targets" => spec.targets = array_of_strings(value)?,
            "exports" => spec.exports = parse_exports(value)?,
            "skills" => spec.skills = parse_skills(value)?,
            "templates" => spec.templates = parse_templates(value)?,
            "adapters" => spec.adapters = parse_string_map(value)?,
            "metadata" => spec.metadata = parse_metadata(value)?,
            _ => {
                return Err(PrayError::Parse {
                    kind: "prayspec",
                    message: format!("unsupported assignment: {field}"),
                })
            }
        }
        Ok(())
    }

    fn next_statement(&mut self) -> PrayResult<Option<String>> {
        while self.cursor < self.lines.len() {
            let mut statement = self.lines[self.cursor].trim().to_string();
            self.cursor += 1;
            if statement.is_empty() {
                continue;
            }
            while !statement.ends_with(" do")
                && statement != "end"
                && self.cursor < self.lines.len()
                && (statement.trim_end().ends_with(',') || !is_balanced(&statement))
            {
                let next = self.lines[self.cursor].trim();
                self.cursor += 1;
                if next.is_empty() {
                    continue;
                }
                statement.push(' ');
                statement.push_str(next);
            }
            return Ok(Some(statement));
        }
        Ok(None)
    }
}

fn parse_dependency(rest: &str, optional: bool) -> PrayResult<PackageDependency> {
    let (values, keywords) = parse_call(rest)?;
    let name = string_from_value(values.first().ok_or_else(|| PrayError::Parse {
        kind: "prayspec",
        message: "missing dependency name".to_string(),
    })?)?;
    let constraint = values
        .get(1)
        .map(string_from_value)
        .transpose()?
        .unwrap_or("*".to_string());
    Ok(PackageDependency {
        name,
        constraint,
        optional: keywords
            .get("optional")
            .and_then(|value| value.as_bool())
            .unwrap_or(optional),
    })
}

fn parse_call(rest: &str) -> PrayResult<(Vec<LiteralValue>, BTreeMap<String, LiteralValue>)> {
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

fn parse_keyword_segment(segment: &str) -> PrayResult<Option<(String, LiteralValue)>> {
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
        return Ok(Some((left.to_string(), parse_literal(right)?)));
    }
    Ok(None)
}

fn parse_exports(value: &str) -> PrayResult<BTreeMap<String, PackageExport>> {
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
            },
        );
    }
    Ok(exports)
}

fn parse_skills(value: &str) -> PrayResult<BTreeMap<String, PackageSkill>> {
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

fn parse_templates(value: &str) -> PrayResult<BTreeMap<String, PackageTemplate>> {
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

fn parse_string_map(value: &str) -> PrayResult<BTreeMap<String, String>> {
    let map = parse_literal_map(value)?;
    let mut output = BTreeMap::new();
    for (key, literal) in map {
        output.insert(key, string_from_value(&literal)?);
    }
    Ok(output)
}

fn parse_metadata(value: &str) -> PrayResult<BTreeMap<String, LiteralValue>> {
    parse_literal_map(value)
}

fn map_string(map: &BTreeMap<String, LiteralValue>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|value| value.as_string().map(str::to_string))
}

fn array_of_strings(value: &str) -> PrayResult<Vec<String>> {
    let array = parse_literal(value)?;
    let values = array.as_array().ok_or_else(|| PrayError::Parse {
        kind: "prayspec",
        message: "expected array".to_string(),
    })?;
    values.iter().map(string_from_value).collect()
}

fn string_from_value(value: &LiteralValue) -> PrayResult<String> {
    value
        .as_string()
        .map(str::to_string)
        .ok_or_else(|| PrayError::Parse {
            kind: "prayspec",
            message: format!("expected string-like literal, found {:?}", value),
        })
}

fn string_from_literal(value: &str) -> PrayResult<String> {
    string_from_value(&parse_literal(value)?)
}
