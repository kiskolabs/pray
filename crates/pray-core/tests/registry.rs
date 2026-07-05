use pray_core::derived_metadata::RegistryDerivedMetadata;
use pray_core::hashing::sha256_prefixed;
use pray_core::manifest::ManifestPackage;
use pray_core::registry::{
    resolve_registry_package_root, RegistryPackageMetadata, RegistryPackageVersion,
};
use pray_core::resolve_context::PackageResolutionContext;

use serde::Serialize;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn prefers_torrent_sidecar_when_available() {
    let artifact_bytes = build_artifact_bytes();
    let piece_size = 4;
    let artifact_path = "v1/artifacts/sample/base/1.0.0/package.praypkg";
    let source_url = start_registry_fixture(&artifact_bytes, artifact_path, true, piece_size);
    let project_root = unique_temp_dir("pray-core-torrent-sidecar");
    let declaration = ManifestPackage {
        name: "sample/base".to_string(),
        constraint: "*".to_string(),
        source: Some("default".to_string()),
        exports: vec![],
        targets: vec![],
        features: vec![],
        optional: false,
        path: None,
        git: None,
        tag: None,
        rev: None,
        tarball: None,
        oci: None,
    };

    let resolved_root = resolve_registry_package_root(
        &project_root,
        &source_url,
        &declaration,
        &PackageResolutionContext::default(),
    )
    .expect("torrent sidecar should resolve")
    .root;

    assert!(resolved_root.join("package.prayspec").exists());
    let counts = read_request_counts(&source_url);
    assert_eq!(counts.metadata.load(Ordering::SeqCst), 1);
    assert_eq!(counts.sidecar.load(Ordering::SeqCst), 1);
    assert_eq!(counts.direct_artifact.load(Ordering::SeqCst), 0);
    assert_eq!(
        counts.range_artifact.load(Ordering::SeqCst),
        expected_piece_count(&artifact_bytes, piece_size)
    );
}

#[test]
fn registry_package_version_merges_derived_metadata_without_conflict() {
    let mut existing = RegistryPackageVersion {
        version: "1.0.0".to_string(),
        artifact: "v1/artifacts/sample/base/1.0.0/package.praypkg".to_string(),
        artifact_hash: Some("sha256:existing".to_string()),
        tree_hash: Some("sha256:tree".to_string()),
        yanked: false,
        targets: vec![],
        exports: vec![],
        signer: Some("publisher@example.com".to_string()),
        signer_fingerprint: None,
        published_at: Some("1".to_string()),
        signature: Some("signature".to_string()),
        derived_metadata: None,
    };
    let incoming = RegistryPackageVersion {
        derived_metadata: Some(RegistryDerivedMetadata {
            summary: "shared guidance".to_string(),
            topics: vec!["guidance".to_string()],
            categories: vec!["documentation".to_string()],
            possible_effects: vec!["clarifies usage".to_string()],
            possible_side_effects: vec![],
            embeddings: vec![],
            file_count: Some(2),
            character_count: Some(42),
            token_count: Some(7),
        }),
        ..existing.clone()
    };

    assert!(existing.same_identity(&incoming));
    existing.merge_annotations_from(&incoming);
    assert_eq!(existing.derived_metadata, incoming.derived_metadata);
}

#[test]
fn falls_back_to_direct_artifact_when_sidecar_is_missing() {
    let artifact_bytes = build_artifact_bytes();
    let piece_size = 4;
    let artifact_path = "v1/artifacts/sample/base/1.0.0/package.praypkg";
    let source_url = start_registry_fixture(&artifact_bytes, artifact_path, false, piece_size);
    let project_root = unique_temp_dir("pray-core-torrent-fallback");
    let declaration = ManifestPackage {
        name: "sample/base".to_string(),
        constraint: "*".to_string(),
        source: Some("default".to_string()),
        exports: vec![],
        targets: vec![],
        features: vec![],
        optional: false,
        path: None,
        git: None,
        tag: None,
        rev: None,
        tarball: None,
        oci: None,
    };

    let resolved_root = resolve_registry_package_root(
        &project_root,
        &source_url,
        &declaration,
        &PackageResolutionContext::default(),
    )
    .expect("direct artifact fallback should resolve")
    .root;

    assert!(resolved_root.join("package.prayspec").exists());
    let counts = read_request_counts(&source_url);
    assert_eq!(counts.metadata.load(Ordering::SeqCst), 1);
    assert_eq!(counts.sidecar.load(Ordering::SeqCst), 1);
    assert_eq!(counts.direct_artifact.load(Ordering::SeqCst), 1);
    assert_eq!(counts.range_artifact.load(Ordering::SeqCst), 0);
}

struct RequestCounts {
    metadata: Arc<AtomicUsize>,
    sidecar: Arc<AtomicUsize>,
    direct_artifact: Arc<AtomicUsize>,
    range_artifact: Arc<AtomicUsize>,
}

