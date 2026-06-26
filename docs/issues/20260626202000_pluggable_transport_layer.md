# Pluggable Transport Layer for Pray Federation

## Overview

Pray's federation protocol should be transport-agnostic. The sync protocol (discovery, index, package metadata) operates at the logical layer, while the transport layer handles actual delivery over various channels.

This enables Pray to work over:
- Standard HTTP/HTTPS
- Custom protocols (echo.kisko.dev)
- Messaging platforms (Telegram, Discord, Slack)
- Email (SMTP/IMAP)
- File systems (shared folders, USB drives)
- Version control (Git repositories)
- Any custom transport

## Design Principles

**Transport independence:**
- Federation protocol defines *what* to sync
- Transport adapter defines *how* to deliver
- Same protocol works over any transport
- No transport-specific logic in core

**Adapter pattern:**
- Each transport implements a standard interface
- Core sync engine calls adapter methods
- Adapter handles transport-specific details
- Easy to add new transports

**Graceful degradation:**
- Transports have different capabilities
- Some support streaming, some don't
- Some support push, some only pull
- Adapter declares capabilities

**Security per transport:**
- Each transport has appropriate security model
- HTTP: TLS + API keys
- Telegram: Bot tokens + private channels
- Discord: Bot tokens + role permissions
- File: OS permissions

## Transport Abstraction Interface

```rust
/// Core trait for transport adapters
trait TransportAdapter {
    /// Transport capabilities
    fn capabilities(&self) -> TransportCapabilities;
    
    /// Fetch discovery information from peer
    async fn fetch_discovery(&self, peer: &PeerConfig) -> Result<FederationInfo>;
    
    /// Fetch index from peer
    async fn fetch_index(&self, peer: &PeerConfig, since: Option<Timestamp>) -> Result<IndexResponse>;
    
    /// Fetch package metadata from peer
    async fn fetch_package(&self, peer: &PeerConfig, name: &str) -> Result<PackageMetadata>;
    
    /// Fetch artifact from peer
    async fn fetch_artifact(&self, peer: &PeerConfig, artifact: &ArtifactRef) -> Result<Vec<u8>>;
    
    /// Send metadata to peer (for push-capable transports)
    async fn push_package(&self, peer: &PeerConfig, metadata: &PackageMetadata) -> Result<()>;
    
    /// Subscribe to real-time updates (for streaming transports)
    async fn subscribe(&self, peer: &PeerConfig) -> Result<UpdateStream>;
}

/// Transport capabilities
struct TransportCapabilities {
    /// Supports pull-based fetching
    pull: bool,
    
    /// Supports push-based sending
    push: bool,
    
    /// Supports streaming updates
    streaming: bool,
    
    /// Supports binary data
    binary: bool,
    
    /// Maximum message size (None = unlimited)
    max_message_size: Option<usize>,
    
    /// Supports partial responses
    partial_responses: bool,
}
```

## Transport Configurations

### HTTP Transport (default)

```toml
[[federation.peers]]
name = "upstream"
transport = "http"
url = "https://prayers.kisko.dev"
trust = "full"
direction = "pull"

[federation.peers.upstream.http]
timeout = "30s"
headers = { "X-API-Key" = "secret123" }
tls_verify = true
```

### Telegram Transport

```toml
[[federation.peers]]
name = "telegram-backup"
transport = "telegram"
trust = "full"
direction = "bidirectional"

[federation.peers.telegram-backup.telegram]
bot_token = "123456:ABC-DEF..."
channel = "@pray_packages"
# Or private channel ID
# channel_id = -1001234567890
poll_interval = "60s"
```

**How it works:**
1. Bot posts package metadata as JSON messages
2. Artifacts posted as file attachments
3. Clients poll for new messages
4. Bot filters by package name in message text

**Message format:**
```
📦 sample/base v1.4.3

{
  "name": "sample/base",
  "version": "1.4.3",
  "artifact_hash": "sha256:abc...",
  "signature": "ssh-ed25519...",
  "timestamp": "2024-01-15T10:30:00Z"
}

[Artifact file attached]
```

### Discord Transport

```toml
[[federation.peers]]
name = "discord-team"
transport = "discord"
trust = "full"
direction = "bidirectional"

[federation.peers.discord-team.discord]
bot_token = "MTk4NjIyNDgzNDcxOTI1MjQ4.Cl2FMQ.ZnCjm1XVW7vRze4b7Cq4se7kKWs"
guild_id = "123456789012345678"
channel_id = "987654321098765432"
# Optional: thread per package
use_threads = true
poll_interval = "30s"
```

**How it works:**
1. Bot posts to dedicated channel or forum
2. One thread per package for organized history
3. Metadata in embeds, artifacts as attachments
4. Clients react with ✅ after successful sync

