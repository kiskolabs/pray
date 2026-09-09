use crate::literal::{
    find_top_level, is_balanced, parse_literal, prepare_parser_lines, split_top_level, LiteralValue,
};
use crate::package_upstream::{parse_upstream, PackageUpstream};
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;

#[path = "package_spec_maps.rs"]
mod maps;
#[path = "package_spec_hash.rs"]
mod package_hash;
use maps::{
    array_of_strings, parse_exports, parse_metadata, parse_skills, parse_string_map,
    parse_templates, string_from_literal, string_from_value,
};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream: Option<PackageUpstream>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageExport {
    pub kind: String,
    pub path: String,
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub except: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_path: Option<String>,
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

pub fn parse_package_spec(text: &str) -> PrayResult<PackageSpec> {
    let lines = prepare_parser_lines(text);
    let mut parser = BlockParser::new(&lines);
    parser.parse_root()
}

struct BlockParser<'a> {
    lines: &'a [Cow<'a, str>],
    cursor: usize,
}

impl<'a> BlockParser<'a> {
    fn new(lines: &'a [Cow<'a, str>]) -> Self {
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
        if let Some(rest) = statement.strip_prefix("spec.upstream ") {
            if spec.upstream.is_some() {
                return Err(PrayError::Parse {
                    kind: "prayspec",
                    message: "upstream may only be declared once".to_string(),
                });
            }
            spec.upstream = Some(parse_upstream(rest)?);
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
