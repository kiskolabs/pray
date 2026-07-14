# SSH-native distribution transport

## Participants

Engineering design for pray client and pray server communication over SSH only, without HTTP.

## Decisions

Pray should support a standalone distribution mode where the wire protocol between client and server runs inside an SSH session. The application protocol stays the same as today's `pray serve` HTTP API; only the transport changes.

URL scheme: `pray+ssh://`. Alias: `ssh://` when the Prayfile source kind is explicitly `pray_ssh` or when the manifest parser infers pray SSH from a `pray+ssh://` prefix.

Deployment model: OpenSSH `ForceCommand` or `Subsystem pray` launches `pray serve --stdio --root <path>`. The server does not bind an HTTP port. Encryption and connection authentication come from SSH (host keys, user keys, agent).

Framing: length-prefixed frames on stdin and stdout of the pray server process. Each frame is a UTF-8 JSON document. Maximum frame size is implementation-defined; conforming implementations must accept at least 16 MiB per frame.

RPC envelope: every request and response includes `spec: "pray-ssh-rpc-v1"` and a correlation `id`. Responses include `status` (HTTP-equivalent code), `content_type`, and `body`. JSON bodies are embedded as JSON values. Binary bodies use `body_encoding: "base64"`.

Authentication in SSH-native mode: a successful SSH connection satisfies transport authentication. The server maps the SSH public key fingerprint (or Unix user) to a publisher identity for publish operations. HTTP-style `auth.*` RPC methods remain available for hybrid hosts that expose both HTTP and SSH, but are not required for private SSH-only servers.

HTML routes (`GET /`, `GET /packages/...`) are not exposed over SSH-RPC. Clients use JSON methods only.

## Effects

Specification section 29.5 documents the transport. Reference implementation includes stdio RPC server, SSH transport adapter, pray+ssh source resolution, publisher push policy (`v1/ssh_publishers.json` + `PRAY_SSH_PUBLISHER`), session lifecycle fixes, HTTP API routing through `handle_rpc`, and stdio integration tests (install, publish round-trip, push auth, sync pull).

## Next

1. Optional OpenSSH CI fixture with real `authorized_keys` when a runner provides it.

## Source

- `crates/pray-cli/src/server.rs` HTTP route table
- `crates/pray-transport/src/types.rs` `TransportAdapter` trait
- `docs/issues/20260626202000_pluggable_transport_layer.md`
- Prior discussion: SSH as sole transmission layer between pray client and pray server

## RPC mapping from HTTP routes

All methods use POST semantics at the RPC layer (there is no GET). Params replace path segments and query strings.

### Core distribution (required)

| HTTP (reference CLI today) | RPC method | Params | Response body type |
|-----------------------------|------------|--------|-------------------|
| `GET /.well-known/pray-federation.json` | `federation.discovery` | `{}` | `FederationInfo` |
| `GET /v1/sync/index?since=<ts>` | `sync.index` | `{ "since": <i64 optional> }` | `IndexResponse` |
| `GET /v1/sync/package/{name}` | `sync.package` | `{ "name": "<package>" }` | `PackageMetadata` |
| `POST /v1/sync/push` | `sync.push` | `{ "metadata": <PackageMetadata> }` | `{ "status": "ok", "package": "..." }` |
| `GET /v1/artifacts/...` or static `GET` under artifact path | `artifact.get` | `{ "path": "<relative path>" }` | raw bytes, base64 in envelope |
| `PUT /v1/artifacts/...` | `artifact.put` | `{ "path": "<relative path>", "body": "<base64>" }` | `{ "status": "ok", "artifact": "..." }` |

`artifact.get` and `artifact.put` paths use the same relative layout as static hosting (`v1/artifacts/...`). `sanitize_request_path` rules from the HTTP server apply.

### Confessions (optional)

| HTTP | RPC method | Params | Response body type |
|------|------------|--------|-------------------|
| `POST /v1/confessions` | `confession.submit` | `ConfessionSubmission` | `{ "status": "ok", "package": "...", "version": "..." }` |

### HTTP auth (optional, hybrid hosts)

