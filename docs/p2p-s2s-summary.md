# P2P and S2S Support for Pray: Complete Design

This document summarizes the complete design for decentralized distribution in Pray.

## Overview

Pray now has a complete design for three distribution models:

1. **Centralized** (implemented): Single distribution point
2. **Federated** (designed): Server-to-server sync with explicit trust
3. **P2P** (designed): Torrent/DHT-based distribution

## Key Question Answered

**How will servers ensure consistency in a decentralized network?**

Servers maintain **eventual consistency** through:
- Periodic server-to-server synchronization
- Hash verification at every hop
- Signature validation before acceptance
- Provenance tracking (origin server, sync path, timestamps)
- Conflict detection for same version with different hashes

## S2S Communication

**Yes, there is server-to-server communication**, and it is similar to the publish-install chain:

```
Publish chain (user → server):
  pray publish → Server A → validates, signs, stores

S2S chain (server → server):
  Server A → sync protocol → Server B → validates signatures → stores
                          ↘ Server C → validates signatures → stores

Install chain (server → user):
  Server B → pray install → validates, locks
```

## Manual Setup with Peer Discovery

**Federation requires manual setup** (Phase 1):
- Servers declare peers in `prayers.toml` config
- Explicit trust relationships (full, metadata_only, disabled)
- Pull, push, or bidirectional sync
- Namespace filters for selective sync

**Servers can share peer lists** (Phase 3):
- Optional public peer list in `/.well-known/pray-federation.json`
- Allows topology discovery without automatic peering
- XMPP-inspired DNS SRV records for automatic endpoint discovery
- Domain owner controls federation through DNS

## FIDONet and XMPP Inspiration

**FIDONet model:**
- Store-and-forward message propagation
- Explicit node relationships
- Manual configuration of routes
- Eventual consistency through scheduled sync

**XMPP contributions:**
- Server-to-server federation at scale (millions of servers)
- DNS SRV records for automatic discovery
- STARTTLS and SASL for security
- Well-documented in RFCs (RFC 6120, RFC 6121)
- Proven federation model despite XML verbosity