fn read_request_counts(source_url: &str) -> RequestCounts {
    let counts = request_counts()
        .lock()
        .expect("request count fixture should be initialized")
        .get(source_url)
        .cloned()
        .expect("request counts should exist for fixture");
    RequestCounts {
        metadata: counts.metadata.clone(),
        sidecar: counts.sidecar.clone(),
        direct_artifact: counts.direct_artifact.clone(),
        range_artifact: counts.range_artifact.clone(),
    }
}

fn start_registry_fixture(
    artifact_bytes: &[u8],
    artifact_path: &str,
    include_sidecar: bool,
    piece_size: usize,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind registry fixture");
    let address = listener.local_addr().expect("fixture address");
    let source_url = format!("http://{}", address);
    let counts = Arc::new(FixtureCounts {
        metadata: Arc::new(AtomicUsize::new(0)),
        sidecar: Arc::new(AtomicUsize::new(0)),
        direct_artifact: Arc::new(AtomicUsize::new(0)),
        range_artifact: Arc::new(AtomicUsize::new(0)),
    });
    request_counts()
        .lock()
        .expect("request count fixture should be available")
        .insert(source_url.clone(), counts.clone());

    let metadata = registry_metadata(artifact_path, artifact_bytes);
    let sidecar_manifest =
        include_sidecar.then(|| torrent_manifest(artifact_path, artifact_bytes, piece_size));
    let artifact_bytes = artifact_bytes.to_vec();
    let artifact_path = artifact_path.to_string();
    let counts_for_thread = counts;

    let expected_requests = if include_sidecar {
        2 + expected_piece_count(&artifact_bytes, piece_size)
    } else {
        3
    };

    thread::spawn(move || {
        for _ in 0..expected_requests {
            let (mut stream, _) = listener.accept().expect("accept registry request");
            let request = read_http_request(&mut stream);
            handle_registry_request(
                &mut stream,
                &request,
                &artifact_path,
                &artifact_bytes,
                &metadata,
                sidecar_manifest.as_ref(),
                &counts_for_thread,
            );
        }
    });

    source_url
}

fn handle_registry_request(
    stream: &mut TcpStream,
    request: &str,
    artifact_path: &str,
    artifact_bytes: &[u8],
    metadata: &RegistryPackageMetadata,
    sidecar_manifest: Option<&TorrentManifestFixture>,
    counts: &FixtureCounts,
) {
    let first_line = request.lines().next().expect("request line");
    let path = first_line.split_whitespace().nth(1).expect("request path");
    let range_header = request.lines().find_map(|line| {
        line.strip_prefix("Range: ")
            .map(|value| value.trim().to_string())
    });

    if path == "/v1/packages/sample/base.json" {
        counts.metadata.fetch_add(1, Ordering::SeqCst);
        respond_json(
            stream,
            &serde_json::to_vec(metadata).expect("metadata json"),
        );
        return;
    }

    let sidecar_path = format!("/{}.praytorrent.json", artifact_path);
    if path == sidecar_path {
        counts.sidecar.fetch_add(1, Ordering::SeqCst);
        match sidecar_manifest {
            Some(manifest) => respond_json(
                stream,
                &serde_json::to_vec(manifest).expect("manifest json"),
            ),
            None => respond_not_found(stream),
        }
        return;
    }

    let artifact_request_path = format!("/{}", artifact_path);
    if path == artifact_request_path {
        if let Some(range_header) = range_header {
            counts.range_artifact.fetch_add(1, Ordering::SeqCst);
            let (start, end) = parse_range(&range_header, artifact_bytes.len());
            respond_partial(
                stream,
                &artifact_bytes[start..=end],
                start,
                end,
                artifact_bytes.len(),
            );
        } else {
            counts.direct_artifact.fetch_add(1, Ordering::SeqCst);
            respond_ok(stream, artifact_bytes);
        }
        return;
    }

    respond_not_found(stream);
}

fn respond_json(stream: &mut TcpStream, body: &[u8]) {
    respond_with_headers(
        stream,
        "200 OK",
        &[("Content-Type", "application/json")],
        body,
    );
}

fn respond_ok(stream: &mut TcpStream, body: &[u8]) {
    respond_with_headers(
        stream,
        "200 OK",
        &[("Content-Type", "application/octet-stream")],
        body,
    );
}

fn respond_partial(stream: &mut TcpStream, body: &[u8], start: usize, end: usize, total: usize) {
    let content_range = format!("bytes {}-{}/{}", start, end, total);
    let headers = [
        ("Content-Type", "application/octet-stream"),
        ("Content-Range", content_range.as_str()),
    ];
    respond_with_headers(stream, "206 Partial Content", &headers, body);
}

