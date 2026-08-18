# RFC 0104: Federation and transports

- Feature Name: federation-transports
- Type: Standards Track
- Status: Experimental
- Created: 2026-08-17
- Author: Andrei Makarov
- Relates: RFC 0050, RFC 0060, RFC 0101
- Requires: RFC 0050, RFC 0060

## Summary

Optional server-to-server sync and pluggable transports. The logical protocol (what is synced) MUST stay independent of how bytes move. V1 install MUST keep working with static HTTP and git only. Install does not use `pray-transport`. Wire paths stay Experimental until this RFC is Stable.

## Motivation

Adapters grew before a single wire document. Clients need a guarantee that a new transport cannot weaken verify (RFC 0050). Duplicate HTTP, SSH, and torrent code in `pray-core` and `pray-transport` will diverge (RFC 0101).

## Guide-level explanation

An operator lists peers, sets pull or bidirectional, and runs `pray sync`. Clients still install from one URL.

A Level 3 implementation MUST support path, tarball, git, and static registry HTTP. Federation, torrent, P2P, and SSH distribution MAY be omitted; if present they MUST verify the same hashes as HTTP. Signatures follow RFC 0050: mismatch fails when a signature is present; missing signature fails only under require-signed policy.

Explicit peer trust. Eventual consistency. Static and standalone `pray serve` remain valid with federation disabled.

Production operators SHOULD treat federation as exploratory until Stable plus conformance fixtures (RFC 0100).

## Reference-level explanation

Key words follow RFC 2119.

Transport independence: discovery, index, metadata, artifact fetch as capabilities; adapters declare pull/push/streaming.

Experimental HTTP wire served by `server_federation.rs` and fetched by `HttpTransport`: GET `{peer}/.well-known/pray-federation.json` (`spec` `pray-federation-v1`); GET `{peer}/v1/sync/index` with optional `since`; GET `{peer}/v1/sync/package/{name}`; POST `{peer}/v1/sync/push` gated by `authorize_distribution_push` (`--allow-open-push`, bearer token, SSH publishers, or loopback). Relative artifacts use `download_registry_artifact` in `fetch.rs`; absolute `http://` or `https://` artifact URLs use `http_get`.

Experimental SSH wire (`pray+ssh://`, `SshTransport`): JSON-RPC `federation.discovery`, `sync.index`, `sync.package`, `sync.push`; bytes `artifact.get`.

`FederationTransport` and `TorrentTransport` wrap `HttpTransport`. `P2PTransport` is `FederationTransport` named `p2p`; it is not a distinct wire.

Former snapshot transport notes (still Experimental with this RFC):

#### 29.1 Peer-to-peer distribution transport

The specification should also allow a peer-to-peer transport layer for package discovery and artifact seeding.

A conforming implementation may use torrent-style swarms for content distribution and a collective DHT for discovery, inspired by BitTorrent, Freenet, and GNUnet.

Peer-to-peer transport must preserve the same package identity, artifact hash verification, signature checks, yanking semantics, and provenance guarantees as static registry hosting.

P2P transport is optional. A conforming implementation must still work with local, private, and static registry sources without it.
#### 29.2 Server-to-server federation

The specification should allow distribution points to form federated networks through explicit server-to-server (S2S) synchronization.

A conforming implementation may support a federation protocol inspired by FIDONet, NNTP, and ActivityPub where:

- Servers establish explicit peer relationships through configuration
- Servers sync package metadata, derived metadata, confessions, and artifacts from trusted peers
- Sync operates on a pull, push, or bidirectional model
- Each server validates packages before accepting them
- Consistency is eventual through periodic synchronization
- Provenance is tracked (origin server, sync path, timestamps)

Federation protocol requirements:

- Discovery endpoint at `/.well-known/pray-federation.json` exposing server capabilities and sync URLs
- Index sync endpoint returning changed packages since a timestamp
- Package metadata sync endpoint with federation-specific fields (origin, publisher, signature, derived metadata, confessions)
- Standard artifact URLs for package file retrieval
- Hash verification and signature validation before acceptance
- Conflict detection for same version with different hashes

Trust levels:

- `full`: Accept metadata, derived metadata, confessions, and artifacts; mirror packages locally
- `metadata_only`: Accept metadata, derived metadata, and confessions but fetch artifacts from origin
- `disabled`: Peer listed but sync paused

Sync directions:

- `pull`: Server fetches updates from peer
- `push`: Server sends updates to peer
- `bidirectional`: Both pull and push

Servers may optionally publish their known peer list, trusted publishers, and confession relay peers to enable discovery of the federation topology.

Federation is optional. A conforming implementation must work without federation support.
#### 29.5 SSH-native distribution transport

A conforming implementation may expose a distribution point over SSH without HTTP. The client opens an SSH session; the server runs `pray serve --stdio` (or an equivalent subsystem entrypoint) and exchanges the same logical registry operations as Section 29 and Section 29.2 through a framed RPC protocol on stdin and stdout.

This transport is optional. A conforming implementation must still support static hosting, HTTP `pray serve`, and the other source kinds in Section 28 without SSH.

##### URL scheme

Pray SSH sources use the `pray+ssh://` scheme:

```
source "team", "pray+ssh://pray@prayers.internal"
source "team", "pray+ssh://pray@prayers.internal:2222"
```

Form:

```
pray+ssh://[<user>@]<host>[:<port>][/<path>]
```

- `user` defaults to implementation policy or the current SSH username
- `port` defaults to `22`
- `path` is an optional hint; the server root is normally fixed by server configuration (`--root`)

