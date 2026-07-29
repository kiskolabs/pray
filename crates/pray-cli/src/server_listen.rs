use crate::server::{dispatch_http_request, ServeAuth};
use crate::server_http::write_response;
use pray_core::resource_limits::{
    MAX_SERVE_BODY_BYTES, MAX_SERVE_CONCURRENT_CONNECTIONS, MAX_SERVE_HEADER_BYTES,
    SERVE_SOCKET_TIMEOUT_SECS,
};
use pray_core::{PrayError, PrayResult};
use std::io::{BufRead, BufReader, Read};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

static ACTIVE_SERVE_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

struct ServeConnectionGuard;

impl Drop for ServeConnectionGuard {
    fn drop(&mut self) {
        ACTIVE_SERVE_CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
    }
}

pub fn run_server(root: PathBuf, host: String, port: u16, allow_open_push: bool) -> PrayResult<()> {
    let listener = TcpListener::bind((host.as_str(), port))?;
    println!("Serving {} on http://{}:{}", root.display(), host, port);
    let auth = ServeAuth::http(host, allow_open_push);
    let timeout = Duration::from_secs(SERVE_SOCKET_TIMEOUT_SECS);
    for connection in listener.incoming() {
        let stream = connection?;
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));
        let active = ACTIVE_SERVE_CONNECTIONS.fetch_add(1, Ordering::SeqCst);
        if active >= MAX_SERVE_CONCURRENT_CONNECTIONS {
            ACTIVE_SERVE_CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
            let mut rejected = stream;
            let _ = write_response(&mut rejected, 503, "text/plain", b"busy".to_vec());
            continue;
        }
        let root = root.clone();
        let auth = auth.clone();
        thread::spawn(move || {
            let _guard = ServeConnectionGuard;
            if let Err(error) = handle_connection(root, auth, stream) {
                eprintln!("serve error: {error}");
            }
        });
    }
    Ok(())
}

fn handle_connection(root: PathBuf, auth: ServeAuth, mut stream: TcpStream) -> PrayResult<()> {
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
    let mut header_bytes = 0usize;
    let mut authorization = None;
    loop {
        let mut header_line = String::new();
        reader.read_line(&mut header_line)?;
        header_bytes = header_bytes.saturating_add(header_line.len());
        if header_bytes > MAX_SERVE_HEADER_BYTES {
            write_response(
                &mut stream,
                431,
                "text/plain",
                b"headers too large".to_vec(),
            )?;
            return Ok(());
        }
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
            } else if name.eq_ignore_ascii_case("authorization") {
                authorization = Some(value.trim().to_string());
            }
        }
    }
    let mut auth = auth;
    auth.authorization = authorization;

    if content_length > MAX_SERVE_BODY_BYTES {
        write_response(
            &mut stream,
            413,
            "text/plain",
            b"payload too large".to_vec(),
        )?;
        return Ok(());
    }

    let mut body = vec![0; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    let response = dispatch_http_request(&root, &auth, method, path, &body)?;

    write_response(
        &mut stream,
        response.status,
        &response.content_type,
        response.body,
    )?;
    Ok(())
}