fn respond_not_found(stream: &mut TcpStream) {
    respond_with_headers(stream, "404 Not Found", &[], b"");
}

fn respond_with_headers(
    stream: &mut TcpStream,
    status: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) {
    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (name, value) in headers {
        response.push_str(name);
        response.push_str(": ");
        response.push_str(value);
        response.push_str("\r\n");
    }
    response.push_str("\r\n");
    stream
        .write_all(response.as_bytes())
        .expect("write fixture response");
    stream.write_all(body).expect("write fixture body");
}

fn read_http_request(stream: &mut TcpStream) -> String {
    let mut request = Vec::new();
    let mut buffer = [0u8; 1024];
    loop {
        let read = stream.read(&mut buffer).expect("read request bytes");
        if read == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..read]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    String::from_utf8(request).expect("valid request text")
}

fn parse_range(range_header: &str, total_length: usize) -> (usize, usize) {
    let range = range_header.strip_prefix("bytes=").expect("range prefix");
    let (start_text, end_text) = range.split_once('-').expect("range separator");
    let start = start_text.parse::<usize>().expect("range start");
    let end = end_text.parse::<usize>().expect("range end");
    assert!(end < total_length, "range end should fit the artifact");
    (start, end)
}

fn registry_metadata(artifact_path: &str, artifact_bytes: &[u8]) -> RegistryPackageMetadata {
    RegistryPackageMetadata {
        name: "sample/base".to_string(),
        versions: vec![RegistryPackageVersion {
            version: "1.0.0".to_string(),
            artifact: artifact_path.to_string(),
            artifact_hash: Some(sha256_prefixed(artifact_bytes)),
            tree_hash: None,
            yanked: false,
            targets: vec![],
            exports: vec![],
            signer: None,
            signer_fingerprint: None,
            published_at: None,
            signature: None,
            derived_metadata: None,
        }],
    }
}

fn torrent_manifest(
    artifact_path: &str,
    artifact_bytes: &[u8],
    piece_size: usize,
) -> TorrentManifestFixture {
    let piece_size = piece_size.max(1);
    let mut pieces = Vec::new();
    let mut start = 0usize;
    while start < artifact_bytes.len() {
        let end = std::cmp::min(start + piece_size, artifact_bytes.len()) - 1;
        pieces.push(sha256_prefixed(&artifact_bytes[start..=end]));
        start = end + 1;
    }

    TorrentManifestFixture {
        spec: "pray-torrent-v1".to_string(),
        name: "sample/base".to_string(),
        version: "1.0.0".to_string(),
        artifact_url: artifact_path.to_string(),
        artifact_hash: sha256_prefixed(artifact_bytes),
        piece_size,
        length: artifact_bytes.len(),
        pieces,
        sources: vec![],
        trackers: vec![],
    }
}

fn build_artifact_bytes() -> Vec<u8> {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        let prayspec = br#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.0.0"
  spec.files = ["package.prayspec"]
end
"#;
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Regular);
        header.set_mode(0o644);
        header.set_mtime(0);
        header.set_uid(0);
        header.set_gid(0);
        header.set_size(prayspec.len() as u64);
        header.set_cksum();
        builder
            .append_data(&mut header, "package.prayspec", &prayspec[..])
            .expect("append prayspec file");
        builder.finish().expect("finish tar archive");
    }

    let mut encoder =
        zstd::stream::write::Encoder::new(Vec::new(), 0).expect("create zstd encoder");
    encoder.write_all(&tar_bytes).expect("compress artifact");
    encoder.finish().expect("finish zstd archive")
}

fn expected_piece_count(artifact_bytes: &[u8], piece_size: usize) -> usize {
    let piece_size = piece_size.max(1);
    (artifact_bytes.len() + piece_size - 1) / piece_size
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{unique}"));
    fs::create_dir_all(&path).expect("create temp test directory");
    path
}

#[derive(Clone)]
struct FixtureCounts {
    metadata: Arc<AtomicUsize>,
    sidecar: Arc<AtomicUsize>,
    direct_artifact: Arc<AtomicUsize>,
    range_artifact: Arc<AtomicUsize>,
}

#[derive(Serialize)]
struct TorrentManifestFixture {
    spec: String,
    name: String,
    version: String,
    artifact_url: String,
    artifact_hash: String,
    piece_size: usize,
    length: usize,
    pieces: Vec<String>,
    #[serde(default)]
    sources: Vec<String>,
    #[serde(default)]
    trackers: Vec<String>,
}

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static REQUEST_COUNTS: OnceLock<Mutex<HashMap<String, Arc<FixtureCounts>>>> = OnceLock::new();

fn request_counts() -> &'static Mutex<HashMap<String, Arc<FixtureCounts>>> {
    REQUEST_COUNTS.get_or_init(|| Mutex::new(HashMap::new()))
}
