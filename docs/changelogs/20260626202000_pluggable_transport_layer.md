# Pluggable transport layer for federation

## What changed

- Added design document for pluggable transport layer in `docs/issues/20260626202000_pluggable_transport_layer.md`
- Updated S2S federation protocol to include Phase 5: Pluggable Transports
- Updated P2P/S2S summary to reference transport abstraction

## Why this matters

The federation protocol should work over **any** transport mechanism, not just HTTP. This enables Pray to sync packages through:

- **Standard**: HTTP/HTTPS
- **Custom protocols**: echo.kisko.dev or any TCP-based protocol
- **Messaging**: Telegram, Discord, Slack, Matrix
- **Email**: SMTP/IMAP for async delivery
- **Version control**: Git repositories
- **Filesystem**: Shared folders, network drives, USB keys
- **Sneakernet**: Physical media for air-gapped networks

## Design principles

**Transport independence:**
- Federation protocol defines *what* to sync (metadata, artifacts)
- Transport adapter defines *how* to deliver (HTTP, Telegram, filesystem)
- Same logical protocol works over any transport
- No transport-specific logic in core sync engine

**Adapter pattern:**
- Each transport implements `TransportAdapter` trait
- Core calls adapter methods (fetch_discovery, fetch_index, fetch_package)
- Adapter handles transport-specific details
- Easy to add new transports without modifying core

**Capability-based:**
- Transports declare capabilities (pull, push, streaming, binary)
- Core checks capabilities before attempting operations
- Graceful degradation for limited transports

**Security per transport:**
- Each transport uses appropriate authentication
- End-to-end validation always happens (signatures, hashes)
- Transport only moves bytes, doesn't establish trust
- Untrusted transport can't inject fake packages

## Example configurations

### USB Sneakernet
```toml
[[federation.peers]]
name = "usb-key"
transport = "filesystem"
path = "/media/usb/pray-packages"

# Workflow:
# 1. pray export --output /media/usb/pray-packages
# 2. Walk USB to air-gapped system
# 3. pray import --input /media/usb/pray-packages
```

### Telegram Fallback
```toml
[[federation.peers]]
name = "telegram-backup"
transport = "telegram"

[federation.peers.telegram-backup.telegram]
bot_token = "123456:ABC..."
channel = "@pray_packages"
poll_interval = "60s"
```

### Discord Team Channel
```toml
[[federation.peers]]
name = "discord-team"
transport = "discord"

[federation.peers.discord-team.discord]
bot_token = "MTk4NjI..."
channel_id = "987654..."
use_threads = true  # One thread per package
```

### Git Repository
```toml
[[federation.peers]]
name = "git-mirror"
transport = "git"

[federation.peers.git-mirror.git]
repository = "git@github.com:company/pray-packages.git"
branch = "main"
webhook = true
```

### Multi-transport with Priorities
```toml
# Try HTTP first
[[federation.peers]]
name = "primary"
transport = "http"
url = "https://prayers.kisko.dev"
priority = 1

# Fall back to Telegram if HTTP fails
[[federation.peers]]
name = "fallback"
transport = "telegram"
channel = "@pray_backup"
priority = 2

# Offline USB as last resort
[[federation.peers]]
name = "offline"
transport = "filesystem"
path = "/media/usb/pray"
priority = 3
```

## Use cases

**Corporate air-gap:**
- Internet side exports to USB
- Walk USB across physical gap
- Air-gap side imports from USB

**Remote office with poor connectivity:**
- Use Telegram for small packages
- Fall back to HTTP when available
- Offline USB for bulk sync

**Dev team collaboration:**
- Discord bot posts packages to team channel
- Team reviews metadata before installing
- Thread per package for history

**Version-controlled packages:**
- Git repository as package store
- Review changes via pull requests
- Audit history in Git log

## Transport capability matrix

| Transport | Pull | Push | Stream | Binary | Max Size | Latency |
|-----------|------|------|--------|--------|----------|---------|
| HTTP | ✅ | ✅ | ✅ | ✅ | Unlimited | Low |
| Telegram | ✅ | ✅ | ⚠️ | ✅ | 50MB | Medium |
| Discord | ✅ | ✅ | ⚠️ | ✅ | 25MB | Medium |
| Slack | ✅ | ✅ | ⚠️ | ✅ | 1GB | Medium |
| Matrix | ✅ | ✅ | ✅ | ✅ | 50MB | Medium |
| Email | ✅ | ✅ | ❌ | ✅ | 25MB | High |
| Git | ✅ | ✅ | ❌ | ✅ | Unlimited | High |
| Filesystem | ✅ | ✅ | ✅ | ✅ | Unlimited | Low |

## Implementation phases

**Phase 1**: HTTP only (foundation)
**Phase 2**: Filesystem transport (USB, shared folders)
**Phase 3**: Git transport (repository-based)
**Phase 4**: Message platform transports (Telegram, Discord, Slack)
**Phase 5**: Plugin system (community transports)

## Security guarantee

Even over completely untrusted transport (public Telegram channel):
1. Metadata includes signature
2. Client verifies signature before accepting
3. Artifact hash checked before installing
4. Malicious transport **cannot** inject fake packages

Transport moves bytes. Core validates trust.

## References

- Design doc: `docs/issues/20260626202000_pluggable_transport_layer.md`
- S2S protocol: `docs/issues/20260626193000_server_to_server_federation_protocol.md`
- Summary: `docs/p2p-s2s-summary.md`
- Similar: libp2p transports, IPFS multiaddr, Matrix bridges, Git remotes
