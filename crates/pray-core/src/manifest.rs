use crate::hashing::sha256_prefixed;
use crate::literal::{
    find_top_level, is_balanced, parse_literal, prepare_parser_lines, split_top_level, LiteralValue,
};
use crate::statement_surface::{split_symbol_assignment, SurfaceStatementReader};
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub prayfile_version: String,
    pub sources: Vec<ManifestSource>,
    pub targets: Vec<ManifestTarget>,
    pub packages: Vec<ManifestPackage>,
    pub local: Vec<ManifestLocal>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub symbols: BTreeMap<String, String>,
    pub render: RenderPolicy,
    /// Deprecated Prayfile keywords encountered while parsing (`target`, `output`, `agent`).
    #[serde(default, skip)]
    pub deprecated_keywords: Vec<String>,
}

impl Manifest {
    pub fn note_deprecated_keyword(&mut self, keyword: &str) {
        if !crate::deprecation::is_deprecated_prayfile_keyword(keyword) {
            return;
        }
        if self
            .deprecated_keywords
            .iter()
            .any(|existing| existing == keyword)
        {
            return;
        }
        self.deprecated_keywords.push(keyword.to_string());
    }

    pub fn deprecation_warnings(&self) -> Vec<String> {
        crate::deprecation::deprecation_warnings_for(&self.deprecated_keywords)
    }
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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DestinationMode {
    #[default]
    Legacy,
    Compose,
    Tree,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DestinationEntry {
    Package { name: String },
    Local { path: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportRole {
    Fragment,
    Folder,
    File,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestTarget {
    pub name: String,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    pub max_bytes: Option<u64>,
    #[serde(default)]
    pub mode: DestinationMode,
    #[serde(default)]
    pub scoped: bool,
    #[serde(default)]
    pub entries: Vec<DestinationEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestPackage {
    pub name: String,
    #[serde(default = "default_constraint")]
    pub constraint: String,
    pub source: Option<String>,
    #[serde(default)]
    pub exports: Vec<String>,
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub optional: bool,
    pub path: Option<String>,
    pub git: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
    pub tarball: Option<String>,
    pub oci: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<ExportRole>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bound: bool,
}

fn default_constraint() -> String {
    "*".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestLocal {
    pub path: String,
    #[serde(default = "default_local_position")]
    pub position: String,
    #[serde(default)]
    pub optional: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bound: bool,
}

fn default_local_position() -> String {
    "after".to_string()
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

pub fn read_manifest_text(manifest_path: &Path) -> PrayResult<String> {
    fs::read_to_string(manifest_path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            let manifest_label = manifest_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Prayfile");
            PrayError::Manifest(format!(
                "missing {manifest_label}; run pray init to create one"
            ))
        } else {
            PrayError::Io(error)
        }
    })
}

pub fn parse_manifest(text: &str) -> PrayResult<Manifest> {
    let lines = prepare_parser_lines(text);
    let mut parser = BlockParser::new(&lines);
    parser.parse_root()
}

struct BlockParser<'a> {
    lines: &'a [Cow<'a, str>],
    cursor: usize,
    group_stack: Vec<Vec<String>>,
    surface: SurfaceStatementReader,
}

impl<'a> BlockParser<'a> {
    fn new(lines: &'a [Cow<'a, str>]) -> Self {
        Self {
            lines,
            cursor: 0,
            group_stack: Vec::new(),
            surface: SurfaceStatementReader::default(),
        }
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
        crate::manifest_validate::validate_manifest_semantics(&manifest)?;
        Ok(manifest)
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
            manifest.note_deprecated_keyword(crate::deprecation::DEPRECATED_TARGET);
            let (target, is_block) = parse_target_header(rest)?;
            manifest.targets.push(target);
            if is_block {
                let index = manifest.targets.len() - 1;
                self.parse_target_block(manifest, index)?;
            }
            return Ok(());
        }
        if let Some(rest) = statement.strip_prefix("group ") {
            let (group_names, is_block) = parse_group_header(rest)?;
            if !is_block {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: "group must use a block".to_string(),
                });
            }
            if !self.group_stack.is_empty() {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: "nested group blocks are not supported".to_string(),
                });
            }
            self.group_stack.push(group_names);
            self.parse_group_block(manifest)?;
            self.group_stack.pop();
            return Ok(());
        }
        if let Some(rest) = statement.strip_prefix("agent ") {
            manifest.note_deprecated_keyword(crate::deprecation::DEPRECATED_AGENT);
            crate::destination::upsert_package(manifest, self.parse_package_with_groups(rest)?)?;
            return Ok(());
        }
        if let Some(rest) = statement.strip_prefix("package ") {
            crate::destination::upsert_package(manifest, self.parse_package_with_groups(rest)?)?;
            return Ok(());
        }
        if statement == "pray do" || statement == "template do" {
            self.parse_symbols_block(manifest)?;
            return Ok(());
        }
        if let Some(rest) = statement
            .strip_prefix("pray ")
            .or_else(|| statement.strip_prefix("use "))
            .or_else(|| statement.strip_prefix("include "))
        {
            self.apply_pray_statement(manifest, rest, None)?;
            return Ok(());
        }
        if let Some(rest) = statement
            .strip_prefix("compose ")
            .or_else(|| statement.strip_prefix("output "))
        {
            if statement.starts_with("output ") {
                if !statement.ends_with(" do") {
                    return Err(PrayError::Parse {
                        kind: "manifest",
                        message:
                            "top-level output must use a compose block (output \"path\" do ... end)"
                                .to_string(),
                    });
                }
                manifest.note_deprecated_keyword(crate::deprecation::DEPRECATED_OUTPUT);
            }
            self.parse_destination_block(manifest, rest, DestinationMode::Compose)?;
            return Ok(());
        }
        if let Some(rest) = statement
            .strip_prefix("tree ")
            .or_else(|| statement.strip_prefix("folder "))
            .or_else(|| statement.strip_prefix("skills "))
        {
            if (statement.starts_with("folder ") || statement.starts_with("skills "))
                && !statement.ends_with(" do")
            {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: "top-level folder/skills must use a tree block".to_string(),
                });
            }
            self.parse_destination_block(manifest, rest, DestinationMode::Tree)?;
            return Ok(());
        }
        if let Some(rest) = statement.strip_prefix("file ") {
            self.parse_file_block(manifest, rest)?;
            return Ok(());
        }
        if let Some(rest) = statement.strip_prefix("local ") {
            let mut local = parse_local_decl(rest)?;
            local.bound = false;
            crate::destination::upsert_local(manifest, local);
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

    fn parse_destination_block(
        &mut self,
        manifest: &mut Manifest,
        rest: &str,
        mode: DestinationMode,
    ) -> PrayResult<()> {
        let is_block = rest.trim_end().ends_with("do");
        if !is_block {
            return Err(PrayError::Parse {
                kind: "manifest",
                message: format!(
                    "{} must use a block",
                    match mode {
                        DestinationMode::Compose => "compose",
                        DestinationMode::Tree => "tree",
                        DestinationMode::Legacy => "destination",
                    }
                ),
            });
        }
        let header = rest.trim_end_matches("do").trim();
        let (values, _) = parse_call(header)?;
        let path = string_from_value(values.first().ok_or_else(|| PrayError::Parse {
            kind: "manifest",
            message: "destination missing path".to_string(),
        })?)?;
        let target = crate::destination::new_destination_target(mode, &path);
        manifest.targets.push(target);
        let index = manifest.targets.len() - 1;
        while let Some(statement) = self.next_statement()? {
            if statement == "end" {
                return Ok(());
            }
            if let Some(pray_rest) = statement
                .strip_prefix("pray ")
                .or_else(|| statement.strip_prefix("use "))
                .or_else(|| statement.strip_prefix("include "))
                .or_else(|| statement.strip_prefix("agent "))
                .or_else(|| statement.strip_prefix("package "))
            {
                self.apply_pray_statement(manifest, pray_rest, Some(index))?;
                continue;
            }
            if mode == DestinationMode::Compose {
                if let Some(local_rest) = statement.strip_prefix("local ") {
                    let mut local = parse_local_decl(local_rest)?;
                    local.bound = true;
                    crate::destination::bind_local_entry(&mut manifest.targets[index], &local.path);
                    crate::destination::upsert_local(manifest, local);
                    continue;
                }
            }
            return Err(PrayError::Parse {
                kind: "manifest",
                message: format!("unsupported statement inside destination block: {statement}"),
            });
        }
        Err(PrayError::Parse {
            kind: "manifest",
            message: "missing 'end' for destination block".to_string(),
        })
    }

    fn parse_symbols_block(&mut self, manifest: &mut Manifest) -> PrayResult<()> {
        while let Some(statement) = self.next_statement()? {
            if statement == "end" {
                return Ok(());
            }
            let Some((key, value_literal)) = split_symbol_assignment(&statement) else {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: format!(
                        "unsupported statement inside pray/template block: {statement}"
                    ),
                });
            };
            if !crate::substitute::is_pray_symbol_key(&key) {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: format!("invalid pray symbol key `{key}`"),
                });
            }
            if manifest.symbols.contains_key(&key) {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: format!("duplicate pray symbol `{key}`"),
                });
            }
            manifest
                .symbols
                .insert(key, string_from_literal(&value_literal)?);
        }
        Err(PrayError::Parse {
            kind: "manifest",
            message: "missing 'end' for pray/template block".to_string(),
        })
    }

    fn parse_file_block(&mut self, manifest: &mut Manifest, rest: &str) -> PrayResult<()> {
        let is_block = rest.trim_end().ends_with("do");
        if !is_block {
            return Err(PrayError::Parse {
                kind: "manifest",
                message: "file must use a block (or use pray ..., file: \"path\")".to_string(),
            });
        }
        let header = rest.trim_end_matches("do").trim();
        let (values, _) = parse_call(header)?;
        let file_path = string_from_value(values.first().ok_or_else(|| PrayError::Parse {
            kind: "manifest",
            message: "file block missing path".to_string(),
        })?)?;
        let mut saw_package = false;
        while let Some(statement) = self.next_statement()? {
            if statement == "end" {
                if !saw_package {
                    return Err(PrayError::Parse {
                        kind: "manifest",
                        message: "file block requires a pray package declaration".to_string(),
                    });
                }
                return Ok(());
            }
            if let Some(pray_rest) = statement
                .strip_prefix("pray ")
                .or_else(|| statement.strip_prefix("use "))
                .or_else(|| statement.strip_prefix("include "))
                .or_else(|| statement.strip_prefix("agent "))
                .or_else(|| statement.strip_prefix("package "))
            {
                let mut package = self.parse_package_with_groups(pray_rest)?;
                if package.file.is_some() {
                    return Err(PrayError::Parse {
                        kind: "manifest",
                        message: "file: keyword is invalid inside a file block".to_string(),
                    });
                }
                package.file = Some(file_path.clone());
                package.bound = true;
                if !package.roles.contains(&ExportRole::File) {
                    package.roles.push(ExportRole::File);
                }
                crate::destination::upsert_package(manifest, package)?;
                saw_package = true;
                continue;
            }
            return Err(PrayError::Parse {
                kind: "manifest",
                message: format!("unsupported statement inside file block: {statement}"),
            });
        }
        Err(PrayError::Parse {
            kind: "manifest",
            message: "missing 'end' for file block".to_string(),
        })
    }

    fn apply_pray_statement(
        &mut self,
        manifest: &mut Manifest,
        rest: &str,
        destination_index: Option<usize>,
    ) -> PrayResult<()> {
        let (values, keywords) = parse_call(rest)?;
        if values.is_empty() {
            return Err(PrayError::Parse {
                kind: "manifest",
                message: "pray missing package or path".to_string(),
            });
        }
        let first = string_from_value(&values[0])?;
        let has_package_signal = values.len() > 1
            || keywords.contains_key("source")
            || keywords.contains_key("export")
            || keywords.contains_key("exports")
            || keywords.contains_key("file")
            || keywords.contains_key("optional")
            || keywords.contains_key("path")
            || keywords.contains_key("git")
            || keywords.contains_key("tag")
            || keywords.contains_key("rev")
            || keywords.contains_key("tarball")
            || keywords.contains_key("oci")
            || keywords.contains_key("targets")
            || keywords.contains_key("features");

        let in_compose = destination_index.is_some_and(|index| {
            manifest
                .targets
                .get(index)
                .is_some_and(|target| target.mode == DestinationMode::Compose)
        });

        if !has_package_signal && crate::destination::is_local_path_form(&first) {
            if !in_compose {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: "local pray paths are only valid inside compose blocks".to_string(),
                });
            }
            let local = ManifestLocal {
                path: first,
                position: "after".to_string(),
                optional: false,
                bound: true,
            };
            if let Some(index) = destination_index {
                crate::destination::bind_local_entry(&mut manifest.targets[index], &local.path);
            }
            crate::destination::upsert_local(manifest, local);
            return Ok(());
        }

        let mut package = parse_package_decl(rest)?;
        package.groups = self.group_stack.last().cloned().unwrap_or_default();
        if package.file.is_some() {
            if destination_index.is_some() {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: "file: is mutually exclusive with compose/tree nesting".to_string(),
                });
            }
            package.bound = true;
            if !package.roles.contains(&ExportRole::File) {
                package.roles.push(ExportRole::File);
            }
        }
        if let Some(index) = destination_index {
            let mode = manifest.targets[index].mode;
            package.bound = true;
            if let Some(role) = crate::destination::role_for_destination(mode) {
                if !package.roles.contains(&role) {
                    package.roles.push(role);
                }
            }
            crate::destination::bind_package_entry(&mut manifest.targets[index], &package.name);
        }
        crate::destination::upsert_package(manifest, package)?;
        Ok(())
    }

    fn parse_group_block(&mut self, manifest: &mut Manifest) -> PrayResult<()> {
        while let Some(statement) = self.next_statement()? {
            if statement == "end" {
                return Ok(());
            }
            if let Some(rest) = statement.strip_prefix("agent ") {
                manifest.note_deprecated_keyword(crate::deprecation::DEPRECATED_AGENT);
                crate::destination::upsert_package(
                    manifest,
                    self.parse_package_with_groups(rest)?,
                )?;
                continue;
            }
            if let Some(rest) = statement
                .strip_prefix("package ")
                .or_else(|| statement.strip_prefix("pray "))
                .or_else(|| statement.strip_prefix("use "))
            {
                crate::destination::upsert_package(
                    manifest,
                    self.parse_package_with_groups(rest)?,
                )?;
                continue;
            }
            return Err(PrayError::Parse {
                kind: "manifest",
                message: format!(
                    "group blocks only support agent, package, or pray declarations: {statement}"
                ),
            });
        }
        Err(PrayError::Parse {
            kind: "manifest",
            message: "missing 'end' for group block".to_string(),
        })
    }

    fn parse_package_with_groups(&self, rest: &str) -> PrayResult<ManifestPackage> {
        let mut package = parse_package_decl(rest)?;
        package.groups = self.group_stack.last().cloned().unwrap_or_default();
        Ok(package)
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
            if manifest.targets.get(target_index).is_none() {
                return Err(PrayError::Manifest("target index out of range".to_string()));
            }
            apply_target_statement(manifest, target_index, statement)?;
        }
        Err(PrayError::Parse {
            kind: "manifest",
            message: "missing 'end' for target block".to_string(),
        })
    }

    fn next_statement(&mut self) -> PrayResult<Option<String>> {
        if let Some(pending) = self.surface.next_pending() {
            return Ok(Some(pending));
        }
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
            self.surface.push_raw(statement);
            if let Some(normalized) = self.surface.next_pending() {
                return Ok(Some(normalized));
            }
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
        mode: DestinationMode::Legacy,
        scoped: false,
        entries: Vec::new(),
    };
    Ok((target, is_block))
}

