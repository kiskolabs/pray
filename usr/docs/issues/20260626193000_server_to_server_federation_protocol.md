# Server-to-server federation protocol for Pray distribution points

## Problem

How do multiple `pray serve` instances maintain consistency and share package availability across a decentralized network?

The current spec supports:
- Static registries (HTTP hosting)
- Local `pray serve` for single-server distribution
- Future P2P transport (BitTorrent/DHT)

Missing: a server-to-server (S2S) federation layer where distribution points can sync with trusted peers, similar to FIDONet-style store-and-forward networks.

## Goals

1. Allow distribution points to form trusted peer networks
2. Enable package metadata and artifact synchronization between servers
3. Maintain eventual consistency across the federation
4. Preserve hash verification, signatures, and provenance
5. Support manual peer configuration (explicit trust model)
6. Allow servers to share peer lists for discovery
7. Keep federation optional—static and standalone servers still work

## Non-goals

- Automatic peer discovery (Phase 1)
- Consensus algorithms or byzantine fault tolerance
- Real-time synchronization
- Centralized coordination

## Design

### Architecture model

The federation follows FIDONet/NNTP/ActivityPub principles:

```
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│   Server A  │◄───────►│   Server B  │◄───────►│   Server C  │
│ (upstream)  │         │   (hub)     │         │ (peer)      │
└─────────────┘         └─────────────┘         └─────────────┘
       │                       │
       │                       │
       ▼                       ▼
  ┌─────────┐           ┌─────────┐
  │ Client  │           │ Client  │
  └─────────┘           └─────────┘
```

**Key properties:**
- Servers establish explicit peer relationships through config
- Each server chooses which peers to trust and sync from
- Clients query one server; that server may proxy or redirect to peers
- Sync protocol propagates package metadata and artifacts
- Each server validates packages before accepting them

### Federation config

Servers declare peers in a federation config file:

**`prayers.toml`** (or `--config` path):

```toml
[server]
host = "0.0.0.0"
port = 7429
root = "./prayers"

[federation]
enabled = true
sync_interval = "1h"
artifact_sync = "on_demand"  # or "mirror_all"

[[federation.peers]]
name = "upstream"
url = "https://prayers.kisko.dev"
trust = "full"  # full | metadata_only | disabled
direction = "pull"  # pull | push | bidirectional

[[federation.peers]]
name = "backup"
url = "https://backup.example.com"
trust = "full"
direction = "bidirectional"

[[federation.peers]]
name = "community"
url = "https://community.prayers.org"
trust = "metadata_only"
direction = "pull"

[federation.filters]
namespaces = ["sample/*", "company/*"]  # only sync these namespaces
exclude = ["sample/deprecated"]
```

### Trust levels

- **`full`**: Accept metadata and artifacts, verify signatures, mirror packages
- **`metadata_only`**: Accept metadata but fetch artifacts from original source
- **`disabled`**: Peer listed but not synced (administrative hold)

### Sync directions

- **`pull`**: This server fetches updates from the peer
- **`push`**: This server sends updates to the peer
- **`bidirectional`**: Both pull and push

### S2S sync protocol

The sync protocol uses JSON over HTTP for simplicity and tooling support. While XMPP demonstrates federation at scale, its XML verbosity is unnecessary for static package metadata exchange.

See `docs/issues/20260626200000_lessons_from_graphql_protobuf_grpc.md` for detailed discussion of format choices, with Phase 2 adding optional Protocol Buffers support and Phase 3 adding streaming capabilities.

#### Discovery endpoint

`GET /.well-known/pray-federation.json`

Response:
```json
{
  "spec": "pray-federation-v1",
  "server": {
    "name": "prayers.kisko.dev",
    "version": "pray-serve/0.1.0",
    "capabilities": ["sync", "artifacts", "confess"]
  },
  "sync": {
    "index_url": "/v1/sync/index",
    "package_url": "/v1/sync/package/{name}",
    "artifact_url": "/v1/artifacts/{name}/{version}/{filename}",
    "since_param": "since"
  },
  "peers": [
    {
      "name": "backup",
      "url": "https://backup.example.com",
      "public": true
    }
  ]
}
```

