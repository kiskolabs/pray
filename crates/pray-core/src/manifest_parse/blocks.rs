use super::call::{parse_call, string_from_literal, string_from_value};
use super::decls::{parse_local_decl, parse_package_decl};
use super::BlockParser;
use crate::manifest::{DestinationMode, ExportRole, Manifest, ManifestLocal};
use crate::statement_surface::split_symbol_assignment;
use crate::{PrayError, PrayResult};

impl BlockParser<'_> {
    pub(super) fn parse_destination_block(
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

    pub(super) fn parse_symbols_block(&mut self, manifest: &mut Manifest) -> PrayResult<()> {
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

    pub(super) fn parse_file_block(&mut self, manifest: &mut Manifest, rest: &str) -> PrayResult<()> {
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

    pub(super) fn apply_pray_statement(
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

    pub(super) fn parse_group_block(&mut self, manifest: &mut Manifest) -> PrayResult<()> {
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

}
