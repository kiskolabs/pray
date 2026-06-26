# Pray Distribution Architecture: Federation and P2P

Visual guide to the different distribution models supported by Pray.

## 1. Centralized (Current)

```mermaid
graph TD
    Server[Distribution Point<br/>prayers.kisko.dev]
    Client1[pray install<br/>Client A]
    Client2[pray install<br/>Client B]
    Client3[pray install<br/>Client C]
    Publisher[pray publish<br/>Publisher]
    
    Publisher -->|publish| Server
    Server -->|install| Client1
    Server -->|install| Client2
    Server -->|install| Client3
    
    style Server fill:#4a9eff
    style Publisher fill:#ff6b6b
    style Client1 fill:#51cf66
    style Client2 fill:#51cf66
    style Client3 fill:#51cf66
```

**Characteristics:**
- Single source of truth
- Simple to operate
- Single point of failure
- Requires trust in one operator

## 2. Federation (New Design)

```mermaid
graph TD
    ServerA[Server A<br/>prayers.kisko.dev<br/>upstream]
    ServerB[Server B<br/>backup.example.com<br/>hub]
    ServerC[Server C<br/>community.prayers.org<br/>peer]
    
    Client1[Client 1]
    Client2[Client 2]
    Client3[Client 3]
    
    Publisher[Publisher]
    
    Publisher -->|publish| ServerA
    
    ServerA <-->|S2S sync<br/>bidirectional| ServerB
    ServerB <-->|S2S sync<br/>bidirectional| ServerC
    ServerC -.->|S2S sync<br/>pull only| ServerA
    
    ServerA -->|install| Client1
    ServerB -->|install| Client2
    ServerC -->|install| Client3
    
    style ServerA fill:#4a9eff
    style ServerB fill:#4a9eff
    style ServerC fill:#4a9eff
    style Publisher fill:#ff6b6b
    style Client1 fill:#51cf66
    style Client2 fill:#51cf66
    style Client3 fill:#51cf66
```

**Characteristics:**
- Explicit peer relationships
- Eventual consistency through sync
- Each server validates packages
- Clients query one server
- Federation transparent to clients
- Resilient to single server failure

**Lessons from XMPP:**
- DNS SRV records can enable automatic peer discovery
- STARTTLS for transport security between servers
- Dialback or SASL for server authentication
- Stanza routing is well-defined in RFC 6120
- XML is verbose; JSON is simpler for static metadata
- Federation scales: millions of XMPP servers federate successfully

## 3. Peer-to-peer (Future)

```mermaid
graph TD
    DHT[DHT Network<br/>Package Discovery]
    
    Peer1[Peer 1]
    Peer2[Peer 2]
    Peer3[Peer 3]
    Peer4[Peer 4]
    
    Client1[Client 1]
    Client2[Client 2]
    
    Publisher[Publisher]
    
    Publisher -->|announce| DHT
    DHT -->|discover| Client1
    DHT -->|discover| Client2
    
    Publisher -.->|seed| Peer1
    Peer1 -.->|seed| Peer2
    Peer2 -.->|seed| Peer3
    Peer3 -.->|seed| Peer4
    
    Peer1 -.->|torrent| Client1
    Peer2 -.->|torrent| Client1
    Peer3 -.->|torrent| Client2
    Peer4 -.->|torrent| Client2
    
    style DHT fill:#ff9800
    style Publisher fill:#ff6b6b
    style Peer1 fill:#9c27b0
    style Peer2 fill:#9c27b0
    style Peer3 fill:#9c27b0
    style Peer4 fill:#9c27b0
    style Client1 fill:#51cf66
    style Client2 fill:#51cf66
```

**Characteristics:**
- No central server required
- DHT for discovery
- Torrent-style artifact distribution
- Self-organizing swarms
- Highly resilient
- Slower initial discovery

## 4. Hybrid: Federation + P2P

```mermaid
graph TD
    ServerA[Server A]
    ServerB[Server B]
    DHT[DHT Network]
    
    Peer1[P2P Peer 1]
    Peer2[P2P Peer 2]
    
    Client1[Client 1<br/>uses federation]
    Client2[Client 2<br/>uses P2P]
    
    ServerA <-->|S2S sync| ServerB
    
    ServerA -.->|seed to DHT| DHT
    ServerB -.->|seed to DHT| DHT
    
    DHT -.->|discover| Peer1
    DHT -.->|discover| Peer2
    
    ServerA -->|HTTP| Client1
    
    Peer1 -.->|torrent| Client2
    Peer2 -.->|torrent| Client2
    
    style ServerA fill:#4a9eff
    style ServerB fill:#4a9eff
    style DHT fill:#ff9800
    style Peer1 fill:#9c27b0
    style Peer2 fill:#9c27b0
    style Client1 fill:#51cf66
    style Client2 fill:#51cf66
```

