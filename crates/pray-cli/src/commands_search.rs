use crate::project_paths::manifest_path;
use pray_core::manifest::parse_manifest;
use pray_core::registry_search::{search_http_registry, search_local_registry, RegistrySearchHit};
use pray_core::{PrayError, PrayResult};
use std::fs;
use std::path::PathBuf;

pub(crate) fn search_command(
    query: String,
    source: Option<String>,
    root: Option<PathBuf>,
    url: Option<String>,
) -> PrayResult<()> {
    let hits = if let Some(root) = root {
        search_local_registry(&root, &query, true)?
    } else if let Some(url) = url {
        search_http_registry(&url, &query, true)?
    } else {
        search_from_prayfile(&query, source.as_deref())?
    };
    print_hits(&hits);
    Ok(())
}

fn search_from_prayfile(
    query: &str,
    source_name: Option<&str>,
) -> PrayResult<Vec<RegistrySearchHit>> {
    let manifest_text = fs::read_to_string(manifest_path()).map_err(|_| {
        PrayError::Unsupported(
            "search requires a Prayfile, or pass --root PATH / --url URL".to_string(),
        )
    })?;
    let manifest = parse_manifest(&manifest_text)?;
    let source = if let Some(name) = source_name {
        manifest
            .sources
            .iter()
            .find(|source| source.name == name)
            .ok_or_else(|| PrayError::Resolution(format!("unknown source: {name}")))?
    } else {
        manifest
            .sources
            .iter()
            .find(|source| {
                source.kind == "registry"
                    || source.kind == "static index"
                    || source.kind == "pray_ssh"
            })
            .or_else(|| manifest.sources.first())
            .ok_or_else(|| {
                PrayError::Resolution(
                    "Prayfile has no sources; pass --root PATH or --url URL".to_string(),
                )
            })?
    };
    if source.kind == "path" {
        let project_root = manifest_path()
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        return search_local_registry(&project_root.join(&source.url), query, true);
    }
    if source.kind == "registry" || source.kind == "static index" {
        return search_http_registry(&source.url, query, true);
    }
    Err(PrayError::Unsupported(format!(
        "search does not support source kind {}",
        source.kind
    )))
}

fn print_hits(hits: &[RegistrySearchHit]) {
    if hits.is_empty() {
        println!("no packages matched");
        return;
    }
    for hit in hits {
        match &hit.summary {
            Some(summary) => println!("{} — {summary}", hit.name),
            None => println!("{}", hit.name),
        }
    }
}
