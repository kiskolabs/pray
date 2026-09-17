use pray_core::embed::{resolve_project_with_options, ResolveOptions};
use pray_core::resolve::git_source_cache_directory;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const UNUSED_BLOB_BYTES: usize = 64 * 1024;

#[test]
fn update_clones_git_sources_that_have_no_declared_package() {
    let root = unique_temp("pray-unused-git-source");
    let previous_cache = std::env::var("PRAY_CACHE").ok();
    let cache = root.join("cache");
    fs::create_dir_all(&cache).expect("cache");
    std::env::set_var("PRAY_CACHE", &cache);

    let used = init_git_repo(&root.join("used"), b"used catalog\n");
    let unused_blob: Vec<u8> = (0..UNUSED_BLOB_BYTES)
        .map(|index| (index % 251) as u8)
        .collect();
    let unused = init_git_repo(&root.join("unused"), &unused_blob);
    write_path_package(&root);
    fs::write(
        root.join("Prayfile"),
        format!(
            "prayfile \"1\"\n\
             source \"used\", \"git+file://{used}\"\n\
             source \"unused\", \"git+file://{unused}\"\n\
             pray \"bench/pkg\", path: \"packages/pkg\"\n"
        ),
    )
    .expect("Prayfile");

    resolve_project_with_options(
        &root.join("Prayfile"),
        &ResolveOptions {
            refresh_source_revisions: true,
            ignore_locked_versions: true,
            ..ResolveOptions::default()
        },
    )
    .expect("resolve");

    let used_cache = git_source_cache_directory(&root, &format!("file://{used}"));
    let unused_cache = git_source_cache_directory(&root, &format!("file://{unused}"));
    assert!(
        used_cache.join(".git").is_dir(),
        "used git source should be cloned"
    );
    assert!(
        unused_cache.join(".git").is_dir(),
        "unused git source should still be cloned on update"
    );
    let unused_bytes = directory_bytes(&unused_cache);
    assert!(
        unused_bytes >= UNUSED_BLOB_BYTES as u64 / 2,
        "unused catalog clone should keep most of the blob, got {unused_bytes} bytes"
    );
    restore_cache(previous_cache);
    let _ = fs::remove_dir_all(&root);
}

fn restore_cache(previous: Option<String>) {
    match previous {
        Some(value) => std::env::set_var("PRAY_CACHE", value),
        None => std::env::remove_var("PRAY_CACHE"),
    }
}

fn write_path_package(root: &Path) {
    let package = root.join("packages/pkg");
    fs::create_dir_all(&package).expect("package");
    fs::write(
        package.join("bench-pkg.prayspec"),
        "Package::Specification.new do |spec|\n  spec.name = \"bench/pkg\"\n  spec.version = \"1.0.0\"\n  spec.files = [\"bench-pkg.prayspec\"]\nend\n",
    )
    .expect("prayspec");
}

fn init_git_repo(path: &Path, blob: &[u8]) -> String {
    fs::create_dir_all(path).expect("repo");
    fs::write(path.join("catalog.bin"), blob).expect("blob");
    git(path, &["init", "-b", "main"]);
    git(path, &["add", "-A"]);
    git(
        path,
        &[
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "-m",
            "catalog",
        ],
    );
    path.canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .into_owned()
}

fn git(directory: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .current_dir(directory)
        .env("GIT_AUTHOR_NAME", "pray")
        .env("GIT_AUTHOR_EMAIL", "pray@example.com")
        .env("GIT_COMMITTER_NAME", "pray")
        .env("GIT_COMMITTER_EMAIL", "pray@example.com")
        .args(arguments)
        .output()
        .expect("git");
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn directory_bytes(path: &Path) -> u64 {
    let mut total = 0;
    let mut stack = vec![path.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).expect("read") {
            let entry = entry.expect("entry");
            let file_type = entry.file_type().expect("type");
            if file_type.is_dir() {
                stack.push(entry.path());
            } else if file_type.is_file() {
                total += entry.metadata().expect("meta").len();
            }
        }
    }
    total
}

fn unique_temp(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(&path).expect("temp");
    path
}
