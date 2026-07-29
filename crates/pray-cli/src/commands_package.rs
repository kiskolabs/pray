use crate::auth_client::{login_with_passkey, login_with_ssh_agent};
use crate::materialize::{materialize_package_directory, write_package_archive};
use crate::project_paths::{
    lockfile_path, manifest_path, resolve_project, resolve_project_with_options, workspace_root,
};
use pray_core::hashing::normalize_line_endings;
use pray_core::lockfile::read_lockfile;
use pray_core::manifest::{parse_manifest, read_manifest_text};
use pray_core::resolve_context::ResolveOptions;
use pray_core::{PrayError, PrayResult};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn format_command() -> PrayResult<()> {
    let path = manifest_path();
    let original = read_manifest_text(&path)?;
    let manifest = parse_manifest(&original)?;
    let project = resolve_project_with_options(
        &path,
        &ResolveOptions {
            offline: true,
            ..ResolveOptions::default()
        },
    )
    .or_else(|_| resolve_project(&path))?;
    let hints = pray_core::format_manifest::classify_format_hints(&project);
    let formatted = pray_core::format_manifest::format_recommended(&manifest, &hints)?;
    if formatted != original {
        fs::write(&path, formatted)?;
    }

    if let Ok(lockfile) = read_lockfile(&lockfile_path()) {
        for target in &lockfile.target {
            for output in &target.outputs {
                let output_path = Path::new(output);
                if !output_path.exists() {
                    continue;
                }
                let original_output = fs::read_to_string(output_path)?;
                let formatted_output =
                    format_marker_comments(&normalize_line_endings(&original_output));
                if formatted_output != original_output {
                    fs::write(output_path, formatted_output)?;
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn package_command() -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    for package in &project.packages {
        let output_path = package_archive_path(&package.declaration.name, &package.spec.version);
        write_package_archive(package, &output_path)?;
    }
    Ok(())
}

pub(crate) fn login_command(
    servers: Vec<String>,
    email: String,
    credential_id: Option<String>,
    passkey_key: Option<PathBuf>,
    public_key: Option<PathBuf>,
    ssh_agent: bool,
) -> PrayResult<()> {
    let session_root = workspace_root();
    for server in servers {
        let session = if let Some(passkey_key) = &passkey_key {
            let credential_id = credential_id.as_ref().ok_or_else(|| {
                PrayError::Unsupported("passkey login requires --credential-id".to_string())
            })?;
            login_with_passkey(&server, credential_id, passkey_key, &session_root)?
        } else if ssh_agent {
            let public_key = public_key.as_ref().ok_or_else(|| {
                PrayError::Unsupported("ssh-agent login requires --public-key".to_string())
            })?;
            login_with_ssh_agent(&server, public_key, &session_root)?
        } else {
            return Err(PrayError::Unsupported(
                "login requires an authentication mode".to_string(),
            ));
        };
        if session.email != email {
            return Err(PrayError::Resolution(format!(
                "login completed for {} but {} was requested",
                session.email, email
            )));
        }
        println!(
            "logged in as {} via {} on {}",
            session.email, session.kind, server
        );
    }
    Ok(())
}

pub(crate) fn vendor_command() -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    for package in &project.packages {
        let output_directory =
            vendor_package_path(&package.declaration.name, &package.spec.version);
        materialize_package_directory(package, &output_directory)?;
    }
    Ok(())
}

fn package_archive_path(name: &str, version: &str) -> PathBuf {
    PathBuf::from(format!("{}-{}.praypkg", name.replace('/', "-"), version))
}

fn vendor_package_path(name: &str, version: &str) -> PathBuf {
    PathBuf::from(".pray/vendor")
        .join(name.replace('/', "-"))
        .join(version)
}

fn format_marker_comments(text: &str) -> String {
    let lines: Vec<String> = text
        .split('\n')
        .map(|line| match canonical_marker_line(line) {
            Some(marker) => marker,
            None => line.to_string(),
        })
        .collect();
    let mut output = lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn canonical_marker_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let remainder = trimmed.strip_prefix("<!--")?.trim_start();
    let remainder = remainder.strip_prefix("pray:")?;
    let content = remainder.strip_suffix("-->")?.trim();
    if content == "0 ignore-comments" {
        return Some("<!-- pray:0 ignore-comments -->".to_string());
    }
    if content
        .chars()
        .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    {
        return Some(format!("<!-- pray:{content} -->"));
    }
    None
}