| HTTP | RPC method |
|------|------------|
| `POST /v1/auth/register` | `auth.register` |
| `POST /v1/auth/verify` | `auth.verify` |
| `POST /v1/auth/session` | `auth.session` |
| `POST /v1/auth/passkeys/challenge` | `auth.passkeys.challenge` |
| `POST /v1/auth/passkeys/login` | `auth.passkeys.login` |
| `POST /v1/auth/passkeys/enroll` | `auth.passkeys.enroll` |
| `POST /v1/auth/ssh-keys/challenge` | `auth.ssh_keys.challenge` |
| `POST /v1/auth/ssh-keys/login` | `auth.ssh_keys.login` |
| `POST /v1/auth/ssh-keys/enroll` | `auth.ssh_keys.enroll` |

Request and response JSON shapes match the existing `pray_core::auth` types. SSH-native servers may return `405` equivalent (`status: 405`) for these methods when HTTP auth is disabled.

### Not mapped

| HTTP | Reason |
|------|--------|
| `GET /` | HTML index; use `sync.index` |
| `GET /packages/{name}` | HTML package page; use `sync.package` |

## Frame format

```text
request  := u32_be(length) json_request
response := u32_be(length) json_response
```

Request:

```json
{
  "spec": "pray-ssh-rpc-v1",
  "id": "1",
  "method": "sync.package",
  "params": {
    "name": "sample/base"
  }
}
```

Success response:

```json
{
  "spec": "pray-ssh-rpc-v1",
  "id": "1",
  "status": 200,
  "content_type": "application/json",
  "body": {
    "name": "sample/base",
    "versions": [],
    "updated_at": "0"
  }
}
```

Binary success response:

```json
{
  "spec": "pray-ssh-rpc-v1",
  "id": "2",
  "status": 200,
  "content_type": "application/octet-stream",
  "body_encoding": "base64",
  "body": "..."
}
```

Error response:

```json
{
  "spec": "pray-ssh-rpc-v1",
  "id": "1",
  "status": 404,
  "content_type": "application/json",
  "body": {
    "error": "package metadata not found: sample/missing"
  }
}
```

## Session lifecycle

1. Client opens SSH to `user@host` (port 22 unless overridden in URL).
2. Remote executes `pray serve --stdio --root <root>` (via ForceCommand or Subsystem).
3. Client sends one or more framed RPC requests on the session stdin; reads framed responses from stdout.
4. Client closes SSH when done. Server exits when stdin closes.

Multiple requests per SSH session are allowed. Implementations should reuse one session for install/publish workflows.

## Prayfile examples

```manifest
source "team", "pray+ssh://pray@prayers.internal"
source "team", "pray+ssh://pray@prayers.internal:2222"
```

Lockfile records:

```toml
[[source]]
name = "team"
kind = "pray_ssh"
url = "pray+ssh://pray@prayers.internal"
```

## Server configuration sketch

```sshconfig
Subsystem pray /usr/bin/pray serve --stdio --root /var/lib/pray

Match User pray
    ForceCommand /usr/bin/pray serve --stdio --root /var/lib/pray
    AllowTcpForwarding no
    X11Forwarding no
```

Publisher mapping file (implementation-defined, suggested path `v1/ssh_publishers.json`):

```json
{
  "publishers": [
    {
      "fingerprint": "SHA256:abcdef...",
      "id": "team-ci",
      "push": true
    }
  ]
}
```

Connections without a listed fingerprint may still pull (`sync.index`, `sync.package`, `artifact.get`). Push methods require a mapped publisher with `push: true`.

## TransportAdapter mapping

`SshTransport` implements the same `TransportAdapter` methods as `HttpTransport`:

- `fetch_discovery` -> `federation.discovery`
- `fetch_index` -> `sync.index`
- `fetch_package` -> `sync.package`
- `fetch_artifact` -> `artifact.get`
- `push_package` -> `sync.push` (after `artifact.put` for each new artifact)

Peer config:

```toml
[[federation.peers]]
name = "team-vps"
transport = "ssh"
url = "pray+ssh://pray@prayers.internal"
trust = "full"
direction = "bidirectional"
```

## Verification unchanged

Package hashes, tree hashes, signatures, and render digests are verified on the client the same way as for HTTP or static hosting. SSH only moves bytes and authenticates the host and connection.
