use crate::hashing::sha256_prefixed;
use crate::registry_http::{http_get, http_get_with_headers, join_url};
use crate::resource_limits::MAX_TORRENT_ARTIFACT_BYTES;
use crate::{PrayError, PrayResult};
use serde::{Deserialize, Serialize};

const TORRENT_MANIFEST_SPEC: &str = "pray-torrent-v1";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct TorrentManifest {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct TorrentPieceRange {
    start: usize,
    end: usize,
    hash: String,
}

impl TorrentManifest {
    fn validate(&self) -> PrayResult<()> {
        if self.spec != TORRENT_MANIFEST_SPEC {
            return Err(PrayError::Parse {
                kind: "torrent manifest",
                message: format!("unexpected spec: {}", self.spec),
            });
        }
        if self.piece_size == 0 {
            return Err(PrayError::Parse {
                kind: "torrent manifest",
                message: "piece size must be greater than zero".to_string(),
            });
        }
        if self.length > MAX_TORRENT_ARTIFACT_BYTES {
            return Err(PrayError::Integrity(format!(
                "torrent artifact length exceeds {MAX_TORRENT_ARTIFACT_BYTES} bytes"
            )));
        }
        let expected_piece_count = if self.length == 0 {
            0
        } else {
            self.length.div_ceil(self.piece_size)
        };
        if self.pieces.len() != expected_piece_count {
            return Err(PrayError::Parse {
                kind: "torrent manifest",
                message: format!(
                    "expected {} piece hash(es), found {}",
                    expected_piece_count,
                    self.pieces.len()
                ),
            });
        }
        Ok(())
    }

    fn piece_ranges(&self) -> Vec<TorrentPieceRange> {
        self.pieces
            .iter()
            .enumerate()
            .map(|(index, hash)| {
                let start = index * self.piece_size;
                let end = self
                    .length
                    .saturating_sub(1)
                    .min(start + self.piece_size - 1);
                TorrentPieceRange {
                    start,
                    end,
                    hash: hash.clone(),
                }
            })
            .collect()
    }
}

impl TorrentPieceRange {
    fn length(&self) -> usize {
        self.end.saturating_sub(self.start) + 1
    }
}

pub(crate) fn fetch_torrent_manifest(
    source_url: &str,
    artifact_path: &str,
) -> PrayResult<Option<TorrentManifest>> {
    let url = join_url(source_url, &format!("{}.praytorrent.json", artifact_path));
    match http_get(&url) {
        Ok(response) => {
            let manifest: TorrentManifest =
                serde_json::from_slice(&response).map_err(|error| PrayError::Parse {
                    kind: "torrent manifest",
                    message: error.to_string(),
                })?;
            manifest.validate()?;
            Ok(Some(manifest))
        }
        Err(PrayError::Resolution(message) | PrayError::Network(message))
            if message.contains("HTTP 404") =>
        {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn fetch_torrent_artifact(
    source_url: &str,
    artifact_path: &str,
    manifest: &TorrentManifest,
) -> PrayResult<Vec<u8>> {
    let artifact_url = if manifest.artifact_url.starts_with("http://")
        || manifest.artifact_url.starts_with("https://")
    {
        manifest.artifact_url.clone()
    } else {
        join_url(source_url, &manifest.artifact_url)
    };

    let sources = if manifest.sources.is_empty() {
        vec![artifact_url]
    } else {
        manifest
            .sources
            .iter()
            .map(|source| {
                if source.starts_with("http://") || source.starts_with("https://") {
                    source.clone()
                } else {
                    join_url(source_url, source)
                }
            })
            .collect()
    };

    if manifest.length > MAX_TORRENT_ARTIFACT_BYTES {
        return Err(PrayError::Integrity(format!(
            "torrent artifact length exceeds {MAX_TORRENT_ARTIFACT_BYTES} bytes"
        )));
    }
    let mut bytes = vec![0u8; manifest.length];
    for piece in manifest.piece_ranges() {
        let piece_bytes = download_torrent_piece(&sources, &piece)?;
        if sha256_prefixed(&piece_bytes) != piece.hash {
            return Err(PrayError::Integrity(format!(
                "torrent piece hash mismatch for {artifact_path} {}..{}",
                piece.start, piece.end
            )));
        }
        bytes[piece.start..=piece.end].copy_from_slice(&piece_bytes);
    }

    if sha256_prefixed(&bytes) != manifest.artifact_hash {
        return Err(PrayError::Integrity(format!(
            "torrent artifact hash mismatch for {artifact_path}"
        )));
    }

    Ok(bytes)
}

fn download_torrent_piece(sources: &[String], piece: &TorrentPieceRange) -> PrayResult<Vec<u8>> {
    let range_header = format!("bytes={}-{}", piece.start, piece.end);
    for source in sources {
        match http_get_with_headers(source, &[("Range", &range_header)]) {
            Ok((response, _status)) if response.len() == piece.length() => return Ok(response),
            Ok(_) => continue,
            Err(_) => continue,
        }
    }

    Err(PrayError::Resolution(format!(
        "unable to download torrent piece {}-{}",
        piece.start, piece.end
    )))
}
