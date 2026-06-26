# XMPP-inspired DNS discovery for Pray federation

## Overview

Phase 3 of Pray federation may add DNS SRV record support for automatic peer discovery, inspired by XMPP's federation model (RFC 6120).

## DNS SRV records

Example SRV record for a Pray federation endpoint:

```
_pray-federation._tcp.prayers.kisko.dev. 86400 IN SRV 0 5 7429 sync.prayers.kisko.dev.
```

Format breakdown:
- `_pray-federation._tcp`: Service and protocol
- `prayers.kisko.dev`: Domain being queried
- `0`: Priority (lower is higher priority)
- `5`: Weight (for load balancing among same priority)
- `7429`: Port number
- `sync.prayers.kisko.dev.`: Target hostname

## Benefits

**Automatic discovery:**
- Clients/servers can discover federation endpoints without hardcoded URLs
- Domain owner controls federation endpoint through DNS

**Load balancing:**
- Multiple SRV records with different weights
- Distribute sync load across multiple servers

**Priority routing:**
- Primary and backup endpoints
- Graceful failover

**No configuration:**
- Reduces manual peer configuration
- Easier to update endpoints (DNS change vs config push)

## Query flow

```
1. Server wants to sync with prayers.kisko.dev
2. Query: _pray-federation._tcp.prayers.kisko.dev (SRV)
3. Response: sync.prayers.kisko.dev:7429 priority=0 weight=5
4. Connect to sync.prayers.kisko.dev:7429
5. Fetch /.well-known/pray-federation.json
6. Begin sync protocol
```

## Security requirements

**DNSSEC validation:**
- SRV records must be DNSSEC-signed
- Unsigned records ignored in secure mode
- Prevents DNS spoofing attacks

**TLS verification:**
- TLS certificate must match resolved hostname
- Certificate pinning optional but recommended
- Same security as manual configuration

**Origin validation:**
- Package signatures still required
- DNS only discovers endpoint, doesn't establish trust
- Malicious DNS can't inject invalid packages

**Fallback:**
- If DNS query fails, fall back to manual config
- If DNSSEC validation fails, fall back or error
- DNS is convenience, not requirement

## XMPP comparison

**What XMPP does:**
- RFC 6120 Section 3.2 defines SRV lookups
- `_xmpp-server._tcp.example.com` for S2S
- Dialback or SASL for server authentication
- STARTTLS for transport security

**What Pray would do:**
- `_pray-federation._tcp.example.com` for S2S sync
- API keys or mTLS for server authentication
- TLS from the start (no STARTTLS upgrade)
- Simpler: fewer round trips, static content

## Example configurations

### DNS zone file

```zone
; Federation endpoint
_pray-federation._tcp.prayers.kisko.dev. 86400 IN SRV 0 10 7429 sync1.prayers.kisko.dev.
_pray-federation._tcp.prayers.kisko.dev. 86400 IN SRV 0 10 7429 sync2.prayers.kisko.dev.
_pray-federation._tcp.prayers.kisko.dev. 86400 IN SRV 10 0 7429 backup.prayers.kisko.dev.

; DNSSEC signatures
prayers.kisko.dev. IN DNSKEY ...
prayers.kisko.dev. IN RRSIG ...
```

### Server config with DNS discovery

```toml
[federation]
enabled = true

[[federation.peers]]
name = "kisko"
domain = "prayers.kisko.dev"  # Uses DNS SRV lookup
trust = "full"
direction = "pull"
require_dnssec = true

[[federation.peers]]
name = "backup"
url = "https://backup.example.com"  # Manual URL, no DNS
trust = "full"
direction = "pull"
```

## Implementation complexity

**Low complexity:**
- Standard DNS libraries support SRV queries
- DNSSEC validation built into resolvers
- No new protocols to implement

**Medium complexity:**
- Fallback logic (DNS → manual)
- Error handling (no SRV, multiple SRV, invalid)
- Testing with real DNS infrastructure

**High complexity (optional):**
- Dynamic peer refresh based on TTL
- Health checks and automatic failover
- Monitoring and alerting for DNS changes

## Phase 3 recommendations

**Must have:**
- Basic SRV query support
- DNSSEC validation
- TLS certificate verification
- Fallback to manual config

**Should have:**
- Multiple SRV record support
- Priority and weight-based selection
- DNS query caching (respect TTL)

**Could have:**
- Dynamic peer refresh
- Negative caching (no SRV found)
- Health-based failover

**Won't have (Phase 3):**
- Automatic peer trust establishment
- Distributed namespace authority
- Real-time DNS updates

## References

- RFC 6120: XMPP Core (Section 3.2: Resolution of Fully Qualified Domain Names)
- RFC 2782: DNS SRV records
- RFC 4033-4035: DNSSEC specifications
- `docs/issues/20260626193000_server_to_server_federation_protocol.md`: Main federation design
