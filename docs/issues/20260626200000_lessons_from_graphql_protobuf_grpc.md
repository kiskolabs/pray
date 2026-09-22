# Lessons from GraphQL, Protocol Buffers, and gRPC for Pray Federation

## Overview

While Pray's federation protocol starts with JSON over HTTP/REST, GraphQL, Protocol Buffers, and gRPC offer valuable lessons for API design, schema evolution, and efficiency.

## GraphQL Lessons

### What GraphQL does well

**Schema introspection:**
- Clients can query the schema itself
- Self-documenting APIs
- Tooling can generate types and validation
- Enables discovery without external documentation

**Precise data fetching:**
- Clients request exactly the fields they need
- Reduces over-fetching (too much data)
- Reduces under-fetching (multiple round trips)
- Single request can fetch nested relationships

**Strong typing:**
- Schema defines types, fields, and relationships
- Compile-time validation
- Better IDE support and autocomplete
- Clear contracts between client and server

**Versioning through evolution:**
- Add new fields without breaking clients
- Deprecate old fields but keep them working
- No URL versioning needed
- Gradual migration path

### What GraphQL struggles with

**Complexity:**
- Query language adds cognitive overhead
- N+1 query problem requires careful resolver design
- Caching harder than REST (no URL-based caching)
- Rate limiting more complex (can't just count requests)

**Overkill for simple APIs:**
- CRUD operations don't need GraphQL's flexibility
- REST is simpler for straightforward resource access
- Package metadata is relatively flat

**Performance characteristics:**
- Parse and execute query language
- Resolver overhead for each field
- Can be slower than optimized REST endpoints

### Application to Pray

**What we should adopt:**

✅ **Schema endpoint for introspection:**
```json
GET /.well-known/pray-federation-schema.json

{
  "version": "v1",
  "endpoints": {
    "index": {
      "url": "/v1/sync/index",
      "method": "GET",
      "parameters": {
        "since": {
          "type": "timestamp",
          "required": false,
          "description": "Only return packages updated after this time"
        }
      },
      "response": {
        "type": "IndexResponse",
        "fields": ["spec", "sync_version", "packages"]
      }
    }
  },
  "types": {
    "IndexResponse": {
      "spec": "string",
      "sync_version": "integer",
      "packages": ["PackageSummary"]
    },
    "PackageSummary": {
      "name": "string",
      "updated_at": "timestamp",
      "url": "string"
    }
  }
}
```

✅ **Partial responses:**
Allow clients to request specific fields to reduce bandwidth:
```
GET /v1/sync/package/sample/base?fields=name,versions.version,versions.artifact_hash
```

✅ **Schema versioning:**
Add version to schema, support multiple versions:
```json
{
  "spec": "pray-federation-v1",
  "schema_version": "1.2.0",
  "supported_versions": ["v1", "v2-preview"]
}
```

❌ **What we should skip:**
- Full GraphQL query language (overkill for package sync)
- Custom resolver logic (our data is static)
- GraphQL-specific tooling requirements

## Protocol Buffers Lessons

### What Protocol Buffers does well

**Efficient binary format:**
- Much smaller than JSON (3-10x reduction)
- Faster to parse and serialize
- Lower bandwidth costs
- Better for large-scale systems

**Schema-first design:**
- `.proto` files define data structures
- Types are explicit and enforced
- Code generation for multiple languages
- Self-documenting contracts

**Evolution support:**
- Field numbers enable backward/forward compatibility
- Add optional fields without breaking old clients
- Remove deprecated fields safely
- Clear migration path

**Strong typing:**
- Catch errors at compile time
- Better performance (no runtime type checking)
- Clear data contracts

### What Protocol Buffers struggles with

**Binary format opacity:**
- Can't inspect messages with curl or browser
- Requires tools to decode
- Harder to debug
- Requires schema to parse

**Schema dependency:**
- Clients need the `.proto` file
- Breaking changes harder to detect at runtime
- Schema distribution is a separate problem

**Limited expressiveness:**
- No native union types (use oneof)
- No optional/required distinction in proto3
- JSON mapping has quirks

### Application to Pray

**What we should adopt:**

✅ **Binary format option for Phase 2+:**
```
GET /v1/sync/package/sample/base
Accept: application/x-protobuf
```

Server responds with binary protobuf if supported, falls back to JSON otherwise.

✅ **Schema definition language:**
Even if we use JSON, define a schema language (JSON Schema or custom):

```json
// pray-federation.schema.json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "definitions": {
    "PackageMetadata": {
      "type": "object",
      "required": ["name", "versions"],
      "properties": {
        "name": {
          "type": "string",
          "pattern": "^[a-z0-9-]+/[a-z0-9-]+$"
        },
        "versions": {
          "type": "array",
          "items": { "$ref": "#/definitions/PackageVersion" }
        }
      }
    }
  }
}
```

✅ **Field evolution rules:**
Document compatibility rules similar to protobuf:
- Never remove required fields
- Never change field types
- Add new fields as optional
- Use default values for missing fields

✅ **Efficient encoding for large payloads:**
For artifact downloads, consider binary format:
```
# Artifact metadata in JSON (human-readable)
GET /v1/artifacts/sample/base/1.4.3.json

# Artifact payload in efficient format (binary)
GET /v1/artifacts/sample/base/1.4.3.praypkg
```

❌ **What we should skip:**
- Requiring protobuf for all communication (reduces accessibility)
- `.proto` file distribution complexity
- Breaking REST/HTTP conventions

**Recommendation:**
- **Phase 1**: JSON only (debuggability, accessibility, simple tooling)
- **Phase 2**: Add optional protobuf support with content negotiation
- **Phase 3**: Measure bandwidth savings, optimize hot paths

## gRPC Lessons

### What gRPC does well

**HTTP/2 multiplexing:**
- Multiple streams over single connection
- Reduced latency
- Better connection reuse
- Built-in flow control

**Streaming:**
- Server streaming: push updates to client
- Client streaming: upload large data in chunks
- Bidirectional streaming: real-time sync
- Efficient for long-lived connections

**Strong service contracts:**
- `.proto` service definitions
- Clear RPC boundaries
- Code generation for clients
- Type-safe APIs

**Deadline/timeout propagation:**
- Timeouts propagate through call chain
- Better distributed system behavior
- Cancellation support

### What gRPC struggles with

**Browser support:**
- Requires gRPC-Web proxy
- No native browser support
- Extra infrastructure complexity

**Debugging difficulty:**
- Binary protocol harder to inspect
- Need special tools (grpcurl)
- Less accessible than REST

**HTTP/2 requirement:**
- Not universally available
- Proxy/firewall issues
- Fallback complexity

**Ecosystem maturity:**
- Less universal than REST
- Fewer tools and libraries
- Steeper learning curve

### Application to Pray

**What we should adopt:**

✅ **HTTP/2 when available:**
Use HTTP/2 for multiplexing, fall back to HTTP/1.1:
```
# Server advertises HTTP/2 support
Alt-Svc: h2=":7429"

# Client uses HTTP/2 if available, HTTP/1.1 otherwise
```

✅ **Streaming for large responses:**
```
GET /v1/sync/index?stream=true

# Server sends JSON stream (newline-delimited JSON)
{"name": "sample/base", "updated_at": "..."}
{"name": "sample/webapp", "updated_at": "..."}
{"name": "company/internal", "updated_at": "..."}
```

Or use HTTP chunked transfer encoding:
```
Transfer-Encoding: chunked

# First chunk
{"packages": [
  {"name": "sample/base", ...}

# Second chunk
  ,{"name": "sample/webapp", ...}

# Final chunk
]}
```

✅ **Bidirectional sync (future):**
WebSocket or HTTP/2 server push for real-time updates:
```
# Phase 4: Real-time sync notifications
WebSocket: wss://prayers.kisko.dev/v1/sync/stream

{
  "event": "package_published",
  "package": "sample/base",
  "version": "1.5.0",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

✅ **Timeout propagation:**
```
GET /v1/sync/package/sample/base
X-Request-Timeout: 30s

# Server respects timeout, returns 504 Gateway Timeout if exceeded
```

❌ **What we should skip:**
- Full gRPC adoption (reduces accessibility)
- Requiring HTTP/2 (not universal)
- Binary-only protocol (hard to debug)

## Hybrid Approach for Pray

### Phase 1: JSON over HTTP/REST

**Simple and accessible:**
- JSON for all responses
- REST endpoints
- HTTP/1.1 or HTTP/2
- Standard HTTP methods (GET, POST)

**Benefits:**
- curl-friendly debugging
- Browser-friendly
- Universal tool support
- No schema distribution needed

**Drawbacks:**
- Larger payloads
- Slower parsing
- No streaming

### Phase 2: Efficient Binary Option

**Add protobuf support via content negotiation:**
```
# Client requests protobuf
GET /v1/sync/package/sample/base
Accept: application/x-protobuf

# Server responds with protobuf if supported
Content-Type: application/x-protobuf

[binary protobuf data]

# Or falls back to JSON
Content-Type: application/json
```

**Benefits:**
- 70-90% bandwidth reduction
- Faster parsing
- Still supports JSON fallback
- Backward compatible

**Schema distribution:**
```
# Protobuf schema available for clients
GET /.well-known/pray-federation.proto

service PrayFederation {
  rpc GetIndex(IndexRequest) returns (IndexResponse);
  rpc GetPackage(PackageRequest) returns (PackageResponse);
}

message IndexRequest {
  optional int64 since = 1;
}

message IndexResponse {
  string spec = 1;
  int64 sync_version = 2;
  repeated PackageSummary packages = 3;
}
```

### Phase 3: Streaming and HTTP/2

**Add streaming for large responses:**
```
# Newline-delimited JSON stream
GET /v1/sync/index?stream=true
Accept: application/x-ndjson

{"name": "sample/base", "updated_at": "2024-01-01T00:00:00Z"}
{"name": "sample/webapp", "updated_at": "2024-01-02T00:00:00Z"}
```

**HTTP/2 server push for dependencies:**
When client requests a package, server pushes related packages:
```
# Client requests package
GET /v1/sync/package/sample/webapp

# Server pushes dependency metadata (HTTP/2)
PUSH_PROMISE: /v1/sync/package/sample/base
```

### Phase 4: Real-time Sync

**WebSocket or gRPC streaming for live updates:**
```
# Subscribe to package updates
WebSocket: wss://prayers.kisko.dev/v1/sync/live

> {"subscribe": ["sample/*", "company/*"]}

< {"event": "package_published", "package": "sample/base", "version": "1.5.0"}
< {"event": "package_yanked", "package": "sample/old", "version": "1.0.0"}
```

## Comparison Matrix

| Feature | REST+JSON | GraphQL | Protobuf | gRPC | Pray Phase 1 | Pray Phase 2+ |
|---------|-----------|---------|----------|------|--------------|---------------|
| Human-readable | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ (fallback) |
| Efficient | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ (optional) |
| Streaming | ⚠️ | ⚠️ | ✅ | ✅ | ❌ | ✅ |
| Schema | ⚠️ | ✅ | ✅ | ✅ | ⚠️ | ✅ |
| Browser-friendly | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ |
| Caching | ✅ | ⚠️ | ❌ | ⚠️ | ✅ | ✅ |
| Debugging | ✅ | ✅ | ❌ | ❌ | ✅ | ⚠️ |
| Learning curve | Low | Med | Med | High | Low | Med |

## Recommendations for Pray

### Immediate (Phase 1)

1. **JSON over HTTP/REST**
   - Simple, accessible, debuggable
   - Standard REST endpoints
   - HTTP/1.1 or HTTP/2 automatic

2. **Schema endpoint**
   - Add `/.well-known/pray-federation-schema.json`
   - JSON Schema for validation
   - Self-documenting API

3. **Partial responses**
   - Support `?fields=` parameter
   - Reduce over-fetching

4. **Versioning**
   - Version in discovery endpoint
   - Support multiple versions

### Near-term (Phase 2)

1. **Binary format option**
   - Add protobuf support via `Accept` header
   - Measure bandwidth savings
   - Keep JSON as default

2. **Streaming support**
   - Newline-delimited JSON for large lists
   - HTTP chunked transfer encoding

3. **HTTP/2 optimization**
   - Use multiplexing when available
   - Fall back to HTTP/1.1 gracefully

### Future (Phase 3+)

1. **Real-time sync**
   - WebSocket for live updates
   - Server push for urgent changes (yanks)

2. **gRPC service (optional)**
   - For high-throughput S2S sync
   - Alongside REST API
   - Not required

## Why Not Full gRPC/GraphQL?

**gRPC problems for Pray:**
- Requires HTTP/2 (not universal)
- Browser support limited (needs proxy)
- Debugging harder (binary protocol)
- Overkill for static package metadata
- Less accessible for `pray serve` operators

**GraphQL problems for Pray:**
- Query language complexity unnecessary
- Package metadata is relatively flat
- No deeply nested relationships
- REST is simpler for CRUD operations
- Caching benefits lost

**Pray's sweet spot:**
- Start simple with JSON/REST (Phase 1)
- Add efficiency where it matters (Phase 2)
- Preserve accessibility and debuggability
- Gradual optimization based on real-world use

## Protobuf Schema Example (Phase 2)

```protobuf
// pray-federation.proto
syntax = "proto3";

package pray.federation.v1;

// Discovery endpoint response
message FederationInfo {
  string spec = 1;
  ServerInfo server = 2;
  SyncEndpoints sync = 3;
  repeated PeerInfo peers = 4;
}

message ServerInfo {
  string name = 1;
  string version = 2;
  repeated string capabilities = 3;
}

message SyncEndpoints {
  string index_url = 1;
  string package_url = 2;
  string artifact_url = 3;
  string since_param = 4;
}

// Index sync response
message IndexResponse {
  string spec = 1;
  int64 sync_version = 2;
  repeated PackageSummary packages = 3;
}

message PackageSummary {
  string name = 1;
  string updated_at = 2;
  string url = 3;
}

// Package metadata response
message PackageMetadata {
  string name = 1;
  repeated PackageVersion versions = 2;
  string updated_at = 3;
}

message PackageVersion {
  string version = 1;
  string artifact = 2;
  string artifact_hash = 3;
  string tree_hash = 4;
  bool yanked = 5;
  repeated string targets = 6;
  repeated string exports = 7;
  string published_at = 8;
  PublisherInfo publisher = 9;
  SignatureInfo signature = 10;
  OriginInfo origin = 11;
}

message PublisherInfo {
  string id = 1;
  string key_fingerprint = 2;
}

message SignatureInfo {
  string algorithm = 1;
  bytes signature = 2;
  string public_key = 3;
}

message OriginInfo {
  string server = 1;
  string first_seen = 2;
}
```

## References

**GraphQL:**
- GraphQL specification: https://spec.graphql.org/
- GraphQL best practices: https://graphql.org/learn/best-practices/
- Over-fetching and under-fetching: https://graphql.org/learn/thinking-in-graphs/

**Protocol Buffers:**
- Protocol Buffers documentation: https://protobuf.dev/
- Language guide: https://protobuf.dev/programming-guides/proto3/
- Style guide: https://protobuf.dev/programming-guides/style/

**gRPC:**
- gRPC documentation: https://grpc.io/docs/
- gRPC concepts: https://grpc.io/docs/what-is-grpc/introduction/
- gRPC performance best practices: https://grpc.io/docs/guides/performance/

**Related Pray docs:**
- `docs/issues/20260626193000_server_to_server_federation_protocol.md`
- `docs/distribution-architecture.md`
- `SPEC.md` Section 29.2: Server-to-server federation
