use crate::literal::{is_balanced, prepare_parser_lines};
use crate::manifest::{DestinationMode, Manifest, ManifestPackage};
use crate::statement_surface::SurfaceStatementReader;
use crate::{PrayError, PrayResult};
use std::borrow::Cow;

mod blocks;
mod call;
mod decls;

use call::string_from_literal;
use decls::{
    apply_target_statement, parse_group_header, parse_local_decl, parse_package_decl,
    parse_render_policy, parse_source, parse_target_header,
};

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
