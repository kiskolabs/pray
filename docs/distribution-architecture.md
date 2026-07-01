# Distribution architecture

This note compares the supported ways to move pre-inference input packages while keeping install, verify, and sync behavior deterministic.

## 1. Centralized distribution

A single distribution point publishes packages and serves clients.

```mermaid
graph TD
    Origin[Distribution point]
    Publisher[Publisher]
    Client1[Client A]
    Client2[Client B]
    Client3[Client C]

    Publisher -->|publish| Origin
    Origin -->|install| Client1
    Origin -->|install| Client2
    Origin -->|install| Client3
```

**Characteristics:**
- simplest operational model
- one source of truth for package metadata and archives
- easiest client behavior to reason about
- single point of failure unless mirrored

## 2. Federated mirrors

Multiple distribution points sync with explicit peers.

```mermaid
graph TD
    Primary[Primary distribution point]
    MirrorA[Mirror A]
    MirrorB[Mirror B]
    Publisher[Publisher]
    Client1[Client A]
    Client2[Client B]
    Client3[Client C]

    Publisher -->|publish| Primary
    Primary <-->|sync| MirrorA
    MirrorA <-->|sync| MirrorB

    Primary -->|install| Client1
    MirrorA -->|install| Client2
    MirrorB -->|install| Client3
```

**Characteristics:**
- explicit peer relationships
- eventual consistency through sync
- clients still query one server at a time
- provenance and signature checks remain required at every hop
- mirrors can improve availability without changing client workflow

## 3. Peer-distributed transport

Peers may also share artifacts directly for discovery and resilience.

```mermaid
graph TD
    Network[Peer network]
    Seed1[Seed 1]
    Seed2[Seed 2]
    Seed3[Seed 3]
    Client1[Client A]
    Client2[Client B]
    Publisher[Publisher]

    Publisher -->|announce| Network
    Network -->|discover| Client1
    Network -->|discover| Client2

    Seed1 -.->|seed| Seed2
    Seed2 -.->|seed| Seed3
    Seed3 -.->|seed| Seed1
```

**Characteristics:**
- no single server required for artifact availability
- discovery may be slower than direct HTTP lookup
- integrity checks must stay the same as centralized delivery
- useful when availability matters more than a single authoritative endpoint

## Shared verification flow

All models preserve the same verification chain:

```mermaid
sequenceDiagram
    participant Pub as Publisher
    participant Net as Distribution network
    participant Cli as Client

    Pub->>Pub: Compute hashes
    Pub->>Pub: Sign package
    Pub->>Net: Publish package and metadata
    Net->>Net: Verify signature and integrity
    Net->>Net: Store or sync artifact
    Cli->>Net: Request package
    Net->>Cli: Return metadata and artifact
    Cli->>Cli: Verify artifact hash
    Cli->>Cli: Verify signature
    Cli->>Cli: Record lockfile state
```

## Practical guidance

- keep publish high trust
- keep install deterministic
- keep sync transparent to clients
- keep provenance visible in the lockfile
- prefer the simplest transport that satisfies availability and verification requirements
