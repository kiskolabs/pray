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

const UNUSED_ARTIFACT_BYTES: usize = 512 * 1024;

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
fn git_catalog_with_unused_artifact_still_resolves_the_used_package() {
    let measured = clone_catalog_with_unused_artifact(UNUSED_ARTIFACT_BYTES);
    println!(
        "git_fetch_bytes unused={} clone_bytes={}",
        UNUSED_ARTIFACT_BYTES, measured.cache_bytes
    );
    assert_eq!(measured.package_name, "sample/base");
    assert!(
        measured.cache_bytes < UNUSED_ARTIFACT_BYTES as u64,
        "a blobless clone should not keep an unused {UNUSED_ARTIFACT_BYTES}-byte artifact, got {} bytes",
        measured.cache_bytes
    );
}

#[test]
#[ignore = "scaling guard; run with: cargo test -p pray-core --test git_source_fetch_bytes -- --ignored --nocapture"]
fn git_clone_bytes_stay_small_when_unused_artifact_grows() {
    const SMALL: usize = 256 * 1024;
    const LARGE: usize = 1024 * 1024;
    let small = clone_catalog_with_unused_artifact(SMALL);
    let large = clone_catalog_with_unused_artifact(LARGE);
    let grown = grow_origin_and_refresh(&large, LARGE);
    println!(
        "git_fetch_bytes unused={}->{} clone_bytes={}->{} fetch_delta_bytes={}",
        SMALL,
        LARGE,
        small.cache_bytes,
        large.cache_bytes,
        grown.saturating_sub(large.cache_bytes)
    );
    assert!(
        large.cache_bytes < LARGE as u64 / 2,
        "a blobless clone should not grow with unused artifact size, got {} bytes for {LARGE}",
        large.cache_bytes
    );
    let fetch_delta = grown.saturating_sub(large.cache_bytes);
    assert!(
        fetch_delta < LARGE as u64 / 2,
        "refresh should not fetch an unused {LARGE}-byte blob, got fetch_delta={fetch_delta}"
    );
}

struct CatalogClone {
    cache_bytes: u64,
    package_name: String,
    root: PathBuf,
    catalog: String,
    _cache_env: CacheEnv,
}

impl Drop for CatalogClone {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn update_options() -> ResolveOptions {
    ResolveOptions {
        refresh_source_revisions: true,
        ignore_locked_versions: true,
        ..ResolveOptions::default()
    }
}

fn clone_catalog_with_unused_artifact(unused_bytes: usize) -> CatalogClone {
    let root = unique_temp("pray-git-artifact-catalog");
    let previous = std::env::var("PRAY_CACHE").ok();
    let cache = root.join("cache");
    fs::create_dir_all(&cache).expect("cache");
    std::env::set_var("PRAY_CACHE", &cache);
    let catalog = write_catalog_with_unused_artifact(&root.join("catalog"), unused_bytes);
    fs::write(
        root.join("Prayfile"),
        format!(
            "prayfile \"1\"\n\
             source \"dist\", \"git+file://{catalog}\"\n\
             pray \"sample/base\", \"~> 1.0\", source: \"dist\"\n"
        ),
    )
    .expect("Prayfile");
    let project =
        resolve_project_with_options(&root.join("Prayfile"), &update_options()).expect("resolve");
    let clone_cache = git_source_cache_directory(&root, &format!("file://{catalog}"));
    CatalogClone {
        cache_bytes: directory_bytes(&clone_cache),
        package_name: project.packages[0].declaration.name.clone(),
        root,
        catalog,
        _cache_env: CacheEnv(previous),
    }
}

fn grow_origin_and_refresh(cloned: &CatalogClone, extra_bytes: usize) -> u64 {
    let catalog = PathBuf::from(&cloned.catalog);
    fs::write(
        catalog.join("v1/artifacts/sample/heavy/1.0.0/extra.bin"),
        incompressible(extra_bytes),
    )
    .expect("extra blob");
    git(&catalog, &["add", "-A"]);
    git(
        &catalog,
        &[
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "-m",
            "grow",
        ],
    );
    resolve_project_with_options(&cloned.root.join("Prayfile"), &update_options())
        .expect("refresh");
    directory_bytes(&git_source_cache_directory(
        &cloned.root,
        &format!("file://{}", cloned.catalog),
    ))
}

fn write_catalog_with_unused_artifact(path: &Path, unused_bytes: usize) -> String {
    write_used_package(path, "sample/base");
    let unused = incompressible(unused_bytes);
    let unused_path = path.join("v1/artifacts/sample/heavy/1.0.0/sample-heavy-1.0.0.praypkg");
    if let Some(parent) = unused_path.parent() {
        fs::create_dir_all(parent).expect("unused artifact dir");
    }
    fs::write(unused_path, unused).expect("unused artifact");
    commit_git(path)
}

fn write_used_package(root: &Path, name: &str) {
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

fn incompressible(bytes: usize) -> Vec<u8> {
    let mut out = vec![0u8; bytes];
    let mut state: u32 = 0x9e37_79b9;
    for byte in &mut out {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        *byte = (state >> 24) as u8;
    }
    out
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
    git(path, &["config", "uploadpack.allowFilter", "true"]);
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
        let Ok(entries) = fs::read_dir(&current) else {
            continue;
        };
        for entry in entries {
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
