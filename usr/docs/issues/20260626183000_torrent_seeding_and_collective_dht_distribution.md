# Torrent seeding and collective DHT distribution for Pray packages

Add documentation for a future distribution model that can seed packages through torrent-style swarms and discover package content through a collective DHT, inspired by BitTorrent, Freenet, and GNUnet.

## Desired capabilities
- keep the existing static registry and direct-source model
- allow packages to be seeded over peer-to-peer transports
- allow package discovery through a DHT-backed network index
- preserve hash verification, signatures, and provenance across all transports
- keep the P2P transport optional so the spec still works with local, private, and static hosting

## Why this matters
This gives Prayfile a path to resilient distribution that does not depend on a single server while still keeping the same static declaration and verification model.

## Suggested first documentation slice
1. Add a short note in `README.md` describing P2P seeding and DHT discovery as a future distribution transport.
2. Add a matching normative note in `SPEC.md` under distribution points.
3. Keep the core guarantee unchanged: distribution stays static, hash-verified, and non-executable.

## Next

README already has the future-tense P2P sentence. SPEC.md was retired; RFC 0060, RFC 0062, and RFC 0104 hold the contract. DHT announce still fails parse. Magnet and tracker announce are not implemented. Follow usr/docs/issues/20260916225000_git-free-distribution.md for static-host recipes and the e2e recommendation.

## Source

usr/docs/issues/20260916225000_git-free-distribution.md
rfcs/0062-distribution-protocols.md
rfcs/0104-federation-transports.md