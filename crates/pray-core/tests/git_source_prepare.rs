use pray_core::embed::{resolve_project_with_options, ResolveOptions};
use pray_core::hashing::sha256_prefixed;
use pray_core::package_spec::PackageSpec;
use pray_core::registry::{RegistryPackageMetadata, RegistryPackageVersion};
use pray_core::resolve::git_source_cache_directory;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct CacheEnv(Option<String>);

impl Drop for CacheEnv {
    fn drop(&mut self) {
        match self.0.take() {
            Some(value) => std::env::set_var("PRAY_CACHE", value),
            None => std::env::remove_var("PRAY_CACHE"),
        }
    }
}

fn update_options() -> ResolveOptions {
    ResolveOptions {
        refresh_source_revisions: true,
        ignore_locked_versions: true,
        ..ResolveOptions::default()
    }
}

#[test]
fn update_does_not_clone_git_sources_that_have_no_declared_package() {
    let root = unique_temp("pray-unused-git-source");
    let _cache_env = pin_cache(&root);
    let used = init_git_repo(&root.join("used"), b"used catalog\n");
    let unused = init_git_repo(&root.join("unused"), b"unused catalog\n");
    write_path_package(&root);
    write_prayfile(
        &root,
        &format!(
            "source \"used\", \"git+file://{used}\"\n\
             source \"unused\", \"git+file://{unused}\"\n\
             pray \"bench/pkg\", path: \"packages/pkg\"\n"
        ),
    );

    resolve_project_with_options(&root.join("Prayfile"), &update_options()).expect("resolve");

    assert!(
        !git_source_cache_directory(&root, &format!("file://{used}"))
            .join(".git")
            .is_dir(),
        "used git source with no package should not be cloned"
    );
    assert!(
        !git_source_cache_directory(&root, &format!("file://{unused}"))
            .join(".git")
            .is_dir(),
        "unused git source should not be cloned"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn update_clones_only_the_git_source_a_package_uses() {
    let root = unique_temp("pray-used-git-source");
    let _cache_env = pin_cache(&root);
    let used = write_git_catalog(&root.join("used"), "sample/base");
    let unused = init_git_repo(&root.join("unused"), b"unused catalog\n");
    write_prayfile(
        &root,
        &format!(
            "source \"used\", \"git+file://{used}\"\n\
             source \"unused\", \"git+file://{unused}\"\n\
             pray \"sample/base\", \"~> 1.0\", source: \"used\"\n"
        ),
    );

    resolve_project_with_options(&root.join("Prayfile"), &update_options()).expect("resolve");

    assert!(
        git_source_cache_directory(&root, &format!("file://{used}"))
            .join(".git")
            .is_dir(),
        "used git source should be cloned"
    );
    assert!(
        !git_source_cache_directory(&root, &format!("file://{unused}"))
            .join(".git")
            .is_dir(),
        "unused git source should not be cloned"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn update_fetches_origin_once_per_used_git_source() {
    let root = unique_temp("pray-git-fetch-once");
    let _cache_env = pin_cache(&root);
    let used = write_git_catalog(&root.join("used"), "sample/base");
    write_prayfile(
        &root,
        &format!(
            "source \"used\", \"git+file://{used}\"\n\
             pray \"sample/base\", \"~> 1.0\", source: \"used\"\n"
        ),
    );
    let options = update_options();
    resolve_project_with_options(&root.join("Prayfile"), &options).expect("cold clone");

    let trace = root.join("git-trace.jsonl");
    std::env::set_var("GIT_TRACE2_EVENT", &trace);
    resolve_project_with_options(&root.join("Prayfile"), &options).expect("warm refresh");
    std::env::remove_var("GIT_TRACE2_EVENT");

    let origin_fetches = count_origin_fetches(&trace);
    assert_eq!(
        origin_fetches, 1,
        "warm update should fetch origin once, got {origin_fetches}"
    );
    let _ = fs::remove_dir_all(&root);
}

fn pin_cache(root: &Path) -> CacheEnv {
    let previous = std::env::var("PRAY_CACHE").ok();
    let cache = root.join("cache");
    fs::create_dir_all(&cache).expect("cache");
    std::env::set_var("PRAY_CACHE", &cache);
    CacheEnv(previous)
}

fn write_prayfile(root: &Path, body: &str) {
    fs::write(root.join("Prayfile"), format!("prayfile \"1\"\n{body}")).expect("Prayfile");
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

fn write_git_catalog(path: &Path, name: &str) -> String {
    fs::create_dir_all(path).expect("catalog");
    let slug = name.replace('/', "-");
    let prayspec = format!(
        "Package::Specification.new do |spec|\n  spec.name = \"{name}\"\n  spec.version = \"1.0.0\"\n  spec.files = [\"package.prayspec\"]\nend\n"
    );
    let prayspec_bytes = prayspec.into_bytes();
    let artifact = pack_praypkg("package.prayspec", &prayspec_bytes);
    let mut files = BTreeMap::new();
    files.insert("package.prayspec".to_string(), prayspec_bytes);
    let tree_hash = PackageSpec::tree_hash_from_file_bytes(&files).expect("tree hash");
    let artifact_path = format!("v1/artifacts/{name}/1.0.0/{slug}-1.0.0.praypkg");
    let metadata = RegistryPackageMetadata {
        name: name.to_string(),
        versions: vec![RegistryPackageVersion {
            version: "1.0.0".to_string(),
            artifact: artifact_path.clone(),
            artifact_hash: Some(sha256_prefixed(&artifact)),
            tree_hash: Some(tree_hash),
            ..RegistryPackageVersion::default()
        }],
    };
    let metadata_path = path.join(format!("v1/packages/{name}.json"));
    if let Some(parent) = metadata_path.parent() {
        fs::create_dir_all(parent).expect("metadata dir");
    }
    fs::write(
        metadata_path,
        serde_json::to_vec(&metadata).expect("metadata"),
    )
    .expect("metadata file");
    let artifact_file = path.join(&artifact_path);
    if let Some(parent) = artifact_file.parent() {
        fs::create_dir_all(parent).expect("artifact dir");
    }
    fs::write(artifact_file, artifact).expect("artifact");
    commit_git(path)
}

fn pack_praypkg(path: &str, contents: &[u8]) -> Vec<u8> {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Regular);
        header.set_mode(0o644);
        header.set_size(contents.len() as u64);
        header.set_cksum();
        builder
            .append_data(&mut header, path, contents)
            .expect("tar entry");
        builder.finish().expect("tar");
    }
    let mut encoder = zstd::stream::write::Encoder::new(Vec::new(), 0).expect("zstd");
    encoder.write_all(&tar_bytes).expect("compress");
    encoder.finish().expect("zstd finish")
}

fn init_git_repo(path: &Path, blob: &[u8]) -> String {
    fs::create_dir_all(path).expect("repo");
    fs::write(path.join("catalog.bin"), blob).expect("blob");
    commit_git(path)
}

fn commit_git(path: &Path) -> String {
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

fn count_origin_fetches(trace: &Path) -> usize {
    let text = fs::read_to_string(trace).unwrap_or_default();
    text.lines()
        .filter(|line| line.contains("\"fetch\"") && line.contains("origin"))
        .count()
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
