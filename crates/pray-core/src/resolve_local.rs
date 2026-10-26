use super::ResolvedLocalFile;
use crate::hashing::{normalize_line_endings, sha256_prefixed};
use crate::manifest::ManifestLocal;
use crate::source_path::DeclaredRead;
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
    let bytes = match crate::source_path::read_declared_file(
        project_root,
        &declaration.path,
        "local file",
    )? {
        DeclaredRead::Missing if declaration.optional => {
            return Ok(ResolvedLocalFile {
                path,
                manifest_path: declaration.path.clone(),
                content: String::new(),
                source_checksum: sha256_prefixed(b""),
                position: declaration.position.clone(),
                optional: true,
            });
        }
        DeclaredRead::Missing => {
            return Err(PrayError::Resolution(missing_local_embed_guidance(
                &declaration.path,
            )));
        }
        DeclaredRead::Bytes(bytes) => bytes,
    };
    let text = String::from_utf8(bytes)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    let content = normalize_line_endings(&text);
    Ok(ResolvedLocalFile {
        source_checksum: sha256_prefixed(content.as_bytes()),
        content,
        path,
        manifest_path: declaration.path.clone(),
        position: declaration.position.clone(),
        optional: declaration.optional,
    })
}