**Characteristics:**
- Best of both worlds
- Federation for reliability
- P2P for scale and resilience
- Clients choose their transport
- Servers can seed to DHT
- Maximum flexibility

## Trust and Verification Flow

All distribution models preserve the same verification chain:

```mermaid
sequenceDiagram
    participant Pub as Publisher
    participant Srv as Server/Network
    participant Cli as Client
    
    Note over Pub: Create package
    Pub->>Pub: Compute tree hash
    Pub->>Pub: Sign with private key
    Pub->>Srv: Publish (package + signature)
    
    Note over Srv: Store or propagate
    Srv->>Srv: Verify signature
    Srv->>Srv: Store artifact hash
    
    Note over Cli: Install package
    Cli->>Srv: Request package
    Srv->>Cli: Return (artifact + metadata)
    Cli->>Cli: Verify artifact hash
    Cli->>Cli: Verify signature
    Cli->>Cli: Check tree hash
    Cli->>Cli: Lock in Prayfile.lock
```

**Security guarantees maintained across all models:**
- Artifact hash verification
- Signature checking
- Tree hash validation
- Provenance tracking
- Lockfile records

## Configuration Examples

### Centralized

```ruby
# Prayfile
source "default", "https://prayers.kisko.dev"

agent "sample/base", "~> 1.4"
```

### Federation (server side)

```toml
# prayers.toml
[server]
host = "0.0.0.0"
port = 7429

[federation]
enabled = true
sync_interval = "1h"

[[federation.peers]]
name = "upstream"
url = "https://prayers.kisko.dev"
trust = "full"
direction = "pull"
```

### Federation (client side)

```ruby
# Prayfile - client doesn't know about federation
source "default", "https://backup.example.com"

agent "sample/base", "~> 1.4"
```

### P2P (future)

```ruby
# Prayfile - client opts into P2P
source "dht", "pray+dht://bootstrap.prayers.network"

agent "sample/base", "~> 1.4"
```

## Comparison Matrix

| Feature | Centralized | Federation | P2P | Hybrid |
|---------|-------------|------------|-----|--------|
| Setup complexity | Low | Medium | Medium | High |
| Operational cost | Medium | High | Low | Medium |
| Resilience | Low | High | Very High | Very High |
| Discovery speed | Fast | Fast | Slow | Fast |
| Bandwidth efficiency | Medium | Medium | High | High |
| Trust model | Single point | Explicit peers | Cryptographic | Both |
| Privacy | Low | Medium | High | Configurable |
| Suitable for | Public/private | Teams, orgs | Public | All |

## XMPP Federation Comparison

**What Pray borrows from XMPP:**
- Server-to-server federation with explicit trust
- Optional DNS-based peer discovery (SRV records)
- Transport security (TLS between servers)
- Authentication mechanisms (can use SASL, mTLS, or API keys)
- Well-defined routing and delivery semantics
- Proven scalability (millions of federated servers)

**What Pray does differently:**
- **JSON instead of XML**: Simpler parsing, smaller payloads, better tooling
- **Static content**: Packages don't change post-publish (unlike dynamic XMPP messages)
- **Pull-based sync**: Servers pull updates periodically (XMPP pushes stanzas in real-time)
- **Content-addressed**: Packages identified by hash, not mutable names
- **Eventual consistency**: Sync happens on schedule, not immediately
- **No presence**: Servers don't maintain session state for peers

**Why not use XMPP directly:**
- XML overhead unnecessary for static package metadata
- Real-time routing complexity not needed for eventual sync
- Package distribution has different trust model than messaging
- Simpler HTTP/REST APIs easier to implement and debug
- XMPP's strengths (real-time, presence, complex routing) don't apply here

**XMPP RFCs for reference:**
- RFC 6120: XMPP Core (server-to-server federation, TLS, SASL)
- RFC 6121: XMPP Instant Messaging (routing, presence)
- RFC 7590: Use of TLS in XMPP (security considerations)

## Implementation Status

- ✅ **Centralized**: Implemented (`pray serve`)
- 🚧 **Federation**: Design complete, implementation planned
- 📋 **P2P**: Design documented, implementation future
- 📋 **Hybrid**: Depends on Federation + P2P

## References

- `SPEC.md` Section 29: Static registry protocol
- `SPEC.md` Section 29.1: Peer-to-peer distribution transport
- `SPEC.md` Section 29.2: Server-to-server federation
- `README.md`: Distribution points
- Issue: `docs/issues/20260626193000_server_to_server_federation_protocol.md`
- Issue: `docs/issues/20260626183000_torrent_seeding_and_collective_dht_distribution.md`
- Issue: `docs/issues/20260626194500_xmpp_dns_discovery_for_federation.md`
- Summary: `docs/p2p-s2s-summary.md`
