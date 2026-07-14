# Server-to-server federation protocol design

## What changed

- Added design document for S2S federation protocol in `docs/issues/20260626193000_server_to_server_federation_protocol.md`
- Added normative section 29.2 to `SPEC.md` for server-to-server federation
- Updated `README.md` to mention federation alongside P2P transport

## Why this matters

Multiple `pray serve` instances need a way to maintain consistency and share package availability across a decentralized network. This design provides a FIDONet/XMPP-inspired federation model where:

- Servers establish explicit peer relationships through configuration
- Servers sync package metadata and artifacts using a pull/push/bidirectional model
- Consistency is eventual through periodic synchronization
- All packages are validated (signatures, hashes) before acceptance
- Provenance tracking maintains origin server information
- Federation is optional and transparent to clients

## Design principles

The federation protocol follows the same trust and verification model as the publish-install chain:

```
Publish chain (user → server):
  pray publish → Server A → validates, signs, stores

S2S chain (server → server):
  Server A → sync protocol → Server B → validates signatures → stores
                          ↘ Server C → validates signatures → stores

Install chain (server → user):
  Server B → pray install → validates, locks
```

## Key features

**Trust model:**
- Manual peer configuration (explicit trust)
- Three trust levels: `full`, `metadata_only`, `disabled`
- Signature verification required
- Hash verification required
- Malicious peers cannot inject invalid packages

**Sync protocol:**
- Discovery endpoint: `/.well-known/pray-federation.json`
- Index sync with timestamp filtering
- Package metadata sync with federation fields
- Standard artifact URLs
- Conflict detection for hash mismatches

**Configuration:**
- Peers declared in `prayers.toml`
- Sync direction: pull, push, or bidirectional
- Namespace filters for selective sync
- Artifact sync: on-demand or mirror-all

**Client transparency:**
- Clients query a single server
- Server may serve from mirror, proxy, or redirect
- Federation topology hidden from clients

## Implementation phases

1. **Phase 1**: Manual peer sync with metadata-only trust
2. **Phase 2**: Automated sync with full mirroring
3. **Phase 3**: Peer discovery and topology maps
4. **Phase 4**: Integration with DHT transport

## Prior art

- **FIDONet**: Explicit node relationships, store-and-forward
- **NNTP**: Server-to-server article propagation
- **XMPP**: Server-to-server federation, DNS discovery, proven at scale
- **ActivityPub**: Federated message delivery
- **Git**: Explicit remote relationships
- **npm mirrors**: Pull-based package replication

## References

- Design doc: `docs/issues/20260626193000_server_to_server_federation_protocol.md`
- Spec section: 29.2 Server-to-server federation
- Related: Section 29.1 Peer-to-peer distribution transport
- Related: Issue on torrent/DHT distribution