The `peers` list is optional and allows servers to share their known federation topology for discovery.

#### Index sync

`GET /v1/sync/index?since=<timestamp>`

Response:
```json
{
  "spec": "prayfile-distribution-1",
  "sync_version": 1704067200,
  "packages": [
    {
      "name": "sample/base",
      "updated_at": "2024-01-01T00:00:00Z",
      "url": "/v1/sync/package/sample/base"
    },
    {
      "name": "sample/webapp",
      "updated_at": "2024-01-02T00:00:00Z",
      "url": "/v1/sync/package/sample/webapp"
    }
  ]
}
```

#### Package metadata sync

`GET /v1/sync/package/sample/base?since=<timestamp>`

Response (extended package metadata with federation fields):
```json
{
  "name": "sample/base",
  "versions": [
    {
      "version": "1.4.3",
      "artifact": "/v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg",
      "artifact_hash": "sha256:abc123...",
      "tree_hash": "sha256:def456...",
      "yanked": false,
      "targets": ["generic"],
      "exports": ["base-instructions"],
      "published_at": "2024-01-01T00:00:00Z",
      "publisher": {
        "id": "user@example.com",
        "key_fingerprint": "SHA256:..."
      },
      "signature": {
        "algorithm": "ssh-ed25519",
        "signature": "base64...",
        "public_key": "ssh-ed25519 ..."
      },
      "origin": {
        "server": "prayers.kisko.dev",
        "first_seen": "2024-01-01T00:00:00Z"
      }
    }
  ],
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### Artifact sync

Artifacts are fetched using standard package artifact URLs:

`GET /v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg`

Artifacts must match the `artifact_hash` in metadata.

### Sync process

```rust
// Pseudocode for sync operation

async fn sync_with_peer(peer: &FederationPeer) -> Result<SyncStats> {
    // 1. Fetch peer's index since our last sync
    let index = peer.fetch_index(our_last_sync_time).await?;
    
    // 2. For each package that changed
    for package in index.packages {
        // 3. Fetch package metadata
        let metadata = peer.fetch_package_metadata(&package.name).await?;
        
        // 4. Validate signatures
        for version in metadata.versions {
            validate_signature(&version)?;
        }
        
        // 5. Apply trust policy
        match peer.trust {
            TrustLevel::Full => {
                // Accept metadata and fetch artifacts
                store_metadata(metadata)?;
                if peer.artifact_sync == ArtifactSync::MirrorAll {
                    for version in metadata.versions {
                        fetch_and_store_artifact(&version).await?;
                    }
                }
            }
            TrustLevel::MetadataOnly => {
                // Store metadata but keep origin URLs
                store_metadata_with_origin(metadata, peer.url)?;
            }
            TrustLevel::Disabled => {
                // Skip
            }
        }
    }
    
    // 6. Update last sync timestamp
    update_last_sync_time(peer, index.sync_version)?;
    
    Ok(stats)
}
```

### Consistency model

**Eventual consistency** through periodic sync:

1. Server A publishes a new package
2. Server B syncs from Server A (pulls metadata and artifact)
3. Server C syncs from Server B (propagates through network)
4. All servers eventually have the package

**Conflict resolution:**

- Packages are content-addressed (hash-based)
- Same `name@version` with different hashes = conflict
- Resolution: reject and log warning, require manual intervention
- Yanked packages propagate as metadata updates

**Provenance tracking:**

Each server records:
- Origin server (where it first saw the package)
- Sync path (which peer it received from)
- First seen timestamp
- Signature validation results

### Client behavior

Clients query a single server and don't need to know about federation:

```sh
pray install sample/base --source https://server-b.example.com
```

Server B may:
- Serve from local mirror
- Redirect to peer server (HTTP 302)
- Proxy request to peer server
- Return metadata with origin URL

### Publish-install chain analogy

The S2S sync is indeed similar to the publish-install chain, but operates at the server level:

```
Publish chain (user → server):
  pray publish → Server A → validates, signs, stores

