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

#[test]
fn same_git_url_with_different_subdirs_resolves_both_distributions() {
    let root = unique_temp("pray-git-shared-url-subdir");
    let previous = std::env::var("PRAY_CACHE").ok();
    let cache = root.join("cache");
    fs::create_dir_all(&cache).expect("cache");
    std::env::set_var("PRAY_CACHE", &cache);
    let _cache_env = CacheEnv(previous);

    let repo = write_two_subdir_catalog(&root.join("catalog"));
    fs::write(
        root.join("Prayfile"),
        format!(
            "prayfile \"1\"\n\
             source \"left\", \"git+file://{repo}\", subdir: \"left\"\n\
             source \"right\", \"git+file://{repo}\", subdir: \"right\"\n\
             pray \"sample/one\", \"~> 1.0\", source: \"left\"\n\
             pray \"sample/two\", \"~> 1.0\", source: \"right\"\n\
             pray \"sample/three\", \"~> 1.0\", source: \"left\"\n"
        ),
    )
    .expect("Prayfile");

    let project = resolve_project_with_options(
        &root.join("Prayfile"),
        &ResolveOptions {
            refresh_source_revisions: true,
            ignore_locked_versions: true,
            ..ResolveOptions::default()
        },
    )
    .expect("resolve both subdirs");

    let mut names: Vec<_> = project
        .packages
        .iter()
        .map(|package| package.declaration.name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["sample/one", "sample/three", "sample/two"]);

    let clone_url = format!("file://{repo}");
    let shared = git_source_cache_directory(&root, &clone_url);
    let left =
        pray_core::resolve::git_source_cache_directory_with_subdir(&root, &clone_url, Some("left"));
    let right = pray_core::resolve::git_source_cache_directory_with_subdir(
        &root,
        &clone_url,
        Some("right"),
    );
    assert!(
        shared.join(".git").is_dir(),
        "subdir-only sources should still populate the URL-only cache for trust import-repo"
    );
    assert!(
        left.join(".git").exists(),
        "left subdir should have its own git worktree"
    );
    assert!(
        right.join(".git").exists(),
        "right subdir should have its own git worktree"
    );
    assert_ne!(
        left, right,
        "different subdirs of the same URL must not share a worktree"
    );
    assert_ne!(
        left, shared,
        "a subdir worktree must not reuse the URL-only cache key"
    );
    let shared_git = git_common_dir(&shared);
    assert_eq!(
        git_common_dir(&left),
        shared_git,
        "left subdir should share the URL-only object store"
    );
    assert_eq!(
        git_common_dir(&right),
        shared_git,
        "right subdir should share the URL-only object store"
    );
    assert_eq!(
        pray_core::resolve::git_source_cached_repository(&root, &clone_url).as_deref(),
        Some(shared.as_path()),
        "trust import-repo should find the URL-only cache after a subdir-only clone"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn cached_repository_finds_a_subdir_worktree_when_url_only_cache_is_missing() {
    let root = unique_temp("pray-git-cached-repo-fallback");
    let clone_url = "file://repo-from-prayfile";
    let leftover =
        pray_core::resolve::git_source_cache_directory_with_subdir(&root, clone_url, Some("left"));
    fs::create_dir_all(&leftover).expect("leftover");
    git(&leftover, &["init", "-b", "main"]);
    git(&leftover, &["remote", "add", "origin", clone_url]);
    assert_eq!(
        pray_core::resolve::git_source_cached_repository(&root, clone_url).as_deref(),
        Some(leftover.as_path())
    );
    let _ = fs::remove_dir_all(&root);
}

fn write_two_subdir_catalog(path: &Path) -> String {
    write_distribution(path, "left", "sample/one");
    write_distribution(path, "left", "sample/three");
    write_distribution(path, "right", "sample/two");
    commit_git(path)
}

fn write_distribution(repo: &Path, subdir: &str, name: &str) {
    let root = repo.join(subdir);
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
    let metadata_path = root.join(format!("v1/packages/{name}.json"));
    if let Some(parent) = metadata_path.parent() {
        fs::create_dir_all(parent).expect("metadata dir");
    }
    fs::write(
        metadata_path,
        serde_json::to_vec(&metadata).expect("metadata"),
    )
    .expect("metadata file");
    let artifact_file = root.join(&artifact_path);
    if let Some(parent) = artifact_file.parent() {
        fs::create_dir_all(parent).expect("artifact dir");
    }
    fs::write(artifact_file, artifact).expect("artifact");
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

fn git_common_dir(repository: &Path) -> PathBuf {
    let output = Command::new("git")
        .current_dir(repository)
        .args(["rev-parse", "--git-common-dir"])
        .output()
        .expect("git-common-dir");
    assert!(
        output.status.success(),
        "git rev-parse --git-common-dir failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let reported = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let path = Path::new(&reported);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repository.join(path)
    };
    resolved
        .canonicalize()
        .expect("canonicalize git-common-dir")
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
