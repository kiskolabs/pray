use pray_core::registry::{ConfessionSubmission, RegistryIndex, RegistryPackageMetadata};
use pray_core::trust::read_registry_trust_settings;
use pray_core::{PrayError, PrayResult};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::thread;

pub fn run_server(root: PathBuf, host: String, port: u16) -> PrayResult<()> {
    let listener = TcpListener::bind((host.as_str(), port))?;
    println!("Serving {} on http://{}:{}", root.display(), host, port);
    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                let root = root.clone();
                thread::spawn(move || {
                    if let Err(error) = handle_connection(root, stream) {
                        eprintln!("serve error: {error}");
                    }
                });
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn handle_connection(root: PathBuf, mut stream: TcpStream) -> PrayResult<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }
    let request_line = request_line.trim_end_matches(['\r', '\n']);
    if request_line.is_empty() {
        return Ok(());
    }
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| PrayError::Resolution("missing HTTP method".to_string()))?;
    let path = parts
        .next()
        .ok_or_else(|| PrayError::Resolution("missing HTTP path".to_string()))?;

    let mut content_length = 0usize;
    loop {
        let mut header_line = String::new();
        reader.read_line(&mut header_line)?;
        let trimmed = header_line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value
                    .trim()
                    .parse::<usize>()
                    .map_err(|error| PrayError::Resolution(error.to_string()))?;
            }
        }
    }

    let mut body = vec![0; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    let response = match (method, strip_query(path)) {
        ("GET", "/") => html_root_response(&root)?,
        ("GET", path) if path.starts_with("/packages/") => html_package_response(&root, path)?,
        ("GET", path) => static_file_response(&root, path)?,
        ("POST", "/v1/confessions") => confession_response(&root, &body)?,
        _ => response_with_status(405, "text/plain", b"method not allowed".to_vec()),
    };

    write_response(
        &mut stream,
        response.status,
        &response.content_type,
        response.body,
    )?;
    Ok(())
}

struct Response {
    status: u16,
    content_type: String,
    body: Vec<u8>,
}

fn html_root_response(root: &Path) -> PrayResult<Response> {
    let index = read_registry_index(root)?;
    let trust = read_registry_trust_settings(root)?;
    let mut list_items = String::new();
    for package in index.packages {
        list_items.push_str(&format!(
            "<li><a href=\"/packages/{path}\">{package}</a></li>",
            path = html_escape(&package)
        ));
    }
    let body = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Pray distribution point</title></head><body><h1>Pray distribution point</h1><p>Spec: {spec}</p><p>Email confirmation: {email}</p><p>Passkeys: {passkeys}</p><p>SSH keys: {ssh_keys}</p><p>SSH-agent signing: {ssh_agent}</p><ul>{packages}</ul></body></html>",
        spec = html_escape(&index.spec),
        email = trust.email_confirmation_label(),
        passkeys = trust.passkeys_label(),
        ssh_keys = trust.ssh_keys_label(),
        ssh_agent = trust.ssh_agent_label(),
        packages = list_items,
    );
    Ok(Response {
        status: 200,
        content_type: "text/html; charset=utf-8".to_string(),
        body: body.into_bytes(),
    })
}

fn html_package_response(root: &Path, path: &str) -> PrayResult<Response> {
    let package_name = path.trim_start_matches("/packages/");
    let metadata_path = root
        .join("v1/packages")
        .join(package_name)
        .with_extension("json");
    let metadata_text = fs::read_to_string(&metadata_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PrayError::Resolution(format!("package metadata not found: {}", package_name))
        } else {
            PrayError::from(error)
        }
    })?;
    let metadata: RegistryPackageMetadata =
        serde_json::from_str(&metadata_text).map_err(|error| PrayError::Parse {
            kind: "registry metadata",
            message: error.to_string(),
        })?;
    let confessions = read_confessions(root, package_name)?;
    let body = render_package_page(package_name, &metadata, &confessions);
    Ok(Response {
        status: 200,
        content_type: "text/html; charset=utf-8".to_string(),
        body: body.into_bytes(),
    })
}

