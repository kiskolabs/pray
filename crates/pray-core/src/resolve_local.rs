use super::ResolvedLocalFile;
use crate::hashing::sha256_prefixed;
use crate::manifest::ManifestLocal;
use crate::resolve_exports::read_text;
use crate::{PrayError, PrayResult};
use std::path::Path;

pub fn missing_local_embed_guidance(path: impl AsRef<str>) -> String {
    let path = path.as_ref();
    format!(
        "Prayfile lists `local \"{path}\"` but the file does not exist. \
         Create the file or remove the entry from Prayfile, then run `pray install`."
    )
}

pub(crate) fn resolve_local_file(
    project_root: &Path,
    declaration: &ManifestLocal,
) -> PrayResult<ResolvedLocalFile> {
    let path = project_root.join(&declaration.path);
    if !path.exists() {
        if declaration.optional {
            return Ok(ResolvedLocalFile {
                path,
                manifest_path: declaration.path.clone(),
                content: String::new(),
                source_checksum: sha256_prefixed(b""),
                position: declaration.position.clone(),
                optional: true,
            });
        }
        return Err(PrayError::Resolution(missing_local_embed_guidance(
            &declaration.path,
        )));
    }
    let content = read_text(&path)?;
    Ok(ResolvedLocalFile {
        source_checksum: sha256_prefixed(content.as_bytes()),
        content,
        path,
        manifest_path: declaration.path.clone(),
        position: declaration.position.clone(),
        optional: declaration.optional,
    })
}