**What Pray does differently:**
- JSON instead of XML (simpler, smaller, better tooling)
- Pull-based periodic sync (not real-time push)
- Static content (packages don't change post-publish)
- Content-addressed (packages identified by hash)
- No presence or session state between servers
- HTTP/REST API (simpler than XMPP's complex routing)

## Documents Created

### Design Documents
1. **S2S Federation Protocol** (`docs/issues/20260626193000_server_to_server_federation_protocol.md`)
   - Complete protocol specification
   - Federation config format
   - Sync endpoints and flows
   - Trust model and security
   - Implementation phases

2. **XMPP DNS Discovery** (`docs/issues/20260626194500_xmpp_dns_discovery_for_federation.md`)
   - DNS SRV record support
   - Automatic peer discovery
   - DNSSEC validation
   - Phase 3 feature

3. **Distribution Architecture** (`docs/distribution-architecture.md`)
   - Visual comparison of all models
   - Mermaid diagrams for each architecture
   - XMPP federation comparison
   - Configuration examples
   - Security guarantees

4. **GraphQL/Protobuf/gRPC Lessons** (`docs/issues/20260626200000_lessons_from_graphql_protobuf_grpc.md`)
   - Format choices and trade-offs
   - Schema introspection from GraphQL
   - Binary efficiency from Protocol Buffers
   - Streaming from gRPC
   - Phase 2+ optimization strategy

5. **Pluggable Transport Layer** (`docs/issues/20260626202000_pluggable_transport_layer.md`)
   - Transport-agnostic federation protocol
   - Adapters for HTTP, Telegram, Discord, Git, filesystem
   - USB/sneakernet workflows
   - Message platform integration
   - Custom protocol support

### Spec Updates
1. **SPEC.md Section 29.2**: Server-to-server federation
2. **README.md**: Federation description

### Changelog
1. **Changelog** (`docs/changelogs/20260626162420_server_to_server_federation_protocol.md`)

## Implementation Roadmap

### Phase 1: Manual Peer Sync
**Goal**: Basic federation with manual config

- Federation config parsing (`prayers.toml`)
- Sync protocol endpoints
  - `/.well-known/pray-federation.json`
  - `/v1/sync/index`
  - `/v1/sync/package/{name}`
- Pull-only sync
- Metadata-only trust level
- Manual sync triggers (`pray serve sync`)
- Signature and hash validation
- Provenance tracking

**CLI commands:**
```sh
pray serve --federation
pray serve sync --peer upstream
pray serve peers
```

### Phase 2: Automated Sync
**Goal**: Production-ready federation

- Scheduled background sync
- Full trust with artifact mirroring
- Bidirectional sync
- Conflict detection and alerts
- Rate limiting and bandwidth management
- Push notifications for urgent updates (yanks)
- Health checks and peer monitoring

**CLI commands:**
```sh
pray serve status
pray serve peer add/remove
```

### Phase 3: Discovery
**Goal**: Easier federation setup

- Shared peer lists in discovery endpoint
- Federation topology visualization
- Peer reputation signals
- DNS SRV records (XMPP-inspired)
  - `_pray-federation._tcp.prayers.kisko.dev`
  - DNSSEC validation
  - Automatic endpoint discovery
- Optional peer announcement protocol

### Phase 4: P2P Integration
**Goal**: Hybrid distribution

- DHT for package discovery
- Torrent-style artifact seeding
- S2S for reliable distribution
- P2P for scale and resilience
- Clients choose transport

## Security Model

All distribution models preserve the same guarantees:

**Content integrity:**
- Artifact hash verification (SHA-256)
- Tree hash validation
- Content-addressed packages

**Publisher identity:**
- Package signature verification
- SSH signing keys or passkeys
- Publisher metadata in lockfile

**Provenance tracking:**
- Origin server recorded
- Sync path tracked
- First seen timestamp
- Signature validation results

**Attack mitigation:**
- Malicious peer cannot inject invalid packages
- Hash mismatch rejected
- Signature failure rejected
- Conflict detection and manual resolution
- Rate limiting prevents abuse

## Why Not XML?

XMPP uses XML for extensibility and structure, but:
- XML is verbose (3-5x larger than JSON)
- Parsing is slower and more complex
- Tooling ecosystem favors JSON
- Package metadata is static, not dynamic
- No need for XML namespaces or complex schema
- Simpler HTTP/REST APIs easier to implement and debug

**Decision**: Use JSON over HTTP for federation protocol while borrowing XMPP's federation model and DNS discovery patterns.

## Why JSON over Protocol Buffers/gRPC?

GraphQL, Protocol Buffers, and gRPC offer performance benefits but have trade-offs:

**Protocol Buffers/gRPC advantages:**
- 70-90% smaller payloads
- Faster parsing
- Strong schemas and code generation
- HTTP/2 multiplexing and streaming

**Why Pray starts with JSON:**
- Debuggability (curl, browser, standard tools)
- Accessibility (no schema distribution needed)
- Universal support (works everywhere)
- Simpler for `pray serve` operators
- Package metadata is relatively small

**Hybrid approach:**
- **Phase 1**: JSON only
- **Phase 2**: Add optional protobuf via `Accept` header
- **Phase 3**: Add streaming (newline-delimited JSON or HTTP/2)
- **Phase 4**: Optional gRPC service for high-throughput

See `docs/issues/20260626200000_lessons_from_graphql_protobuf_grpc.md` for detailed analysis.

## Comparison with Other Systems

| System | Model | Discovery | Consistency | Format | Use case |
|--------|-------|-----------|-------------|--------|----------|
| FIDONet | Store-forward | Manual config | Eventual | Binary | Messages, files |
| NNTP | Article sync | Config or feed | Eventual | Text | Usenet articles |
| XMPP | Real-time S2S | DNS SRV | Real-time | XML | Instant messaging |
| ActivityPub | Actor-to-actor | WebFinger | Eventual | JSON-LD | Social network |
| Git | Pull/push | Manual remotes | Explicit | Binary | Source control |
| npm mirrors | HTTP mirror | Manual config | Eventual | JSON | Packages |
| BitTorrent | P2P swarm | DHT | Eventual | Binary | Large files |
| **Pray** | **S2S + P2P** | **Manual + DNS** | **Eventual** | **JSON** | **AI packages** |

## Open Questions

See main design doc for detailed discussion:

1. **Authentication**: API keys vs mTLS vs SSH
2. **DNS discovery**: When to add XMPP-style SRV records
3. **Namespace ownership**: Origin server authority model
4. **Yank propagation**: Scheduled vs immediate
5. **Stale peers**: Skip, mark unhealthy, or failover

## Testing Strategy

Following `spec/README.md` guidelines:

**Unit tests:**
- Config parsing
- Sync endpoint responses
- Hash and signature validation
- Provenance tracking

**Integration tests:**
- Two-server federation
- Three-server topology
- Conflict detection
- Sync recovery after failure

**End-to-end tests:**
- Publish on Server A → sync to Server B → install from Server B
- Yank propagation
- DNS SRV discovery
- Peer health monitoring

## References

**Primary design docs:**
- `docs/issues/20260626193000_server_to_server_federation_protocol.md`
- `docs/issues/20260626194500_xmpp_dns_discovery_for_federation.md`
- `docs/issues/20260626183000_torrent_seeding_and_collective_dht_distribution.md`
- `docs/issues/20260626200000_lessons_from_graphql_protobuf_grpc.md`
- `docs/issues/20260626202000_pluggable_transport_layer.md`
- `docs/distribution-architecture.md`

**Specification:**
- `SPEC.md` Section 29: Static registry protocol
- `SPEC.md` Section 29.1: Peer-to-peer distribution transport
- `SPEC.md` Section 29.2: Server-to-server federation
- `README.md`: Distribution points
- `README.md`: pray serve

**RFCs and standards:**
- RFC 6120: XMPP Core (S2S federation, TLS, SASL)
- RFC 6121: XMPP Instant Messaging
- RFC 7590: Use of TLS in XMPP
- RFC 2782: DNS SRV records
- RFC 4033-4035: DNSSEC specifications

**Prior art:**
- FIDONet: Store-and-forward networks
- NNTP: Usenet server-to-server
- XMPP: Instant messaging federation
- ActivityPub: Federated social networks
- Git: Distributed version control
- npm mirrors: Package replication
- BitTorrent: P2P file sharing

## Next Steps

1. Review and discuss open questions
2. Refine Phase 1 scope
3. Begin implementation of federation config parsing
4. Add federation protocol tests
5. Implement sync endpoints
6. Build sync engine with validation
7. Add CLI commands for federation management
8. Document server operator guide
9. Create federation tutorial
10. Plan Phase 2 features
