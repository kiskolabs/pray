//! Shared resource ceilings for untrusted network and archive input.

pub const MAX_HTTP_RESPONSE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_TORRENT_ARTIFACT_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_ARCHIVE_ENTRIES: usize = 10_000;
pub const MAX_ARCHIVE_ENTRY_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_ARCHIVE_TOTAL_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_FEDERATION_PEERS: usize = 64;
pub const MAX_SERVE_BODY_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_SERVE_HEADER_BYTES: usize = 64 * 1024;
pub const MAX_SERVE_CONCURRENT_CONNECTIONS: usize = 32;
pub const SERVE_SOCKET_TIMEOUT_SECS: u64 = 30;
