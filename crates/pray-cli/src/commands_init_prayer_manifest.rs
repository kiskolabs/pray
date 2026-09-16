use crate::commands_manifest_edit::insert_manifest_statement;
use crate::project_paths::manifest_path;
use pray_core::local_prayer::{DEFAULT_PATH_SOURCE_DIRECTORY, DEFAULT_PATH_SOURCE_NAME};
use pray_core::manifest::{parse_manifest, read_manifest_text, Manifest, ManifestSource};
use pray_core::transaction;
use pray_core::{PrayError, PrayResult};

pub(crate) struct LocalPrayerSource {
    pub name: String,
    pub directory: String,
}

pub(crate) fn resolve_local_prayer_directory(
    requested: Option<&str>,
) -> PrayResult<LocalPrayerSource> {
    let manifest = parse_manifest(&read_manifest_text(&manifest_path())?)?;
    let path_sources = path_sources(&manifest);
    if let Some(directory) = requested {
        let directory = directory.trim();
        if directory.is_empty() {
            return Err(PrayError::Usage("--path requires a directory".to_string()));
        }
        if let Some(source) = path_sources
            .iter()
            .copied()
            .find(|source| source.url == directory)
        {
            return Ok(LocalPrayerSource {
                name: source.name.clone(),
                directory: source.url.clone(),
            });
        }
        if path_sources.len() == 1 {
            return Err(PrayError::Usage(format!(
                "path source already uses {}",
                path_sources[0].url
            )));
        }
        let name = ensure_path_source(directory)?;
        return Ok(LocalPrayerSource {
            name,
            directory: directory.to_string(),
        });
    }
    match path_sources.len() {
        0 => {
            let name = ensure_path_source(DEFAULT_PATH_SOURCE_DIRECTORY)?;
            Ok(LocalPrayerSource {
                name,
                directory: DEFAULT_PATH_SOURCE_DIRECTORY.to_string(),
            })
        }
        1 => Ok(LocalPrayerSource {
            name: path_sources[0].name.clone(),
            directory: path_sources[0].url.clone(),
        }),
        _ => Err(PrayError::Usage(
            "say which directory with --path".to_string(),
        )),
    }
}

pub(crate) fn declare_local_prayer(package_name: &str) -> PrayResult<()> {
    let manifest_path = manifest_path();
    let manifest_text = read_manifest_text(&manifest_path)?;
    let manifest = parse_manifest(&manifest_text)?;
    if manifest
        .packages
        .iter()
        .any(|package| package.name == package_name)
    {
        return Ok(());
    }
    let statement = format!("pray \"{package_name}\"");
    let updated = insert_into_first_compose(&manifest_text, &statement)
        .unwrap_or_else(|| insert_manifest_statement(&manifest_text, &statement));
    transaction::write_file(&manifest_path, updated)?;
    Ok(())
}

fn path_sources(manifest: &Manifest) -> Vec<&ManifestSource> {
    manifest
        .sources
        .iter()
        .filter(|source| source.kind == "path")
        .collect()
}

fn ensure_path_source(directory: &str) -> PrayResult<String> {
    let manifest_path = manifest_path();
    let manifest_text = read_manifest_text(&manifest_path)?;
    let manifest = parse_manifest(&manifest_text)?;
    if let Some(source) = manifest
        .sources
        .iter()
        .find(|source| source.kind == "path" && source.url == directory)
    {
        return Ok(source.name.clone());
    }
    let name = unused_source_name(&manifest, directory)?;
    let statement = format!("source \"{name}\", path: \"{directory}\"");
    transaction::write_file(
        &manifest_path,
        insert_source_statement(&manifest_text, &statement),
    )?;
    Ok(name)
}

fn unused_source_name(manifest: &Manifest, directory: &str) -> PrayResult<String> {
    let taken: Vec<&str> = manifest
        .sources
        .iter()
        .map(|source| source.name.as_str())
        .collect();
    if !taken.contains(&DEFAULT_PATH_SOURCE_NAME) {
        return Ok(DEFAULT_PATH_SOURCE_NAME.to_string());
    }
    let fallback = directory
        .rsplit(['/', '\\'])
        .find(|segment| !segment.is_empty())
        .unwrap_or(DEFAULT_PATH_SOURCE_NAME);
    if taken.contains(&fallback) {
        return Err(PrayError::Manifest(format!(
            "source name {fallback} is already used"
        )));
    }
    Ok(fallback.to_string())
}

fn insert_source_statement(text: &str, statement: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let insertion_index = lines
        .iter()
        .rposition(|line| line.trim_start().starts_with("source "))
        .map(|index| index + 1)
        .or_else(|| {
            lines
                .iter()
                .position(|line| line.trim_start().starts_with("prayfile "))
                .map(|index| index + 1)
        })
        .unwrap_or(0);
    lines.insert(insertion_index, statement.to_string());
    join_manifest_lines(lines)
}

fn insert_into_first_compose(text: &str, statement: &str) -> Option<String> {
    let mut lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let index = lines.iter().position(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("compose ") && trimmed.ends_with(" do")
    })?;
    let indent = lines[index]
        .chars()
        .take_while(|character| character.is_whitespace())
        .count();
    let prefix = " ".repeat(indent + 2);
    lines.insert(index + 1, format!("{prefix}{statement}"));
    Some(join_manifest_lines(lines))
}

fn join_manifest_lines(lines: Vec<String>) -> String {
    let mut output = lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}
