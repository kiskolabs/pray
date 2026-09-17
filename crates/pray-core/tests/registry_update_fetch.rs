use pray_core::embed::{resolve_project_with_options, ResolveOptions};
use pray_core::hashing::sha256_prefixed;
use pray_core::package_spec::PackageSpec;
use pray_core::registry::{
    resolve_registry_package_root, RegistryPackageMetadata, RegistryPackageVersion,
};
use pray_core::resolve_context::PackageResolutionContext;
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct FetchCounts {
    index: AtomicUsize,
    metadata: AtomicUsize,
    artifact: AtomicUsize,
    sidecar: AtomicUsize,
    distribution: AtomicUsize,
    other: AtomicUsize,
}

#[derive(Clone)]
struct Packaged {
    name: String,
    metadata_path: String,
    artifact_path: String,
    metadata_json: Vec<u8>,
    artifact: Vec<u8>,
}

#[test]
fn update_shaped_resolve_fetches_metadata_not_index() {
    let packages = vec![packaged("sample/base"), packaged("sample/web")];
    let (source_url, counts, stop) = start_counting_registry(&packages);
    let root = unique_temp("pray-update-fetch");
    write_prayfile(&root, &source_url, &["sample/base", "sample/web"]);
    let options = update_options();

    resolve_project_with_options(&root.join("Prayfile"), &options).expect("first resolve");
    resolve_project_with_options(&root.join("Prayfile"), &options).expect("second resolve");
    stop.store(true, Ordering::SeqCst);
    thread::sleep(Duration::from_millis(30));

    assert_eq!(counts.index.load(Ordering::SeqCst), 0);
    assert_eq!(counts.metadata.load(Ordering::SeqCst), 2);
    assert_eq!(counts.artifact.load(Ordering::SeqCst), 2);
    assert_eq!(counts.sidecar.load(Ordering::SeqCst), 0);
    assert_eq!(counts.distribution.load(Ordering::SeqCst), 1);
    assert_eq!(counts.other.load(Ordering::SeqCst), 0);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn cached_package_reuses_in_process_metadata() {
    let package = packaged("sample/base");
    let (source_url, counts, stop) = start_counting_registry(std::slice::from_ref(&package));
    let root = unique_temp("pray-update-metadata-cache");
    let declaration = pray_core::manifest::ManifestPackage {
        name: package.name.clone(),
        source: Some("default".to_string()),
        constraint: "~> 1.0".to_string(),
        ..pray_core::manifest::ManifestPackage::default()
    };
    let context = PackageResolutionContext::default();
    resolve_registry_package_root(&root, &source_url, &declaration, &context).expect("first");
    resolve_registry_package_root(&root, &source_url, &declaration, &context).expect("cached");
    stop.store(true, Ordering::SeqCst);
    thread::sleep(Duration::from_millis(30));

    assert_eq!(counts.index.load(Ordering::SeqCst), 0);
    assert_eq!(counts.metadata.load(Ordering::SeqCst), 1);
    assert_eq!(counts.artifact.load(Ordering::SeqCst), 1);
    assert_eq!(counts.sidecar.load(Ordering::SeqCst), 0);
    assert_eq!(counts.distribution.load(Ordering::SeqCst), 1);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn http_summary_search_fetches_index_then_one_metadata_per_match() {
    let packages = vec![packaged("sample/base"), packaged("sample/web")];
    let (source_url, counts, stop) = start_counting_registry(&packages);
    let hits = pray_core::registry_search::search_http_registry(&source_url, "sample", true)
        .expect("search");
    stop.store(true, Ordering::SeqCst);
    thread::sleep(Duration::from_millis(30));
    assert_eq!(hits.len(), 2);
    assert_eq!(counts.index.load(Ordering::SeqCst), 1);
    assert_eq!(counts.metadata.load(Ordering::SeqCst), 2);
    assert_eq!(counts.artifact.load(Ordering::SeqCst), 0);
}

#[test]
fn two_registry_origins_fetch_per_origin_not_a_shared_index() {
    let alpha_package = packaged("alpha/base");
    let beta_package = packaged("beta/web");
    let (alpha_url, alpha_counts, alpha_stop) =
        start_counting_registry(std::slice::from_ref(&alpha_package));
    let (beta_url, beta_counts, beta_stop) =
        start_counting_registry(std::slice::from_ref(&beta_package));
    let (unused_url, unused_counts, unused_stop) = start_counting_registry(&[]);
    let root = unique_temp("pray-multi-registry");
    write_named_prayfile(
        &root,
        &[
            ("alpha", alpha_url.as_str()),
            ("beta", beta_url.as_str()),
            ("unused", unused_url.as_str()),
        ],
        &[("alpha/base", "alpha"), ("beta/web", "beta")],
    );
    resolve_project_with_options(&root.join("Prayfile"), &update_options()).expect("resolve");
    alpha_stop.store(true, Ordering::SeqCst);
    beta_stop.store(true, Ordering::SeqCst);
    unused_stop.store(true, Ordering::SeqCst);
    thread::sleep(Duration::from_millis(30));

    assert_eq!(alpha_counts.index.load(Ordering::SeqCst), 0);
    assert_eq!(alpha_counts.metadata.load(Ordering::SeqCst), 1);
    assert_eq!(alpha_counts.artifact.load(Ordering::SeqCst), 1);
    assert_eq!(alpha_counts.distribution.load(Ordering::SeqCst), 1);
    assert_eq!(beta_counts.metadata.load(Ordering::SeqCst), 1);
    assert_eq!(beta_counts.artifact.load(Ordering::SeqCst), 1);
    assert_eq!(beta_counts.distribution.load(Ordering::SeqCst), 1);
    assert_eq!(unused_counts.index.load(Ordering::SeqCst), 0);
    assert_eq!(unused_counts.metadata.load(Ordering::SeqCst), 0);
    assert_eq!(unused_counts.artifact.load(Ordering::SeqCst), 0);
    assert_eq!(unused_counts.distribution.load(Ordering::SeqCst), 0);
    assert_eq!(unused_counts.other.load(Ordering::SeqCst), 0);
    let _ = fs::remove_dir_all(&root);
}

fn update_options() -> ResolveOptions {
    ResolveOptions {
        refresh_source_revisions: true,
        ignore_locked_versions: true,
        ..ResolveOptions::default()
    }
}

fn write_prayfile(root: &std::path::Path, source_url: &str, names: &[&str]) {
    let packages: Vec<(&str, &str)> = names.iter().map(|name| (*name, "default")).collect();
    write_named_prayfile(root, &[("default", source_url)], &packages);
}

fn write_named_prayfile(
    root: &std::path::Path,
    sources: &[(&str, &str)],
    packages: &[(&str, &str)],
) {
    let mut text = String::from("prayfile \"1\"\n");
    for (name, url) in sources {
        text.push_str(&format!("source \"{name}\", \"{url}\"\n"));
    }
    for (package_name, source_name) in packages {
        text.push_str(&format!(
            "pray \"{package_name}\", \"~> 1.0\", source: \"{source_name}\"\n"
        ));
    }
    fs::write(root.join("Prayfile"), text).expect("Prayfile");
}

fn packaged(name: &str) -> Packaged {
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
    Packaged {
        name: name.to_string(),
        metadata_path: format!("/v1/packages/{name}.json"),
        artifact_path: format!("/{artifact_path}"),
        metadata_json: serde_json::to_vec(&metadata).expect("metadata"),
        artifact,
    }
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

fn start_counting_registry(packages: &[Packaged]) -> (String, Arc<FetchCounts>, Arc<AtomicBool>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.set_nonblocking(true).expect("nonblocking");
    let source_url = format!("http://{}", listener.local_addr().expect("addr"));
    let counts = Arc::new(FetchCounts {
        index: AtomicUsize::new(0),
        metadata: AtomicUsize::new(0),
        artifact: AtomicUsize::new(0),
        sidecar: AtomicUsize::new(0),
        distribution: AtomicUsize::new(0),
        other: AtomicUsize::new(0),
    });
    let stop = Arc::new(AtomicBool::new(false));
    let packages = packages.to_vec();
    let thread_counts = counts.clone();
    let thread_stop = stop.clone();
    thread::spawn(move || loop {
        if thread_stop.load(Ordering::SeqCst) {
            break;
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream.set_nonblocking(false).expect("blocking stream");
                serve_registry(&mut stream, &packages, &thread_counts);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("accept: {error}"),
        }
    });
    (source_url, counts, stop)
}

fn serve_registry(stream: &mut TcpStream, packages: &[Packaged], counts: &FetchCounts) {
    let request = read_request(stream);
    let path = request
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .unwrap_or("");
    if path == "/v1/index.json" {
        counts.index.fetch_add(1, Ordering::SeqCst);
        let names: Vec<_> = packages
            .iter()
            .map(|package| package.name.clone())
            .collect();
        let body = serde_json::json!({
            "spec": "prayfile-distribution-1",
            "packages": names
        });
        respond(
            stream,
            "200 OK",
            "application/json",
            &body.to_string().into_bytes(),
        );
        return;
    }
    if path == "/v1/distribution.json" {
        counts.distribution.fetch_add(1, Ordering::SeqCst);
        respond(stream, "404 Not Found", "text/plain", b"");
        return;
    }
    if path.ends_with(".praytorrent.json") {
        counts.sidecar.fetch_add(1, Ordering::SeqCst);
        respond(stream, "404 Not Found", "text/plain", b"");
        return;
    }
    for package in packages {
        if path == package.metadata_path {
            counts.metadata.fetch_add(1, Ordering::SeqCst);
            respond(stream, "200 OK", "application/json", &package.metadata_json);
            return;
        }
        if path == package.artifact_path {
            counts.artifact.fetch_add(1, Ordering::SeqCst);
            respond(
                stream,
                "200 OK",
                "application/octet-stream",
                &package.artifact,
            );
            return;
        }
    }
    counts.other.fetch_add(1, Ordering::SeqCst);
    respond(stream, "404 Not Found", "text/plain", b"");
}

fn read_request(stream: &mut TcpStream) -> String {
    let mut request = Vec::new();
    let mut buffer = [0u8; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                request.extend_from_slice(&buffer[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    String::from_utf8_lossy(&request).into_owned()
}

fn respond(stream: &mut TcpStream, status: &str, content_type: &str, body: &[u8]) {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
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
