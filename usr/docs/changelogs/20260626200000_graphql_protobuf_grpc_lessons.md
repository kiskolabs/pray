# GraphQL, Protocol Buffers, and gRPC lessons for federation API design

## What changed

- Added design document analyzing GraphQL, Protocol Buffers, and gRPC lessons in `docs/issues/20260626200000_lessons_from_graphql_protobuf_grpc.md`
- Updated S2S federation protocol to reference format choices
- Updated P2P/S2S summary with hybrid approach explanation

## Why this matters

Modern API technologies offer valuable lessons for Pray's federation protocol design:

**GraphQL contributions:**
- Schema introspection and self-documenting APIs
- Partial responses to reduce over-fetching
- Strong typing and validation
- Versioning through schema evolution

**Protocol Buffers contributions:**
- Efficient binary format (70-90% smaller than JSON)
- Schema-first design with code generation
- Backward/forward compatibility through field numbering
- Clear evolution rules

**gRPC contributions:**
- HTTP/2 multiplexing and streaming
- Bidirectional communication
- Timeout propagation
- Strong service contracts

## Design decision

**Phase 1: JSON over HTTP/REST**
- Start simple and accessible
- curl-friendly debugging
- Universal tool support
- No schema distribution needed

**Phase 2: Add binary option**
- Optional protobuf via content negotiation (`Accept: application/x-protobuf`)
- Measure real-world bandwidth savings
- Keep JSON as default fallback
- Backward compatible

**Phase 3: Streaming**
- Newline-delimited JSON for large responses
- HTTP/2 server push for dependencies
- WebSocket for real-time updates

**Phase 4: Optional gRPC**
- For high-throughput S2S sync
- Alongside REST API
- Not required for conformance

## Key insights

**Why not full GraphQL:**
- Query language overkill for static package metadata
- Package data is flat, not deeply nested
- REST caching benefits lost
- Adds complexity without proportional value

**Why not full gRPC:**
- Binary protocol harder to debug
- Browser support requires proxy
- Not universal (firewall issues)
- HTTP/2 requirement too restrictive
- Less accessible for `pray serve` operators

**Why not protobuf-only:**
- Can't debug with curl or browser
- Requires schema distribution
- Breaks REST conventions
- Reduces accessibility

**Pray's sweet spot:**
- Start with JSON/REST (simple, accessible)
- Add efficiency where measured (protobuf optional)
- Preserve debuggability (JSON always available)
- Gradual optimization based on real use

## Features borrowed

From GraphQL:
- Schema endpoint: `/.well-known/pray-federation-schema.json`
- Partial responses: `?fields=name,versions.version`
- Schema versioning and evolution

From Protocol Buffers:
- Optional binary format via content negotiation
- Field evolution rules documented
- Schema available at `/.well-known/pray-federation.proto`

From gRPC:
- HTTP/2 when available, HTTP/1.1 fallback
- Streaming for large responses
- Timeout propagation headers
- Future: WebSocket for real-time sync

## Comparison matrix

| Feature | REST+JSON | GraphQL | Protobuf | gRPC | Pray Phase 1 | Pray Phase 2+ |
|---------|-----------|---------|----------|------|--------------|---------------|
| Human-readable | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ (fallback) |
| Efficient | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ (optional) |
| Streaming | ⚠️ | ⚠️ | ✅ | ✅ | ❌ | ✅ |
| Schema | ⚠️ | ✅ | ✅ | ✅ | ⚠️ | ✅ |
| Debugging | ✅ | ✅ | ❌ | ❌ | ✅ | ⚠️ |

## References

- Design doc: `docs/issues/20260626200000_lessons_from_graphql_protobuf_grpc.md`
- S2S protocol: `docs/issues/20260626193000_server_to_server_federation_protocol.md`
- Summary: `docs/p2p-s2s-summary.md`
- GraphQL spec: https://spec.graphql.org/
- Protocol Buffers: https://protobuf.dev/
- gRPC: https://grpc.io/docs/
