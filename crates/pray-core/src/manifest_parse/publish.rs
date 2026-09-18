use super::call::{parse_call, string_from_value};
use super::decls::parse_package_decl;
use super::BlockParser;
use crate::publish_remote::ManifestPublishRemote;
use crate::{PrayError, PrayResult};

impl BlockParser<'_> {
    pub(super) fn apply_publish_statement(
        &mut self,
        remotes: &mut Vec<ManifestPublishRemote>,
        rest: &str,
    ) -> PrayResult<()> {
        let is_block = rest.trim_end().ends_with(" do");
        let header = rest.trim_end().strip_suffix(" do").unwrap_or(rest).trim();
        let mut remote = parse_publish_header(header)?;
        if is_block {
            self.parse_publish_block(&mut remote)?;
        }
        remotes.push(remote);
        Ok(())
    }

    fn parse_publish_block(&mut self, remote: &mut ManifestPublishRemote) -> PrayResult<()> {
        while let Some(statement) = self.next_statement()? {
            if statement == "end" {
                return Ok(());
            }
            let Some(pray_rest) = statement
                .strip_prefix("pray ")
                .or_else(|| statement.strip_prefix("use "))
                .or_else(|| statement.strip_prefix("include "))
                .or_else(|| statement.strip_prefix("agent "))
                .or_else(|| statement.strip_prefix("package "))
            else {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: format!("publish blocks only support pray package names: {statement}"),
                });
            };
            let package = parse_package_decl(pray_rest)?;
            if remote.packages.iter().any(|name| name == &package.name) {
                return Err(PrayError::Parse {
                    kind: "manifest",
                    message: format!(
                        "duplicate package {} in publish \"{}\"",
                        package.name, remote.name
                    ),
                });
            }
            remote.packages.push(package.name);
        }
        Err(PrayError::Parse {
            kind: "manifest",
            message: "missing 'end' for publish block".to_string(),
        })
    }
}

pub(super) fn parse_publish_header(rest: &str) -> PrayResult<ManifestPublishRemote> {
    let (values, keywords) = parse_call(rest)?;
    if values.is_empty() {
        return Err(PrayError::Parse {
            kind: "manifest",
            message: "publish requires a name".to_string(),
        });
    }
    let name = string_from_value(&values[0])?;
    let path = keywords.get("path").map(string_from_value).transpose()?;
    let url = if let Some(value) = values.get(1) {
        Some(string_from_value(value)?)
    } else {
        None
    };
    if keywords.contains_key("git")
        || keywords.contains_key("source")
        || keywords.contains_key("signing_key")
        || keywords.contains_key("token")
    {
        return Err(PrayError::Parse {
            kind: "manifest",
            message: format!(
                "publish \"{name}\" does not take git:, source:, signing_key:, or token:"
            ),
        });
    }
    Ok(ManifestPublishRemote {
        name,
        path,
        url,
        packages: Vec::new(),
    })
}
