use crate::manifest::ManifestTarget;
use crate::{PrayError, PrayResult};
use std::path::Path;

const HTML_COMMENT_EXTENSIONS: &[&str] = &["md", "markdown", "html", "htm"];
const BINARY_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "ico", "pdf", "zip", "gz", "tgz", "tar", "wasm", "bin",
    "woff", "woff2", "exe", "dylib", "so",
];

pub fn compose_writes_header(target: &ManifestTarget, output: &Path, project_header: bool) -> bool {
    match target.header {
        Some(value) => value,
        None => project_header && is_agents_markdown(output),
    }
}

pub fn compose_header_text(
    target: &ManifestTarget,
    output: &Path,
    project_header: bool,
) -> Option<String> {
    if !compose_writes_header(target, output, project_header) {
        return None;
    }
    let name = output
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| output.to_string_lossy().into_owned());
    let guidance = if is_agents_markdown(output) {
        format!("Do not edit managed blocks in `{name}` or provisioned files under `.agents/`.")
    } else {
        format!("Do not edit managed blocks in `{name}`.")
    };
    Some(format!(
        "<!-- pray:0 ignore-comments -->\n\n# Agent context\n\n{guidance}\nTo change shared guidance, update `Prayfile` and run `pray install`."
    ))
}

pub fn ensure_html_comment_compose_dest(output: &Path) -> PrayResult<()> {
    let dest = dest_display(output);
    let Some(extension) = extension_lower(output) else {
        return Err(unknown_compose_dest(&dest));
    };
    if HTML_COMMENT_EXTENSIONS.contains(&extension.as_str()) {
        return Ok(());
    }
    if extension == "json" {
        return Err(PrayError::Render(format!(
            "compose cannot write JSON; use file: \"{dest}\" for unmarked bytes"
        )));
    }
    if BINARY_EXTENSIONS.contains(&extension.as_str()) {
        return Err(PrayError::Render(format!(
            "compose cannot write a binary file; use file: \"{dest}\" for unmarked bytes"
        )));
    }
    Err(unknown_compose_dest(&dest))
}

fn unknown_compose_dest(dest: &str) -> PrayError {
    PrayError::Render(format!(
        "compose cannot write this file type; use file: \"{dest}\" for unmarked bytes"
    ))
}

fn is_agents_markdown(output: &Path) -> bool {
    output.file_name().is_some_and(|name| name == "AGENTS.md")
}

fn dest_display(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn extension_lower(path: &Path) -> Option<String> {
    path.extension()
        .map(|extension| extension.to_string_lossy().to_ascii_lowercase())
}