fn parse_group_header(rest: &str) -> PrayResult<(Vec<String>, bool)> {
    let is_block = rest.trim_end().ends_with("do");
    let header = rest.trim_end_matches("do").trim();
    let (values, _) = parse_call(header)?;
    if values.is_empty() {
        return Err(PrayError::Parse {
            kind: "manifest",
            message: "group missing name".to_string(),
        });
    }
    let names = values
        .iter()
        .map(string_from_value)
        .collect::<PrayResult<Vec<_>>>()?;
    Ok((names, is_block))
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
    let mut exports = keyword_array(&keywords, "exports");
    if let Some(export) = keywords.get("export").and_then(|value| value.as_string()) {
        if !exports.contains(&export.to_string()) {
            exports.push(export.to_string());
        }
    }
    let mut roles = Vec::new();
    let file = keywords
        .get("file")
        .and_then(|value| value.as_string())
        .map(str::to_string);
    if file.is_some() {
        roles.push(ExportRole::File);
    }
    Ok(ManifestPackage {
        name,
        constraint,
        source: keywords
            .get("source")
            .and_then(|value| value.as_string())
            .map(str::to_string),
        exports,
        targets: keyword_array(&keywords, "targets"),
        features: keyword_array(&keywords, "features"),
        groups: Vec::new(),
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
        file,
        roles,
        bound: false,
    })
}

