use crate::hashing::sha256_prefixed;
use crate::literal::{
    find_top_level, is_balanced, parse_literal, prepare_parser_lines, split_top_level, LiteralValue,
};
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub prayfile_version: String,
    pub sources: Vec<ManifestSource>,
    pub targets: Vec<ManifestTarget>,
    pub packages: Vec<ManifestPackage>,
    pub local: Vec<ManifestLocal>,
    pub render: RenderPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestSource {
    pub name: String,
    pub kind: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subdir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestTarget {
    pub name: String,
    pub outputs: Vec<String>,
    pub skills: Vec<String>,
    pub commands: Vec<String>,
    pub rules: Vec<String>,
    pub max_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestPackage {
    pub name: String,
    pub constraint: String,
    pub source: Option<String>,
    pub exports: Vec<String>,
    pub targets: Vec<String>,
    pub features: Vec<String>,
    pub optional: bool,
    pub path: Option<String>,
    pub git: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
    pub tarball: Option<String>,
    pub oci: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestLocal {
    pub path: String,
    pub position: String,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenderPolicy {
    pub mode: String,
    pub conflict: String,
    pub churn: String,
    pub header: bool,
    pub section_markers: bool,
    pub line_endings: String,
}

impl Default for RenderPolicy {
    fn default() -> Self {
        Self {
            mode: "managed".to_string(),
            conflict: "fail".to_string(),
            churn: "minimal".to_string(),
            header: true,
            section_markers: true,
            line_endings: "lf".to_string(),
        }
    }
}

impl Manifest {
    pub fn canonicalized(&self) -> Self {
        let mut manifest = self.clone();
        manifest
            .sources
            .sort_by(|left, right| left.name.cmp(&right.name));
        manifest
            .targets
            .sort_by(|left, right| left.name.cmp(&right.name));
        manifest.packages.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then(left.source.cmp(&right.source))
                .then(left.constraint.cmp(&right.constraint))
        });
        manifest
            .local
            .sort_by(|left, right| left.path.cmp(&right.path));
        manifest
    }

    pub fn manifest_hash(&self) -> PrayResult<String> {
        let canonical = self.canonicalized();
        let bytes = serde_json::to_vec(&canonical)
            .map_err(|error| PrayError::Manifest(error.to_string()))?;
        Ok(sha256_prefixed(&bytes))
    }
}

pub fn parse_manifest(text: &str) -> PrayResult<Manifest> {
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

    fn parse_root(&mut self) -> PrayResult<Manifest> {
        let mut manifest = Manifest::default();
        while let Some(statement) = self.next_statement()? {
            if statement == "end" {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: "unexpected 'end'".to_string(),
                });
            }
            self.apply_statement(&mut manifest, statement, false)?;
        }
        if manifest.prayfile_version.is_empty() {
            return Err(PrayError::Manifest("missing prayfile version".to_string()));
        }
        Ok(manifest)
    }

    fn parse_nested(&mut self, manifest: &mut Manifest, stop_on_end: bool) -> PrayResult<()> {
        while let Some(statement) = self.next_statement()? {
            if statement == "end" {
                if stop_on_end {
                    return Ok(());
                }
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: "unexpected 'end'".to_string(),
                });
            }
            self.apply_statement(manifest, statement, true)?;
        }
        if stop_on_end {
            Err(PrayError::Parse {
                kind: "manifest",
                message: "missing 'end'".to_string(),
            })
        } else {
            Ok(())
        }
    }

    fn apply_statement(
        &mut self,
        manifest: &mut Manifest,
        statement: String,
        allow_target: bool,
    ) -> PrayResult<()> {
        if let Some(rest) = statement.strip_prefix("prayfile ") {
            manifest.prayfile_version = string_from_literal(rest)?;
            return Ok(());
        }
        if let Some(rest) = statement.strip_prefix("source ") {
            manifest.sources.push(parse_source(rest)?);
            return Ok(());
        }
        if let Some(rest) = statement.strip_prefix("target ") {
            if !allow_target && !statement.ends_with(" do") {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: "target must use a block".to_string(),
                });
            }
            let (target, is_block) = parse_target_header(rest)?;
            manifest.targets.push(target);
            if is_block {
                let index = manifest.targets.len() - 1;
                self.parse_target_block(manifest, index)?;
            }
            return Ok(());
        }
        if let Some(rest) = statement.strip_prefix("group ") {
            let (_group_name, is_block) = parse_group_header(rest)?;
            if is_block {
                self.parse_nested(manifest, true)?;
            }
            return Ok(());
        }
        if let Some(rest) = statement.strip_prefix("agent ") {
            manifest.packages.push(parse_package_decl(rest)?);
            return Ok(());
        }
        if let Some(rest) = statement.strip_prefix("local ") {
            manifest.local.push(parse_local_decl(rest)?);
            return Ok(());
        }
        if let Some(rest) = statement.strip_prefix("render ") {
            manifest.render = parse_render_policy(rest)?;
            return Ok(());
        }
        Err(PrayError::Parse {
            kind: "manifest",
            message: format!("unrecognized statement: {statement}"),
        })
    }

    fn parse_target_block(
        &mut self,
        manifest: &mut Manifest,
        target_index: usize,
    ) -> PrayResult<()> {
        while let Some(statement) = self.next_statement()? {
            if statement == "end" {
                return Ok(());
            }
            let target = manifest
                .targets
                .get_mut(target_index)
                .ok_or_else(|| PrayError::Manifest("target index out of range".to_string()))?;
            apply_target_statement(target, statement)?;
        }
        Err(PrayError::Parse {
            kind: "manifest",
            message: "missing 'end' for target block".to_string(),
        })
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

fn parse_source(rest: &str) -> PrayResult<ManifestSource> {
    let (values, keywords) = parse_call(rest)?;
    if values.is_empty() {
        return Err(PrayError::Parse {
            kind: "manifest",
            message: "source requires a name".to_string(),
        });
    }
    if values.len() < 2 && !keywords.contains_key("path") && !keywords.contains_key("git") {
        return Err(PrayError::Parse {
            kind: "manifest",
            message: "source requires a name and url, path:, or git:".to_string(),
        });
    }
    let name = string_from_value(values.first().ok_or_else(|| PrayError::Parse {
        kind: "manifest",
        message: "source missing name".to_string(),
    })?)?;
    let (kind, url) = if let Some(path) = keywords.get("path") {
        ("path".to_string(), string_from_value(path)?)
    } else if let Some(git) = keywords.get("git") {
        let mut url = string_from_value(git)?;
        if !url.starts_with("git+") {
            url = format!("git+{url}");
        }
        (String::from("git"), url)
    } else {
        let url = string_from_value(values.get(1).ok_or_else(|| PrayError::Parse {
            kind: "manifest",
            message: "source missing url".to_string(),
        })?)?;
        let kind = if url.starts_with("git+") {
            "git"
        } else if url.starts_with("pray+ssh://") || url.starts_with("ssh+pray://") {
            "pray_ssh"
        } else {
            "registry"
        };
        (kind.to_string(), url)
    };
    let subdir = keywords
        .get("subdir")
        .or_else(|| keywords.get("distribution"))
        .map(string_from_value)
        .transpose()?;
    let rev = keywords.get("rev").map(string_from_value).transpose()?;
    let tag = keywords.get("tag").map(string_from_value).transpose()?;
    Ok(ManifestSource {
        name,
        kind,
        url,
        subdir,
        rev,
        tag,
    })
}

fn parse_target_header(rest: &str) -> PrayResult<(ManifestTarget, bool)> {
    let is_block = rest.trim_end().ends_with("do");
    let header = rest.trim_end_matches("do").trim();
    let (values, keywords) = parse_call(header)?;
    let name = string_from_value(values.first().ok_or_else(|| PrayError::Parse {
        kind: "manifest",
        message: "target missing name".to_string(),
    })?)?;
    let outputs = keyword_array(&keywords, "output");
    let mut folders = keyword_array(&keywords, "folder");
    folders.extend(keyword_array(&keywords, "skills"));
    let target = ManifestTarget {
        name,
        outputs,
        skills: folders,
        commands: keyword_array(&keywords, "commands"),
        rules: keyword_array(&keywords, "rules"),
        max_bytes: keywords
            .get("max_bytes")
            .and_then(|value| value.as_integer())
            .map(|value| value as u64),
    };
    Ok((target, is_block))
}

fn parse_group_header(rest: &str) -> PrayResult<(String, bool)> {
    let is_block = rest.trim_end().ends_with("do");
    let header = rest.trim_end_matches("do").trim();
    let (values, _) = parse_call(header)?;
    let name = string_from_value(values.first().ok_or_else(|| PrayError::Parse {
        kind: "manifest",
        message: "group missing name".to_string(),
    })?)?;
    Ok((name, is_block))
}

fn parse_package_decl(rest: &str) -> PrayResult<ManifestPackage> {
    let (values, keywords) = parse_call(rest)?;
    if values.is_empty() {
        return Err(PrayError::Parse {
            kind: "manifest",
            message: "agent missing name".to_string(),
        });
    }
    let name = string_from_value(&values[0])?;
    let constraint = if let Some(value) = values.get(1) {
        crate::constraint::normalize_version_constraint(&string_from_value(value)?)
    } else {
        "*".to_string()
    };
    Ok(ManifestPackage {
        name,
        constraint,
        source: keywords
            .get("source")
            .and_then(|value| value.as_string())
            .map(str::to_string),
        exports: keyword_array(&keywords, "exports"),
        targets: keyword_array(&keywords, "targets"),
        features: keyword_array(&keywords, "features"),
        optional: keywords
            .get("optional")
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
        path: keywords
            .get("path")
            .and_then(|value| value.as_string())
            .map(str::to_string),
        git: keywords
            .get("git")
            .and_then(|value| value.as_string())
            .map(str::to_string),
        tag: keywords
            .get("tag")
            .and_then(|value| value.as_string())
            .map(str::to_string),
        rev: keywords
            .get("rev")
            .and_then(|value| value.as_string())
            .map(str::to_string),
        tarball: keywords
            .get("tarball")
            .and_then(|value| value.as_string())
            .map(str::to_string),
        oci: keywords
            .get("oci")
            .and_then(|value| value.as_string())
            .map(str::to_string),
    })
}

fn parse_local_decl(rest: &str) -> PrayResult<ManifestLocal> {
    let (values, keywords) = parse_call(rest)?;
    let path = string_from_value(values.first().ok_or_else(|| PrayError::Parse {
        kind: "manifest",
        message: "local missing path".to_string(),
    })?)?;
    Ok(ManifestLocal {
        path,
        position: keywords
            .get("position")
            .and_then(|value| value.as_string())
            .unwrap_or("after")
            .to_string(),
        optional: keywords
            .get("optional")
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
    })
}

fn parse_render_policy(rest: &str) -> PrayResult<RenderPolicy> {
    let (_, keywords) = parse_call(rest)?;
    Ok(RenderPolicy {
        mode: keywords
            .get("mode")
            .and_then(|value| value.as_string())
            .unwrap_or("managed")
            .to_string(),
        conflict: keywords
            .get("conflict")
            .and_then(|value| value.as_string())
            .unwrap_or("fail")
            .to_string(),
        churn: keywords
            .get("churn")
            .and_then(|value| value.as_string())
            .unwrap_or("minimal")
            .to_string(),
        header: keywords
            .get("header")
            .and_then(|value| value.as_bool())
            .unwrap_or(true),
        section_markers: keywords
            .get("section_markers")
            .and_then(|value| value.as_bool())
            .unwrap_or(true),
        line_endings: keywords
            .get("line_endings")
            .and_then(|value| value.as_string())
            .unwrap_or("lf")
            .to_string(),
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
        let key = left.trim().trim_start_matches(':').to_string();
        return Ok(Some((key, parse_literal(right)?)));
    }
    Ok(None)
}

fn keyword_array(keywords: &BTreeMap<String, LiteralValue>, key: &str) -> Vec<String> {
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

fn string_from_value(value: &LiteralValue) -> PrayResult<String> {
    value
        .as_string()
        .map(str::to_string)
        .ok_or_else(|| PrayError::Parse {
            kind: "manifest",
            message: format!("expected string-like literal, found {:?}", value),
        })
}

fn string_from_literal(input: &str) -> PrayResult<String> {
    string_from_value(&parse_literal(input)?)
}

fn apply_target_statement(target: &mut ManifestTarget, statement: String) -> PrayResult<()> {
    if let Some(rest) = statement.strip_prefix("output ") {
        target.outputs.push(string_from_literal(rest)?);
        return Ok(());
    }
    if let Some(rest) = statement.strip_prefix("folder ") {
        target.skills.push(string_from_literal(rest)?);
        return Ok(());
    }
    if let Some(rest) = statement.strip_prefix("skills ") {
        target.skills.push(string_from_literal(rest)?);
        return Ok(());
    }
    if let Some(rest) = statement.strip_prefix("commands ") {
        target.commands.push(string_from_literal(rest)?);
        return Ok(());
    }
    if let Some(rest) = statement.strip_prefix("rules ") {
        target.rules.push(string_from_literal(rest)?);
        return Ok(());
    }
    if let Some(rest) = statement.strip_prefix("max_bytes ") {
        let value = parse_literal(rest.trim())?;
        target.max_bytes = value.as_integer().map(|number| number as u64);
        return Ok(());
    }
    Err(PrayError::Parse {
        kind: "manifest",
        message: format!("unrecognized target statement: {statement}"),
    })
}
