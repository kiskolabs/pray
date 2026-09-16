use crate::hashing::{checksum_managed_span_content, marker_id, LOCAL_EMBED_PACKAGE};
use crate::lockfile::ManagedSpanRecord;
use crate::manifest::ManifestTarget;
use crate::resolve::{ResolvedLocalFile, ResolvedPackage};
use crate::substitute::substitute_pray_symbols;
use crate::{PrayError, PrayResult};
use std::path::Path;

pub(crate) struct ContentBuilder {
    content: String,
    next_line: usize,
}

impl ContentBuilder {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            content: String::with_capacity(capacity),
            next_line: 1,
        }
    }

    pub(crate) fn next_line_number(&self) -> usize {
        self.next_line
    }

    pub(crate) fn append_line(&mut self, line: &str) {
        self.content.push_str(line);
        self.content.push('\n');
        self.next_line += 1;
    }

    pub(crate) fn append_empty_line(&mut self) {
        self.content.push('\n');
        self.next_line += 1;
    }

    pub(crate) fn append_body(&mut self, body: &str) {
        let trimmed = body.trim_end_matches('\n');
        if trimmed.is_empty() {
            return;
        }
        for line in trimmed.split('\n') {
            self.append_line(line);
        }
    }

    pub(crate) fn finish(mut self) -> String {
        while self.content.ends_with("\n\n") {
            self.content.pop();
        }
        if !self.content.ends_with('\n') {
            self.content.push('\n');
        }
        self.content
    }
}

pub(crate) fn should_inline_export(package: &ResolvedPackage, export_name: &str) -> bool {
    package
        .spec
        .exports
        .get(export_name)
        .is_none_or(|export| matches!(export.kind.as_str(), "fragment" | "file"))
}

pub(crate) fn append_managed_local(
    builder: &mut ContentBuilder,
    managed_spans: &mut Vec<ManagedSpanRecord>,
    local: &ResolvedLocalFile,
    target: &ManifestTarget,
    output: &Path,
    symbols: &std::collections::BTreeMap<String, String>,
) -> PrayResult<()> {
    if local.content.is_empty() && local.optional {
        return Ok(());
    }
    let body = substitute_pray_symbols(&local.content, symbols)?;
    let seed = format!("local:{}:{}", local.manifest_path, target.name);
    append_managed_span(
        builder,
        managed_spans,
        ManagedSpanInput {
            seed: &seed,
            body: &body,
            output,
            package: LOCAL_EMBED_PACKAGE,
            export: &local.manifest_path,
            source_checksum: &local.source_checksum,
        },
    );
    Ok(())
}

pub(crate) fn append_managed_export(
    builder: &mut ContentBuilder,
    managed_spans: &mut Vec<ManagedSpanRecord>,
    package: &ResolvedPackage,
    export: &str,
    target: &ManifestTarget,
    output: &Path,
    symbols: &std::collections::BTreeMap<String, String>,
) -> PrayResult<()> {
    let raw = match package.export_bodies.get(export) {
        Some(text) => text,
        None if package
            .spec
            .exports
            .get(export)
            .is_some_and(|entry| entry.kind == "file") =>
        {
            return Err(PrayError::Integrity(format!(
                "compose cannot write binary export {export}; use file: for unmarked bytes"
            )));
        }
        None => {
            return Err(PrayError::Render(format!(
                "package {} is missing cached export {}",
                package.declaration.name, export
            )));
        }
    };
    let body = substitute_pray_symbols(raw, symbols)?;
    let seed = format!("{}:{}:{}", package.declaration.name, export, target.name);
    append_managed_span(
        builder,
        managed_spans,
        ManagedSpanInput {
            seed: &seed,
            body: &body,
            output,
            package: &package.declaration.name,
            export,
            source_checksum: &package.source_checksum,
        },
    );
    Ok(())
}

struct ManagedSpanInput<'a> {
    seed: &'a str,
    body: &'a str,
    output: &'a Path,
    package: &'a str,
    export: &'a str,
    source_checksum: &'a str,
}

fn append_managed_span(
    builder: &mut ContentBuilder,
    managed_spans: &mut Vec<ManagedSpanRecord>,
    input: ManagedSpanInput<'_>,
) {
    let id = marker_id(input.seed);
    let open_line = builder.next_line_number();
    builder.append_line(&format!("<!-- pray:{id} -->"));
    builder.append_body(input.body);
    let close_line = builder.next_line_number();
    builder.append_line(&format!("<!-- pray:{id} -->"));
    managed_spans.push(ManagedSpanRecord {
        id,
        target: input.output.to_string_lossy().to_string(),
        open_line,
        close_line,
        ideal_checksum: checksum_managed_span_content(input.body),
        package: input.package.to_string(),
        export: input.export.to_string(),
        source_checksum: input.source_checksum.to_string(),
        silenced: false,
    });
    builder.append_empty_line();
}
