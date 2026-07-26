use pray_core::{PrayError, PrayResult};
use pray_transport::{PeerInfo, TransportError};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) fn load_known_peer_records(root: &Path) -> PrayResult<BTreeMap<String, PeerInfo>> {
    let path = root.join("v1/peers.json");
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(BTreeMap::new());
    };
    let peers: Vec<PeerInfo> = serde_json::from_str(&text).map_err(|error| PrayError::Parse {
        kind: "peer list",
        message: error.to_string(),
    })?;
    let mut records = BTreeMap::new();
    for peer in peers {
        let peer = normalize_peer_info(peer)?;
        records.insert(peer.url.clone(), peer);
    }
    Ok(records)
}

pub(crate) fn write_known_peer_records(root: &Path, peers: &BTreeMap<String, PeerInfo>) -> PrayResult<()> {
    let path = root.join("v1/peers.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let values: Vec<PeerInfo> = peers.values().cloned().collect();
    fs::write(
        path,
        serde_json::to_string_pretty(&values)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    )?;
    Ok(())
}

pub(crate) fn normalize_peer_info(mut peer: PeerInfo) -> PrayResult<PeerInfo> {
    if peer.url.trim().is_empty() {
        return Err(PrayError::Resolution(
            "peer list contains an entry with an empty url".to_string(),
        ));
    }
    if peer.name.trim().is_empty() {
        peer.name = peer.url.clone();
    }
    Ok(peer)
}

pub(crate) fn upsert_known_peer(records: &mut BTreeMap<String, PeerInfo>, peer: PeerInfo) {
    let url = peer.url.clone();
    match records.get_mut(&url) {
        Some(existing) => {
            if (existing.name == existing.url && peer.name != peer.url)
                || (!existing.public && peer.public)
            {
                *existing = peer;
            }
        }
        None => {
            records.insert(url, peer);
        }
    }
}

pub(crate) fn map_transport_error(error: TransportError) -> PrayError {
    match error {
        TransportError::InvalidResponse(message) => PrayError::Parse {
            kind: "federation response",
            message,
        },
        TransportError::Json(error) => PrayError::Parse {
            kind: "federation response",
            message: error.to_string(),
        },
        TransportError::Io(error) => PrayError::Io(error),
        other => PrayError::Resolution(other.to_string()),
    }
}