Lockfile records:

```toml
[[source]]
name = "team"
kind = "pray_ssh"
url = "pray+ssh://pray@prayers.internal"
```

##### Deployment

A typical private host uses OpenSSH with a forced command or subsystem:

```sshconfig
Subsystem pray /usr/bin/pray serve --stdio --root /var/lib/pray
```

The server stores the same static layout as Section 29 (`v1/index.json`, `v1/packages/...`, `v1/artifacts/...`). No HTTP listener is required.

##### Wire protocol

Spec identifier: `pray-ssh-rpc-v1`

Framing:

```text
frame := u32_be(byte_length) utf8_json
```

Each SSH session carries one or more request/response frame pairs on the server process stdin and stdout.

Request envelope:

```json
{
  "spec": "pray-ssh-rpc-v1",
  "id": "<correlation-id>",
  "method": "<method>",
  "params": {}
}
```

Response envelope:

```json
{
  "spec": "pray-ssh-rpc-v1",
  "id": "<correlation-id>",
  "status": 200,
  "content_type": "application/json",
  "body": {}
}
```

Binary payloads use `content_type: "application/octet-stream"` and `body_encoding: "base64"` on the response, or base64 in `params.body` for uploads.

Conforming implementations must accept frames of at least 16 MiB.

##### RPC methods

RPC methods mirror the reference HTTP distribution API. Params replace path segments and query parameters.

Required methods:

- `federation.discovery`: `GET /.well-known/pray-federation.json`; none
- `sync.index`: `GET /v1/sync/index`; `since` optional, integer
- `sync.package`: `GET /v1/sync/package/{name}`; `name` string
- `sync.push`: `POST /v1/sync/push`; `metadata` package metadata object
- `artifact.get`: `GET` static artifact path; `path` relative path under server root
- `artifact.put`: `PUT /v1/artifacts/...`; `path`, `body` base64

Optional methods:

- `confession.submit`: `POST /v1/confessions`
- `auth.register`: `POST /v1/auth/register`
- `auth.verify`: `POST /v1/auth/verify`
- `auth.session`: `POST /v1/auth/session`
- `auth.passkeys.challenge`: `POST /v1/auth/passkeys/challenge`
- `auth.passkeys.login`: `POST /v1/auth/passkeys/login`
- `auth.passkeys.enroll`: `POST /v1/auth/passkeys/enroll`
- `auth.ssh_keys.challenge`: `POST /v1/auth/ssh-keys/challenge`
- `auth.ssh_keys.login`: `POST /v1/auth/ssh-keys/login`
- `auth.ssh_keys.enroll`: `POST /v1/auth/ssh-keys/enroll`

JSON shapes for `federation.discovery`, `sync.index`, `sync.package`, `sync.push`, artifacts, confessions, and auth match the HTTP API and federation types in Section 29.2. HTML index and package pages are not exposed over SSH-RPC.

##### Authentication

SSH-native mode uses SSH for transport authentication:

- host identity via `known_hosts` or equivalent host key pinning (`allowed_host_keys` in client `trust.toml`, optional `host_key_fingerprint` in `Prayfile.lock`)
- user identity via SSH public key fingerprints (`signer_fingerprint` in package metadata, `allowed_publishers` in client trust policy)

The server maps SSH public key fingerprints to publisher identities for push authorization (`v1/ssh_publishers.json`). The reference CLI reads `PRAY_SSH_USER_FINGERPRINT`, `SSH_USER_FINGERPRINT`, or `PRAY_SSH_PUBLISHER` on the server during push. Clients record `signer` (human label) and `signer_fingerprint` (canonical signing identity) in registry metadata.

HTTP-style `auth.*` RPC methods are optional and intended for hybrid hosts. SSH-only servers may reject them.

Package hashes, tree hashes, signatures, and render digests are still verified on the client. SSH establishes who connected and encrypts the channel; it does not replace package signature verification. Package signature formats are defined in Section 59.

##### Federation

SSH may be used as a federation transport between peers:

```toml
[[federation.peers]]
name = "team-vps"
transport = "ssh"
url = "pray+ssh://pray@prayers.internal"
trust = "full"
direction = "bidirectional"
```

The logical federation protocol in Section 29.2 is unchanged; only the wire transport differs from HTTP.

## Implementation notes

Install uses `pray-core` `registry_http.rs`, `registry_ssh.rs`, `registry_torrent.rs`, and `fetch.rs`. Sync, serve, and publish use `pray-transport` types listed in `crates/pray-transport/src/lib.rs`. `HttpTransport::fetch_artifact` still calls core `fetch` for bytes.

`pray sync` fails closed on missing or mismatched `artifact_hash`, then requires `tree_hash`, then `verify_package_signature`. Same-identity local versions skip re-fetch.

Production checklist still wants injected network failure on a two-machine path. Inference from operator notes; no fixture in-tree for that path at this writing.

## Security considerations

Open push, automatic peer discovery, and chat-channel transports expand the threat model. Each adapter MUST document authn. Package signatures remain the artifact trust; transport auth is hop security only.

## Prior art

Store-and-forward peer lists (FIDONet-style), torrents, and XMPP inspired the adapter set. They are not the v1 wire above.

## Unresolved questions

Normative config file name (`prayers.toml` vs flags). Artifact sync on_demand versus mirror_all defaults. Whether SSH native distribution is a transport or a git source with host-key pinning. Whether install HTTP should call `HttpTransport` or stay in `pray-core` until a crate-split RFC (RFC 0101).
