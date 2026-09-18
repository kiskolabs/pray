use pray_core::embed::{write_lockfile, LockSource, Lockfile};
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

pub struct CacheEnv(Option<String>);

impl Drop for CacheEnv {
    fn drop(&mut self) {
        match self.0.take() {
            Some(value) => std::env::set_var("PRAY_CACHE", value),
            None => std::env::remove_var("PRAY_CACHE"),
        }
    }
}

pub struct PinnedShallowCache {
    pub root: PathBuf,
    pub origin: PathBuf,
    pub clone_url: String,
    pub pinned: String,
    _cache_env: CacheEnv,
}

impl Drop for PinnedShallowCache {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub fn pin_cache(root: &Path) -> CacheEnv {
    let previous = std::env::var("PRAY_CACHE").ok();
    let cache = root.join("cache");
    fs::create_dir_all(&cache).expect("cache");
    std::env::set_var("PRAY_CACHE", &cache);
    CacheEnv(previous)
}

pub fn unique_temp(prefix: &str) -> PathBuf {
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

pub fn write_prayfile(root: &Path, body: &str) {
    fs::write(root.join("Prayfile"), format!("prayfile \"1\"\n{body}")).expect("Prayfile");
}

pub fn write_lock_git_pin(root: &Path, clone_url: &str, pinned: &str) {
    write_lockfile(
        &root.join("Prayfile.lock"),
        &Lockfile {
            prayfile_lock: "1".into(),
            spec: "0.1".into(),
            generated_by: "pray test".into(),
            manifest_hash:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            source: vec![LockSource {
                name: "used".into(),
                kind: "git".into(),
                url: format!("git+{clone_url}"),
                revision: Some(pinned.to_string()),
                host_key_fingerprint: None,
            }],
            ..Lockfile::default()
        },
    )
    .expect("lockfile");
}

pub fn write_git_catalog(path: &Path, name: &str) -> PathBuf {
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
    fs::create_dir_all(metadata_path.parent().expect("metadata dir")).expect("metadata dir");
    fs::write(
        metadata_path,
        serde_json::to_vec(&metadata).expect("metadata"),
    )
    .expect("metadata file");
    let artifact_file = path.join(&artifact_path);
    fs::create_dir_all(artifact_file.parent().expect("artifact dir")).expect("artifact dir");
    fs::write(artifact_file, artifact).expect("artifact");
    git(path, &["init", "--template=", "-b", "main"]);
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
    path.canonicalize().expect("canonicalize")
}

pub fn pinned_shallow_cache_fixture(strip_origin: bool) -> PinnedShallowCache {
    let root = unique_temp("pray-pinned-git-fetch");
    let cache_env = pin_cache(&root);
    let origin = write_git_catalog(&root.join("origin"), "sample/base");
    let pinned = git_head(&origin);
    git(
        &origin,
        &["config", "uploadpack.allowReachableSHA1InWant", "true"],
    );
    git(&origin, &["config", "uploadpack.allowFilter", "true"]);
    fs::write(origin.join("later.txt"), "newer tip\n").expect("later");
    git(&origin, &["add", "-A"]);
    git(
        &origin,
        &[
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "-m",
            "later",
        ],
    );
    let clone_url = format!("file://{}", origin.display());
    let cache = git_source_cache_directory(&root, &clone_url);
    fs::create_dir_all(cache.parent().expect("cache parent")).expect("cache parent");
    git(
        &root,
        &[
            "clone",
            "--depth",
            "1",
            "--no-local",
            &clone_url,
            cache.to_str().expect("cache path"),
        ],
    );
    assert!(
        !git_succeeds(&cache, &["cat-file", "-e", &pinned]),
        "fixture cache must lack the pinned commit {pinned}; cache HEAD is {}",
        git_head(&cache)
    );
    if strip_origin {
        for name in ["v1", "later.txt"] {
            let path = origin.join(name);
            if path.is_dir() {
                fs::remove_dir_all(&path).expect("strip origin worktree dir");
            } else if path.exists() {
                fs::remove_file(&path).expect("strip origin worktree file");
            }
        }
    }
    write_prayfile(
        &root,
        &format!("source \"used\", \"git+{clone_url}\"\npray \"sample/base\", \"~> 1.0\", source: \"used\"\n"),
    );
    write_lock_git_pin(&root, &clone_url, &pinned);
    PinnedShallowCache {
        root,
        origin,
        clone_url,
        pinned,
        _cache_env: cache_env,
    }
}

pub fn git_head(directory: &Path) -> String {
    let output = Command::new("git")
        .current_dir(directory)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("rev-parse");
    assert!(
        output.status.success(),
        "rev-parse failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let revision = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(!revision.is_empty(), "HEAD revision was empty");
    revision
}

pub fn git_succeeds(directory: &Path, arguments: &[&str]) -> bool {
    Command::new("git")
        .current_dir(directory)
        .args(arguments)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn git(directory: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .current_dir(directory)
        .env("GIT_AUTHOR_NAME", "pray")
        .env("GIT_AUTHOR_EMAIL", "pray@example.com")
        .env("GIT_COMMITTER_NAME", "pray")
        .env("GIT_COMMITTER_EMAIL", "pray@example.com")
        .args(["-c", "protocol.file.allow=always"])
        .args(arguments)
        .output()
        .expect("git");
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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
