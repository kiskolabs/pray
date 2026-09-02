use crate::manifest::{Manifest, RenderPolicy};
use crate::paths::{validate_destination_path, validate_project_relative_path};
use crate::{PrayError, PrayResult};
use std::collections::BTreeSet;

pub(crate) fn validate_manifest_semantics(manifest: &Manifest) -> PrayResult<()> {
    reject_duplicate_source_names(manifest)?;
    validate_render_policy(&manifest.render)?;
    for target in &manifest.targets {
        for output in &target.outputs {
            validate_destination_path(output)?;
        }
        for folder in target
            .skills
            .iter()
            .chain(target.commands.iter())
            .chain(target.rules.iter())
        {
            validate_destination_path(folder)?;
        }
    }
    for package in &manifest.packages {
        if let Some(path) = &package.path {
            validate_project_relative_path(path)?;
        }
        if let Some(file) = &package.file {
            validate_destination_path(file)?;
        }
    }
    for local in &manifest.local {
        validate_project_relative_path(&local.path)?;
    }
    Ok(())
}

fn reject_duplicate_source_names(manifest: &Manifest) -> PrayResult<()> {
    let mut seen = BTreeSet::new();
    for source in &manifest.sources {
        if !seen.insert(source.name.clone()) {
            return Err(PrayError::Manifest(format!(
                "duplicate source name: {}",
                source.name
            )));
        }
    }
    Ok(())
}

fn validate_render_policy(policy: &RenderPolicy) -> PrayResult<()> {
    let defaults = RenderPolicy::default();
    if policy.mode != defaults.mode {
        return Err(PrayError::Unsupported(format!(
            "render mode :{} is not implemented",
            policy.mode
        )));
    }
    if policy.conflict != "fail" {
        return Err(PrayError::Unsupported(format!(
            "render conflict :{} is not implemented; only :fail is supported",
            policy.conflict
        )));
    }
    if policy.churn != defaults.churn {
        return Err(PrayError::Unsupported(format!(
            "render churn :{} is not implemented",
            policy.churn
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::manifest::parse_manifest;

    #[test]
    fn rejects_escaping_compose_output() {
        let error = parse_manifest(
            r#"
prayfile "1"
target :tool_a do
  output "../escape.md"
end
"#,
        )
        .expect_err("escape");
        assert!(error.to_string().contains("escapes repository root"));
    }

    #[test]
    fn rejects_duplicate_source_names() {
        let error = parse_manifest(
            r#"
prayfile "1"
source "default", "https://example.test/a"
source "default", "https://example.test/b"
target :tool_a do
  output "INSTRUCTIONS.md"
end
"#,
        )
        .expect_err("duplicate");
        assert!(error.to_string().contains("duplicate source name"));
    }

    #[test]
    fn rejects_unimplemented_conflict_policy() {
        let error = parse_manifest(
            r#"
prayfile "1"
        render conflict: :warn
target :tool_a do
  output "INSTRUCTIONS.md"
end
"#,
        )
        .expect_err("conflict");
        assert!(error.to_string().contains("not implemented"));
    }
}
