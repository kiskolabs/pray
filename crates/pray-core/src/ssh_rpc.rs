use crate::{PrayError, PrayResult};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Read, Write};

pub const SSH_RPC_SPEC: &str = "pray-ssh-rpc-v1";
pub const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RpcRequest {
    pub spec: String,
    pub id: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RpcResponse {
    pub spec: String,
    pub id: String,
    pub status: u16,
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_encoding: Option<String>,
    pub body: Value,
}

impl RpcRequest {
    pub fn new(id: impl Into<String>, method: impl Into<String>, params: Value) -> Self {
        Self {
            spec: SSH_RPC_SPEC.to_string(),
            id: id.into(),
            method: method.into(),
            params,
        }
    }
}

impl RpcResponse {
    pub fn json_ok(id: impl Into<String>, body: Value) -> Self {
        Self {
            spec: SSH_RPC_SPEC.to_string(),
            id: id.into(),
            status: 200,
            content_type: "application/json".to_string(),
            body_encoding: None,
            body,
        }
    }

    pub fn binary_ok(id: impl Into<String>, bytes: &[u8]) -> Self {
        Self {
            spec: SSH_RPC_SPEC.to_string(),
            id: id.into(),
            status: 200,
            content_type: "application/octet-stream".to_string(),
            body_encoding: Some("base64".to_string()),
            body: Value::String(STANDARD.encode(bytes)),
        }
    }

    pub fn error(id: impl Into<String>, status: u16, message: impl Into<String>) -> Self {
        Self {
            spec: SSH_RPC_SPEC.to_string(),
            id: id.into(),
            status,
            content_type: "application/json".to_string(),
            body_encoding: None,
            body: serde_json::json!({ "error": message.into() }),
        }
    }

    pub fn decode_body_bytes(&self) -> PrayResult<Vec<u8>> {
        if self.content_type == "application/octet-stream"
            && self.body_encoding.as_deref() == Some("base64")
        {
            let encoded = self.body.as_str().ok_or_else(|| {
                PrayError::Resolution("rpc binary body must be a base64 string".to_string())
            })?;
            STANDARD.decode(encoded).map_err(|error| {
                PrayError::Resolution(format!("rpc binary body base64 decode failed: {error}"))
            })
        } else if self.body.is_string() {
            Ok(self.body.as_str().unwrap_or_default().as_bytes().to_vec())
        } else {
            serde_json::to_vec(&self.body).map_err(|error| PrayError::Manifest(error.to_string()))
        }
    }

    pub fn decode_json_body<T: for<'de> Deserialize<'de>>(&self) -> PrayResult<T> {
        if self.status / 100 != 2 {
            return Err(PrayError::Resolution(format!(
                "rpc {} failed with status {}",
                self.id, self.status
            )));
        }
        serde_json::from_value(self.body.clone()).map_err(|error| PrayError::Parse {
            kind: "ssh rpc response",
            message: error.to_string(),
        })
    }
}

pub fn write_frame(writer: &mut impl Write, payload: &[u8]) -> PrayResult<()> {
    if payload.len() > MAX_FRAME_BYTES {
        return Err(PrayError::Unsupported(format!(
            "rpc frame exceeds maximum size of {MAX_FRAME_BYTES} bytes"
        )));
    }
    let length = u32::try_from(payload.len())
        .map_err(|_| PrayError::Unsupported("rpc frame length overflow".to_string()))?;
    writer
        .write_all(&length.to_be_bytes())
        .map_err(PrayError::from)?;
    writer.write_all(payload).map_err(PrayError::from)?;
    writer.flush().map_err(PrayError::from)?;
    Ok(())
}

pub fn read_frame(reader: &mut impl Read) -> PrayResult<Vec<u8>> {
    let mut length_bytes = [0u8; 4];
    match reader.read_exact(&mut length_bytes) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Err(PrayError::Resolution("rpc stream closed".to_string()));
        }
        Err(error) => return Err(error.into()),
    }
    let length = u32::from_be_bytes(length_bytes) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(PrayError::Unsupported(format!(
            "rpc frame exceeds maximum size of {MAX_FRAME_BYTES} bytes"
        )));
    }
    let mut payload = vec![0u8; length];
    reader.read_exact(&mut payload).map_err(PrayError::from)?;
    Ok(payload)
}

pub fn call_stdio(
    reader: &mut impl Read,
    writer: &mut impl Write,
    request: &RpcRequest,
) -> PrayResult<RpcResponse> {
    let payload =
        serde_json::to_vec(request).map_err(|error| PrayError::Manifest(error.to_string()))?;
    write_frame(writer, &payload)?;
    let response_bytes = read_frame(reader)?;
    serde_json::from_slice(&response_bytes).map_err(|error| PrayError::Parse {
        kind: "ssh rpc response",
        message: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_round_trip_preserves_payload() {
        let payload = br#"{"spec":"pray-ssh-rpc-v1"}"#;
        let mut buffer = Vec::new();
        write_frame(&mut buffer, payload).expect("write frame");
        let mut cursor = std::io::Cursor::new(buffer);
        let decoded = read_frame(&mut cursor).expect("read frame");
        assert_eq!(decoded, payload);
    }

    #[test]
    fn binary_response_round_trips_base64() {
        let response = RpcResponse::binary_ok("1", b"artifact-bytes");
        let bytes = response.decode_body_bytes().expect("decode bytes");
        assert_eq!(bytes, b"artifact-bytes");
    }
}