fn parse_local_decl(rest: &str) -> PrayResult<ManifestLocal> {
    let (values, keywords) = parse_call(rest)?;
    let path = string_from_value(values.first().ok_or_else(|| PrayError::Parse {
        kind: "manifest",
        message: "local missing path".to_string(),
    })?)?;
    let position = keywords
        .get("position")
        .or_else(|| keywords.get("at"))
        .and_then(|value| value.as_string())
        .unwrap_or("after")
        .to_string();
    Ok(ManifestLocal {
        path,
        position,
        optional: keywords
            .get("optional")
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
        bound: false,
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

fn apply_target_statement(
    manifest: &mut Manifest,
    target_index: usize,
    statement: String,
) -> PrayResult<()> {
    if statement.starts_with("output ") {
        manifest.note_deprecated_keyword(crate::deprecation::DEPRECATED_OUTPUT);
    }
    let target = manifest
        .targets
        .get_mut(target_index)
        .ok_or_else(|| PrayError::Manifest("target index out of range".to_string()))?;
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

pub fn format_package_declaration(package: &ManifestPackage) -> String {
    let mut parts = vec![format!("pray \"{}\"", package.name)];
    if package.constraint != "*" {
        parts.push(format!("\"{}\"", package.constraint));
    }
    if let Some(path) = &package.path {
        parts.push(format!("path: \"{path}\""));
    }
    if let Some(source) = &package.source {
        parts.push(format!("source: \"{source}\""));
    }
    if let Some(git) = &package.git {
        parts.push(format!("git: \"{git}\""));
    }
    if let Some(tag) = &package.tag {
        parts.push(format!("tag: \"{tag}\""));
    }
    if let Some(rev) = &package.rev {
        parts.push(format!("rev: \"{rev}\""));
    }
    if let Some(tarball) = &package.tarball {
        parts.push(format!("tarball: \"{tarball}\""));
    }
    if let Some(oci) = &package.oci {
        parts.push(format!("oci: \"{oci}\""));
    }
    if let Some(file) = &package.file {
        parts.push(format!("file: \"{file}\""));
    }
    if !package.exports.is_empty() {
        if package.exports.len() == 1 {
            parts.push(format!("export: \"{}\"", package.exports[0]));
        } else {
            parts.push(format!(
                "exports: [{}]",
                format_string_keyword_list(&package.exports)
            ));
        }
    }
    if !package.targets.is_empty() {
        parts.push(format!(
            "targets: [{}]",
            format_string_keyword_list(&package.targets)
        ));
    }
    if !package.features.is_empty() {
        parts.push(format!(
            "features: [{}]",
            format_string_keyword_list(&package.features)
        ));
    }
    if package.optional {
        parts.push("optional: true".to_string());
    }
    parts.join(", ")
}

pub fn replace_package_declaration(text: &str, package: &ManifestPackage) -> PrayResult<String> {
    let name = &package.name;
    let prefixes = [
        format!("pray \"{name}\""),
        format!("pray '{name}'"),
        format!("use \"{name}\""),
        format!("include \"{name}\""),
        format!("agent \"{name}\""),
        format!("agent '{name}'"),
        format!("package \"{name}\""),
        format!("package '{name}'"),
    ];
    let mut lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let index = lines
        .iter()
        .position(|line| {
            let trimmed = line.trim_start();
            prefixes
                .iter()
                .any(|prefix| trimmed.starts_with(prefix.as_str()))
        })
        .ok_or_else(|| PrayError::Manifest(format!("package {name} not found in manifest")))?;
    lines[index] = format_package_declaration(package);
    let mut output = lines.join("\n");
    if text.ends_with('\n') && !output.ends_with('\n') {
        output.push('\n');
    }
    Ok(output)
}

fn format_string_keyword_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ")
}
