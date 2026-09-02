use super::call::{keyword_array, parse_call, string_from_literal, string_from_value};
use crate::literal::parse_literal;
use crate::manifest::{
    DestinationMode, ExportRole, Manifest, ManifestLocal, ManifestPackage, ManifestSource,
    ManifestTarget,
};
use crate::{PrayError, PrayResult};

pub(super) fn parse_source(rest: &str) -> PrayResult<ManifestSource> {
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

pub(super) fn parse_target_header(rest: &str) -> PrayResult<(ManifestTarget, bool)> {
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
        header: None,
    };
    Ok((target, is_block))
}

pub(super) fn parse_group_header(rest: &str) -> PrayResult<(Vec<String>, bool)> {
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

pub(super) fn parse_package_decl(rest: &str) -> PrayResult<ManifestPackage> {
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

pub(super) fn parse_local_decl(rest: &str) -> PrayResult<ManifestLocal> {
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

pub(super) fn apply_target_statement(
    manifest: &mut Manifest,
    target_index: usize,
    statement: String,
) -> PrayResult<()> {
    if statement.starts_with("output ") {
        manifest.note_deprecated_keyword(crate::deprecation::DEPRECATED_OUTPUT);
    }
    if statement.starts_with("skills ") {
        manifest.note_deprecated_keyword(crate::deprecation::DEPRECATED_SKILLS);
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