**Embed format:**
```json
{
  "title": "📦 sample/base v1.4.3",
  "description": "New package published",
  "color": 3447003,
  "fields": [
    {"name": "Version", "value": "1.4.3"},
    {"name": "Hash", "value": "sha256:abc..."},
    {"name": "Publisher", "value": "user@example.com"},
    {"name": "Signed", "value": "✅ ssh-ed25519"}
  ],
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Slack Transport

```toml
[[federation.peers]]
name = "slack-workspace"
transport = "slack"
trust = "full"
direction = "pull"

[federation.peers.slack-workspace.slack]
bot_token = "xoxb-123456789012-1234567890123-abcdefghijklmnopqrstuvwx"
channel = "#pray-packages"
# Or channel ID
# channel_id = "C01234567"
poll_interval = "60s"
```

### Matrix Transport

```toml
[[federation.peers]]
name = "matrix-room"
transport = "matrix"
trust = "full"
direction = "bidirectional"

[federation.peers.matrix-room.matrix]
homeserver = "https://matrix.org"
user_id = "@praybot:matrix.org"
access_token = "syt_..."
room_id = "!abcdef:matrix.org"
poll_interval = "30s"
```

### Email Transport (SMTP/IMAP)

```toml
[[federation.peers]]
name = "email-backup"
transport = "email"
trust = "metadata_only"
direction = "pull"

[federation.peers.email-backup.email]
imap_server = "imap.gmail.com"
imap_port = 993
smtp_server = "smtp.gmail.com"
smtp_port = 587
username = "pray-sync@example.com"
password = "app-specific-password"
mailbox = "INBOX/Pray"
poll_interval = "5m"
```

**How it works:**
1. Each package version becomes an email
2. Subject: `[pray] sample/base v1.4.3`
3. Body: JSON metadata
4. Attachment: artifact file
5. Clients use IMAP IDLE or polling

### Git Transport

```toml
[[federation.peers]]
name = "git-mirror"
transport = "git"
trust = "full"
direction = "pull"

[federation.peers.git-mirror.git]
repository = "git@github.com:company/pray-packages.git"
branch = "main"
path = "packages"
poll_interval = "5m"
# Or use webhooks
webhook = true
```

**Repository layout:**
```
packages/
  sample/
    base/
      1.4.3/
        metadata.json
        artifact.praypkg
        signature.asc
  index.json
```

### File System Transport

```toml
[[federation.peers]]
name = "shared-drive"
transport = "filesystem"
trust = "full"
direction = "bidirectional"

[federation.peers.shared-drive.filesystem]
path = "/mnt/shared/pray-packages"
# Or network path
# path = "\\\\server\\share\\pray-packages"
watch = true  # Use filesystem watcher instead of polling
```

**Directory layout:**
```
/mnt/shared/pray-packages/
  .pray-federation.json
  packages/
    sample/
      base/
        1.4.3/
          metadata.json
          artifact.praypkg
```

### USB/Sneakernet Transport

```toml
[[federation.peers]]
name = "usb-key"
transport = "filesystem"
trust = "full"
direction = "bidirectional"

[federation.peers.usb-key.filesystem]
path = "/media/usb/pray-packages"
auto_mount = true
watch = true
```

**Workflow:**
1. Export packages to USB: `pray export --peer usb-key`
2. Walk USB to air-gapped system
3. Import packages: `pray import --peer usb-key`

### Custom Protocol (echo.kisko.dev)

```toml
[[federation.peers]]
name = "echo-server"
transport = "custom"
trust = "full"
direction = "pull"

[federation.peers.echo-server.custom]
protocol = "echo"
endpoint = "echo.kisko.dev:1234"
timeout = "30s"
# Custom protocol-specific config
buffer_size = 8192
keepalive = true
```

**Custom adapter implementation:**
```rust
struct EchoTransport {
    endpoint: SocketAddr,
    timeout: Duration,
}

impl TransportAdapter for EchoTransport {
    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            pull: true,
            push: true,
            streaming: true,
            binary: true,
            max_message_size: Some(8192),
            partial_responses: false,
        }
    }
    
    async fn fetch_discovery(&self, peer: &PeerConfig) -> Result<FederationInfo> {
        let mut conn = TcpStream::connect(self.endpoint).await?;
        
        // Echo protocol: send command, receive response
        conn.write_all(b"GET /.well-known/pray-federation.json\n").await?;
        
        let response = read_until_delimiter(&mut conn, b'\n').await?;
        let info: FederationInfo = serde_json::from_slice(&response)?;
        
        Ok(info)
    }
    
    // ... implement other methods
}
```

## Transport Adapter Registry

```rust
/// Global registry of available transports
struct TransportRegistry {
    adapters: HashMap<String, Box<dyn TransportAdapterFactory>>,
}

