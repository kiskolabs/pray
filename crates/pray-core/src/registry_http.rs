use crate::{PrayError, PrayResult};
use std::io::Read;
use std::sync::OnceLock;
use std::time::Duration;

pub(crate) const MAX_HTTP_RESPONSE_BYTES: u64 = 64 * 1024 * 1024;

pub(crate) struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

pub(crate) fn join_url(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

fn http_client() -> PrayResult<&'static reqwest::blocking::Client> {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    if let Some(client) = CLIENT.get() {
        return Ok(client);
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|error| PrayError::Resolution(error.to_string()))?;
    let _ = CLIENT.set(client);
    CLIENT
        .get()
        .ok_or_else(|| PrayError::Resolution("HTTP client unavailable".to_string()))
}

fn ensure_http_url(url: &str) -> PrayResult<()> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err(PrayError::Unsupported(format!(
            "unsupported URL scheme: {url}"
        )))
    }
}

fn read_response_body(response: reqwest::blocking::Response) -> PrayResult<Vec<u8>> {
    let length = response.content_length();
    if length.is_some_and(|value| value > MAX_HTTP_RESPONSE_BYTES) {
        return Err(PrayError::Resolution(format!(
            "HTTP response exceeds {MAX_HTTP_RESPONSE_BYTES} bytes"
        )));
    }
    let mut body = Vec::new();
    let mut limited = response.take(MAX_HTTP_RESPONSE_BYTES.saturating_add(1));
    limited
        .read_to_end(&mut body)
        .map_err(|error| PrayError::Resolution(error.to_string()))?;
    if body.len() as u64 > MAX_HTTP_RESPONSE_BYTES {
        return Err(PrayError::Resolution(format!(
            "HTTP response exceeds {MAX_HTTP_RESPONSE_BYTES} bytes"
        )));
    }
    Ok(body)
}

pub(crate) fn http_get(url: &str) -> PrayResult<Vec<u8>> {
    let response = http_request("GET", url, None, None, &[])?;
    if response.status / 100 != 2 {
        return Err(PrayError::Resolution(format!(
            "GET {url} failed with HTTP {}",
            response.status
        )));
    }
    Ok(response.body)
}

pub(crate) fn http_get_with_headers(url: &str, headers: &[(&str, &str)]) -> PrayResult<(Vec<u8>, u16)> {
    let response = http_request("GET", url, None, None, headers)?;
    Ok((response.body, response.status))
}

pub(crate) fn http_post(url: &str, content_type: &str, body: &[u8]) -> PrayResult<HttpResponse> {
    http_request("POST", url, Some(content_type), Some(body), &[])
}

pub(crate) fn http_put(url: &str, content_type: &str, body: &[u8]) -> PrayResult<HttpResponse> {
    http_request("PUT", url, Some(content_type), Some(body), &[])
}

fn http_request(
    method: &str,
    url: &str,
    content_type: Option<&str>,
    body: Option<&[u8]>,
    headers: &[(&str, &str)],
) -> PrayResult<HttpResponse> {
    ensure_http_url(url)?;
    let client = http_client()?;
    let mut request = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        other => {
            return Err(PrayError::Unsupported(format!(
                "unsupported HTTP method: {other}"
            )))
        }
    };
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    if let Some(content_type) = content_type {
        request = request.header(reqwest::header::CONTENT_TYPE, content_type);
    }
    if let Some(body) = body {
        request = request.body(body.to_vec());
    }
    let response = request
        .send()
        .map_err(|error| PrayError::Resolution(error.to_string()))?;
    let status = response.status().as_u16();
    let body = read_response_body(response)?;
    Ok(HttpResponse { status, body })
}