S2S chain (server → server):
  Server A → sync protocol → Server B → validates signatures → stores
                          ↘ Server C → validates signatures → stores

Install chain (server → user):
  Server B → pray install → validates, locks
```

Both chains preserve:
- Hash verification
- Signature checking
- Provenance tracking
- Explicit trust relationships

### Security model

**Trust is explicit:**
- Servers only sync from configured peers
- All packages validated before acceptance
- Signatures verified against publisher keys
- Malicious peer cannot inject invalid packages

**Attack mitigation:**
- Artifact hash mismatches rejected
- Signature failures rejected
- Sync errors logged for audit
- Rate limiting on sync requests
- Optional peer authentication (mTLS, API keys)

### CLI commands

```sh
# Start server with federation
pray serve --federation

# Manual sync trigger
pray serve sync --peer upstream
pray serve sync --all

# Show federation status
pray serve status

# List known servers in federation
pray serve peers

# Add peer
pray serve peer add upstream https://prayers.kisko.dev --trust full

# Remove peer
pray serve peer remove upstream
```

## Implementation phases

### Phase 1: Manual peer sync
- Config-based peer declaration
- Pull-only sync protocol
- Metadata-only trust level
- Manual sync triggers

### Phase 2: Automated sync
- Scheduled background sync
- Full trust with artifact mirroring
- Bidirectional sync
- Conflict detection and alerts

### Phase 3: Discovery
- Shared peer lists
- Federation topology map
- Peer reputation signals
- Optional peer announcement protocol
- XMPP-inspired DNS SRV records for automatic discovery

### Phase 4: Integration with DHT
- DHT for package discovery
- S2S for artifact distribution
- Hybrid model: DHT + federation

### Phase 5: Pluggable Transports
- Transport abstraction layer
- Filesystem transport (USB, shared folders)
- Git transport (repository-based distribution)
- Message platform transports (Telegram, Discord, Slack)
- Custom protocol support (echo.kisko.dev, etc.)
- Plugin system for community transports

## Open questions

1. **Authentication**: How do peers authenticate each other?
   - mTLS with client certificates (XMPP uses STARTTLS + SASL)
   - API keys in config (simpler for manual setup)
   - SSH-based authentication (aligns with package signing)
   - DNS-based trust (XMPP uses SRV records + DNSSEC)

2. **Bandwidth management**: How to prevent sync storms?
   - Rate limiting per peer
   - Configurable sync schedules
   - Bandwidth quotas

3. **Namespace ownership**: Who controls package namespaces?
   - First publisher wins?
   - Origin server authority?
   - Distributed namespace registry?

4. **Yanking propagation**: How quickly should yanks propagate?
   - Immediate sync on yank?
   - Next scheduled sync?
   - Advisory vs. mandatory yanks?

5. **Stale peer handling**: What if a peer goes offline?
   - Skip and retry later
   - Mark peer as unhealthy
   - Failover to alternate sources

## Prior art

- **FIDONet**: Store-and-forward message network with explicit node relationships
- **NNTP (Usenet)**: Server-to-server article propagation with eventual consistency
- **XMPP (Jabber)**: Server-to-server federation with explicit trust, DNS-based discovery, and routing protocols. Well-documented in RFCs (RFC 6120, RFC 6121). Proven at scale despite XML verbosity—the federation model and trust semantics are solid
- **ActivityPub**: Federated social network with peer-to-peer message delivery
- **Git**: Distributed version control with explicit remote relationships
- **npm registry mirrors**: Pull-based replication of package metadata and tarballs

## References

- `SPEC.md` Section 29: Static registry protocol
- `SPEC.md` Section 29.1: Peer-to-peer distribution transport
- `README.md`: Distribution points
- `README.md`: `pray serve`
- Issue: Torrent seeding and collective DHT distribution