impl TransportRegistry {
    fn new() -> Self {
        let mut registry = Self { adapters: HashMap::new() };
        
        // Register built-in transports
        registry.register("http", Box::new(HttpTransportFactory));
        registry.register("telegram", Box::new(TelegramTransportFactory));
        registry.register("discord", Box::new(DiscordTransportFactory));
        registry.register("filesystem", Box::new(FilesystemTransportFactory));
        registry.register("git", Box::new(GitTransportFactory));
        
        registry
    }
    
    fn register(&mut self, name: &str, factory: Box<dyn TransportAdapterFactory>) {
        self.adapters.insert(name.to_string(), factory);
    }
    
    fn create(&self, config: &PeerConfig) -> Result<Box<dyn TransportAdapter>> {
        let factory = self.adapters.get(&config.transport)
            .ok_or_else(|| Error::UnknownTransport(config.transport.clone()))?;
        
        factory.create(config)
    }
}

/// Factory for creating transport adapters
trait TransportAdapterFactory: Send + Sync {
    fn create(&self, config: &PeerConfig) -> Result<Box<dyn TransportAdapter>>;
}
```

## Core Sync Engine (Transport-Agnostic)

```rust
struct SyncEngine {
    registry: TransportRegistry,
}

impl SyncEngine {
    async fn sync_with_peer(&self, peer: &PeerConfig) -> Result<SyncStats> {
        // 1. Create transport adapter for this peer
        let transport = self.registry.create(peer)?;
        
        // 2. Check capabilities
        let caps = transport.capabilities();
        if !caps.pull && peer.direction.needs_pull() {
            return Err(Error::TransportNotCapable);
        }
        
        // 3. Fetch discovery info
        let info = transport.fetch_discovery(peer).await?;
        
        // 4. Fetch index
        let index = transport.fetch_index(peer, self.last_sync_time(peer)).await?;
        
        // 5. For each package
        for package_summary in index.packages {
            // Fetch metadata (transport-agnostic)
            let metadata = transport.fetch_package(peer, &package_summary.name).await?;
            
            // Validate (always, regardless of transport)
            self.validate_metadata(&metadata)?;
            
            // Store metadata
            self.store_metadata(metadata.clone())?;
            
            // Fetch artifacts if needed
            if peer.trust == TrustLevel::Full {
                for version in metadata.versions {
                    let artifact = transport.fetch_artifact(peer, &version.artifact).await?;
                    
                    // Verify hash (always, regardless of transport)
                    self.verify_artifact_hash(&artifact, &version.artifact_hash)?;
                    
                    self.store_artifact(&version, artifact)?;
                }
            }
        }
        
        Ok(SyncStats { /* ... */ })
    }
}
```

## Message-Based Transport Protocol

For message platforms (Telegram, Discord, Slack), use a command-based protocol:

### Discovery Command
```
/pray discovery
```

Bot responds:
```json
{
  "spec": "pray-federation-v1",
  "server": {
    "name": "telegram-bot",
    "version": "pray-serve/0.1.0",
    "capabilities": ["sync", "artifacts"]
  },
  "transport": "telegram",
  "channel": "@pray_packages"
}
```

### Index Command
```
/pray index since=2024-01-01T00:00:00Z
```

Bot responds with list of packages.

### Package Command
```
/pray package sample/base
```

Bot responds with metadata and artifact.

### Subscribe Command
```
/pray subscribe sample/*
```

Bot sends notifications for matching packages.

## Transport Capability Matrix

| Transport | Pull | Push | Stream | Binary | Max Size | Latency | Security |
|-----------|------|------|--------|--------|----------|---------|----------|
| HTTP | ✅ | ✅ | ✅ | ✅ | Unlimited | Low | TLS |
| Telegram | ✅ | ✅ | ⚠️ | ✅ | 50MB | Medium | Bot token |
| Discord | ✅ | ✅ | ⚠️ | ✅ | 25MB | Medium | Bot token |
| Slack | ✅ | ✅ | ⚠️ | ✅ | 1GB | Medium | OAuth |
| Matrix | ✅ | ✅ | ✅ | ✅ | 50MB | Medium | Access token |
| Email | ✅ | ✅ | ❌ | ✅ | 25MB | High | SMTP auth |
| Git | ✅ | ✅ | ❌ | ✅ | Unlimited | High | SSH/HTTPS |
| Filesystem | ✅ | ✅ | ✅ | ✅ | Unlimited | Low | OS perms |
| Custom | Varies | Varies | Varies | Varies | Varies | Varies | Custom |

## Security Considerations

**Transport-specific authentication:**
- HTTP: TLS + API keys / mTLS
- Telegram: Bot tokens + private channels
- Discord: Bot tokens + role permissions
- Email: SMTP/IMAP credentials
- Git: SSH keys / HTTPS tokens
- Filesystem: OS file permissions

**End-to-end validation:**
- Package signatures verified regardless of transport
- Artifact hashes checked regardless of transport
- Provenance tracked regardless of transport
- Transport only moves bytes, doesn't establish trust

**Untrusted transports:**
Even over completely untrusted transport (public Telegram channel), packages are safe:
1. Metadata includes signature
2. Client verifies signature before accepting
3. Artifact hash checked before installing
4. Malicious transport can't inject fake packages

## Use Cases

### Corporate Air-Gap
```toml
# Export side (internet-connected)
[[federation.peers]]
name = "usb-export"
transport = "filesystem"
path = "/media/usb/pray-export"
direction = "pull"

# Import side (air-gapped)
[[federation.peers]]
name = "usb-import"
transport = "filesystem"
path = "/media/usb/pray-export"
direction = "pull"
```

Workflow:
1. Internet side: `pray serve sync --peer upstream` → downloads to USB
2. Walk USB across air gap
3. Air-gap side: `pray install` → reads from USB

### Remote Office with Poor Internet
```toml
# Use Telegram for small packages
[[federation.peers]]
name = "telegram-fallback"
transport = "telegram"
trust = "full"

[federation.peers.telegram-fallback.telegram]
bot_token = "..."
channel = "@company_pray"
```

When HTTP is slow/unreliable, packages sync via Telegram bot.

### Dev Team Discord
```toml
[[federation.peers]]
name = "team-discord"
transport = "discord"
trust = "full"

[federation.peers.team-discord.discord]
bot_token = "..."
channel_id = "..."
use_threads = true
```

Packages posted to Discord channel, team gets notified, can review before installing.

### Offline Sneakernet
```toml
[[federation.peers]]
name = "usb-key"
transport = "filesystem"
path = "/media/usb/pray"
watch = false
```

Workflow:
1. `pray export --output /media/usb/pray`
2. Physically carry USB
3. `pray import --input /media/usb/pray`

### Hybrid Multi-Transport
```toml
# Primary: HTTP
[[federation.peers]]
name = "primary"
transport = "http"
url = "https://prayers.kisko.dev"
priority = 1

# Fallback: Telegram
[[federation.peers]]
name = "fallback"
transport = "telegram"
channel = "@pray_backup"
priority = 2

# Offline: USB
[[federation.peers]]
name = "offline"
transport = "filesystem"
path = "/media/usb/pray"
priority = 3
```

Client tries transports in priority order.

## Implementation Phases

### Phase 1: HTTP Only
- Single built-in HTTP transport
- Foundation for transport abstraction
- Core sync engine transport-agnostic

### Phase 2: Filesystem Transport
- Add filesystem adapter
- Enable USB/sneakernet workflows
- Test transport abstraction

### Phase 3: Git Transport
- Add Git repository adapter
- Enable version-controlled package storage
- Test pull-only workflows

### Phase 4: Message Platform Transports
- Add Telegram adapter
- Add Discord adapter
- Add Slack adapter
- Test message-based protocols

### Phase 5: Plugin System
- External transport plugins
- Dynamic loading of custom transports
- Community-contributed adapters

## Plugin API for Custom Transports

```rust
// External plugin interface
#[no_mangle]
pub extern "C" fn pray_transport_init() -> *mut dyn TransportAdapter {
    Box::into_raw(Box::new(MyCustomTransport::new()))
}

#[no_mangle]
pub extern "C" fn pray_transport_capabilities() -> TransportCapabilities {
    TransportCapabilities {
        pull: true,
        push: false,
        streaming: false,
        binary: true,
        max_message_size: Some(1024 * 1024),
        partial_responses: false,
    }
}
```

Load plugin:
```toml
[[federation.plugins]]
name = "custom-transport"
path = "/path/to/libcustom_transport.so"

[[federation.peers]]
name = "custom-peer"
transport = "custom-transport"
```

## Testing Strategy

**Unit tests:**
- Each adapter independently
- Mock transport responses
- Capability reporting

**Integration tests:**
- Real transport services (test bots)
- Multi-transport scenarios
- Fallback behavior

**End-to-end tests:**
- Publish via HTTP → sync via Telegram → install
- Cross-transport verification
- Signature validation across transports

## References

**Similar systems:**
- libp2p: Pluggable transports for P2P
- IPFS: Multiple transport protocols
- Matrix bridges: Protocol translation
- Git remotes: Multiple transport schemes
- Email: SMTP/IMAP/POP3 flexibility

**Related Pray docs:**
- `docs/issues/20260626193000_server_to_server_federation_protocol.md`
- `docs/issues/20260626200000_lessons_from_graphql_protobuf_grpc.md`
- `SPEC.md` Section 29.2: Server-to-server federation