fn render_package_page(
    package_name: &str,
    metadata: &RegistryPackageMetadata,
    confessions: &[ConfessionSubmission],
) -> String {
    let mut versions = String::new();
    for version in &metadata.versions {
        let mut details = String::new();
        if let Some(signer) = version.signer.as_ref() {
            details.push_str(&format!("<div>Signer: {}</div>", html_escape(signer)));
        }
        if let Some(signature) = version.signature.as_ref() {
            details.push_str(&format!("<div>Signature: {}</div>", html_escape(signature)));
        }
        if let Some(published_at) = version.published_at.as_ref() {
            details.push_str(&format!(
                "<div>Published at: {}</div>",
                html_escape(published_at)
            ));
        }
        versions.push_str(&format!(
            "<li><a href=\"/{artifact}\">{version}</a>{details}</li>",
            artifact = html_escape(&version.artifact),
            version = html_escape(&version.version),
            details = details,
        ));
    }
    let accepted = confessions
        .iter()
        .filter(|entry| entry.status == "accepted")
        .count();
    let rejected = confessions
        .iter()
        .filter(|entry| entry.status == "rejected")
        .count();
    let mut confession_items = String::new();
    for confession in confessions {
        confession_items.push_str(&format!(
            "<li><strong>{}</strong> {}{}</li>",
            html_escape(confession.status.as_str()),
            html_escape(confession.version.as_str()),
            confession
                .note
                .as_ref()
                .map(|note| format!(": {}", html_escape(note)))
                .unwrap_or_default()
        ));
    }
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{package}</title></head><body><h1>{package}</h1><p>Accepted: {accepted}</p><p>Rejected: {rejected}</p><h2>Versions</h2><ul>{versions}</ul><h2>Confessions</h2><ul>{confession_items}</ul></body></html>",
        package = html_escape(package_name),
        accepted = accepted,
        rejected = rejected,
        versions = versions,
        confession_items = confession_items,
    )
}

fn confession_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let confession: ConfessionSubmission =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "confession",
            message: error.to_string(),
        })?;
    let confession_path = root.join("v1/confessions.jsonl");
    if let Some(parent) = confession_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&confession_path)?;
    let line = serde_json::to_string(&confession)
        .map_err(|error| PrayError::Manifest(error.to_string()))?;
    writeln!(file, "{line}")?;
    let response_body = serde_json::json!({
        "status": "ok",
        "package": confession.package,
        "version": confession.version,
    })
    .to_string();
    Ok(Response {
        status: 201,
        content_type: "application/json".to_string(),
        body: response_body.into_bytes(),
    })
}

fn static_file_response(root: &Path, request_path: &str) -> PrayResult<Response> {
    let relative = sanitize_request_path(request_path)?;
    let path = root.join(relative);
    if path.is_dir() {
        return Err(PrayError::Resolution(format!(
            "directory requested as file: {}",
            request_path
        )));
    }
    let body = fs::read(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PrayError::Resolution(format!("not found: {}", request_path))
        } else {
            PrayError::from(error)
        }
    })?;
    let content_type = content_type_for_path(&path);
    Ok(Response {
        status: 200,
        content_type,
        body,
    })
}

fn read_registry_index(root: &Path) -> PrayResult<RegistryIndex> {
    let index_path = root.join("v1/index.json");
    let index_text = fs::read_to_string(&index_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PrayError::Resolution("missing registry index".to_string())
        } else {
            PrayError::from(error)
        }
    })?;
    serde_json::from_str(&index_text).map_err(|error| PrayError::Parse {
        kind: "registry index",
        message: error.to_string(),
    })
}

fn read_confessions(root: &Path, package_name: &str) -> PrayResult<Vec<ConfessionSubmission>> {
    let path = root.join("v1/confessions.jsonl");
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(Vec::new());
    };
    let mut confessions = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let confession: ConfessionSubmission =
            serde_json::from_str(line).map_err(|error| PrayError::Parse {
                kind: "confession",
                message: error.to_string(),
            })?;
        if confession.package == package_name {
            confessions.push(confession);
        }
    }
    Ok(confessions)
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: Vec<u8>,
) -> PrayResult<()> {
    let reason = reason_phrase(status);
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(&body)?;
    stream.flush()?;
    Ok(())
}

fn response_with_status(status: u16, content_type: &str, body: Vec<u8>) -> Response {
    Response {
        status,
        content_type: content_type.to_string(),
        body,
    }
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

fn strip_query(path: &str) -> &str {
    path.split_once('?').map(|(path, _)| path).unwrap_or(path)
}

fn sanitize_request_path(path: &str) -> PrayResult<PathBuf> {
    let path = path.trim_start_matches('/');
    let mut relative = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            _ => {
                return Err(PrayError::Resolution(format!(
                    "invalid request path: {path}"
                )))
            }
        }
    }
    Ok(relative)
}

fn content_type_for_path(path: &Path) -> String {
    match path.extension().and_then(|value| value.to_str()) {
        Some("json") => "application/json".to_string(),
        Some("jsonl") => "application/x-ndjson".to_string(),
        Some("md") | Some("txt") => "text/plain; charset=utf-8".to_string(),
        Some("html") => "text/html; charset=utf-8".to_string(),
        Some("praypkg") => "application/octet-stream".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
