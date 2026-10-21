use crate::cli_output::println_stdout;
use crate::project_paths::{load_manifest, workspace_root};
use pray_core::resolve::{
    git_source_cache_directory_with_subdir, git_source_cached_repository,
    quarantine_legacy_git_checkouts,
};
use pray_core::PrayResult;
use std::fs;
use std::path::Path;

pub(crate) fn doctor_command(migrate_git_cache: bool) -> PrayResult<()> {
    let project_root = workspace_root();
    println_stdout("git cache layout");
    println_stdout(format!("project {}", project_root.display()));
    report_project_git_cache(&project_root.join(".pray/cache/git"));
    let manifest = match load_manifest() {
        Ok(manifest) => manifest,
        Err(error) => {
            println_stdout(format!("manifest unavailable: {error}"));
            return Ok(());
        }
    };
    let mut migrated = 0usize;
    for source in &manifest.sources {
        if source.kind != "git" {
            continue;
        }
        let clone_url = source.url.strip_prefix("git+").unwrap_or(&source.url);
        let catalog = git_source_cache_directory_with_subdir(
            &project_root,
            clone_url,
            source.subdir.as_deref(),
        );
        let trusted = git_source_cached_repository(&project_root, clone_url);
        println_stdout(format!("source {}", source.name));
        println_stdout(format!(
            "  url {}",
            crate::cli_output::redact_destination_target(clone_url)
        ));
        println_stdout(format!(
            "  catalog {} {}",
            catalog.display(),
            path_state(&catalog)
        ));
        match &trusted {
            Some(path) => println_stdout(format!("  trust-repo {} ready", path.display())),
            None => println_stdout("  trust-repo missing"),
        }
        if migrate_git_cache {
            migrated += quarantine_legacy_git_checkouts(&project_root, clone_url)?;
        }
    }
    if migrate_git_cache {
        println_stdout(format!("migrated {migrated} legacy project git checkout(s)"));
    }
    Ok(())
}

fn report_project_git_cache(cache_root: &Path) {
    if !cache_root.is_dir() {
        println_stdout("project-git absent");
        return;
    }
    let Ok(entries) = fs::read_dir(cache_root) else {
        println_stdout("project-git unreadable");
        return;
    };
    let mut found = false;
    for entry in entries.flatten() {
        found = true;
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("?");
        let kind = if name.ends_with(".legacy-git") {
            "legacy"
        } else if path.join(".pray-revision").is_file() {
            "catalog-stub"
        } else if path.join(".git").is_dir() || path.join("HEAD").is_file() {
            "full-clone"
        } else {
            "other"
        };
        println_stdout(format!("project-git {name} ({kind})"));
    }
    if !found {
        println_stdout("project-git empty");
    }
}

fn path_state(path: &Path) -> &'static str {
    if path.is_dir() {
        "present"
    } else {
        "absent"
    }
}
