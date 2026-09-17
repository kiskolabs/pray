use pray_transport::http::HttpConfig;
use pray_transport::torrent::{TorrentConfig, TorrentTransport};
use pray_transport::types::{ArtifactRef, PeerConfig, SyncDirection, TransportAdapter, TrustLevel};
use std::io::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn torrent_transport_fetches_artifact_using_piece_ranges() {
    let artifact_bytes = b"abcdefghij".to_vec();
    let artifact_length = artifact_bytes.len();
    let server_artifact_bytes = artifact_bytes.clone();
    let piece_size = 4usize;
    let artifact_url = "/artifact.bin".to_string();
    let server_artifact_url = artifact_url.clone();
    let manifest_path = format!("{artifact_url}.praytorrent.json");
    let manifest = TorrentTransport::build_manifest(
        "sample/base".to_string(),
        "1.2.3".to_string(),
        artifact_url.clone(),
        &artifact_bytes,
        piece_size,
        vec![],
        vec![],
    );
    let artifact_hash = manifest.artifact_hash.clone();
    let manifest_json = serde_json::to_vec(&manifest).expect("manifest json");
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("listener address");
    let expected_requests = manifest.piece_ranges().len() + 2;

    let server_task = tokio::spawn(async move {
        for _ in 0..expected_requests {
            let (mut socket, _) = listener.accept().await.expect("accepted connection");
            let mut request = Vec::new();
            let mut buffer = [0u8; 1024];
            loop {
                let read = socket.read(&mut buffer).await.expect("read request");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }

            let request_text = String::from_utf8_lossy(&request);
            let first_line = request_text.lines().next().expect("request line");
            let path = first_line.split_whitespace().nth(1).expect("request path");
            let response = if path == "/v1/distribution.json" {
                http_response(
                    200,
                    "OK",
                    &[("content-type", "application/json")],
                    br#"{"spec":"pray-distribution-config-1","protocols":["torrent"]}"#,
                )
            } else if path == manifest_path {
                http_response(
                    200,
                    "OK",
                    &[("content-type", "application/json")],
                    &manifest_json,
                )
            } else if path == server_artifact_url {
                let range = request_text
                    .lines()
                    .find(|line| line.starts_with("Range: ") || line.starts_with("range: "))
                    .expect("range header");
                let range = range.split_once(':').expect("range header split").1.trim();
                let (start, end) = parse_range(range).expect("parse range");
                let body = server_artifact_bytes[start..=end].to_vec();
                http_response(
                    206,
                    "Partial Content",
                    &[
                        ("content-type", "application/octet-stream"),
                        (
                            "content-range",
                            &format!("bytes {start}-{end}/{artifact_length}"),
                        ),
                    ],
                    &body,
                )
            } else {
                http_response(
                    404,
                    "Not Found",
                    &[("content-type", "text/plain")],
                    b"not found",
                )
            };

            socket
                .write_all(response.as_bytes())
                .await
                .expect("write response");
            socket.shutdown().await.expect("shutdown socket");
        }
    });

    let transport = TorrentTransport::new(TorrentConfig {
        piece_size,
        http: HttpConfig {
            timeout_secs: 5,
            headers: std::collections::HashMap::new(),
            tls_verify: true,
        },
        ..TorrentConfig::default()
    })
    .expect("transport");

    let peer = PeerConfig {
        name: "peer-a".to_string(),
        transport: "torrent".to_string(),
        url: Some(format!("http://{address}")),
        trust: TrustLevel::Full,
        direction: SyncDirection::Pull,
        config: serde_json::json!({}),
    };
    let artifact = ArtifactRef {
        name: "sample/base".to_string(),
        version: "1.2.3".to_string(),
        url: artifact_url,
        hash: artifact_hash,
    };

    let fetched = transport
        .fetch_artifact(&peer, &artifact)
        .await
        .expect("fetch artifact");

    assert_eq!(fetched, artifact_bytes);
    server_task.await.expect("server task");
}

fn http_response(status_code: u16, reason: &str, headers: &[(&str, &str)], body: &[u8]) -> String {
    let mut response = format!(
        "HTTP/1.1 {status_code} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (key, value) in headers {
        response.push_str(&format!("{key}: {value}\r\n"));
    }
    response.push_str("\r\n");
    response.push_str(&String::from_utf8_lossy(body));
    response
}

fn parse_range(header: &str) -> std::result::Result<(usize, usize), std::io::Error> {
    let range = header
        .strip_prefix("bytes=")
        .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidInput, "missing bytes prefix"))?;
    let (start, end) = range
        .split_once('-')
        .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidInput, "missing range dash"))?;
    Ok((
        start
            .parse()
            .map_err(|_| std::io::Error::new(ErrorKind::InvalidInput, "invalid range start"))?,
        end.parse()
            .map_err(|_| std::io::Error::new(ErrorKind::InvalidInput, "invalid range end"))?,
    ))
}
