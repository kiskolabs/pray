# RFC 0062: Distribution protocols

- Feature Name: distribution-protocols
- Type: Standards Track
- Status: Proposed
- Created: 2026-09-16
- Author: Andrei Makarov
- Stakeholders: distribution operators
- Feedback until: 2026-09-30
- Relates: RFC 0060, RFC 0061, RFC 0104
- Requires: RFC 0060

## Summary

A distribution root opts in before publish writes a protocol descriptor such as `.praytorrent.json`. The control is `v1/distribution.json`. Missing file or `protocols: []` writes none. `protocols: ["torrent"]` writes the torrent descriptor. DHT stays off. Trackers stay empty unless that file lists them. Install keeps working from static HTTP when the descriptor is absent.

## Motivation

RFC 0060's static layout is index, package JSON, and `.praypkg`. Extra fetch protocols sit beside the artifact as first-party names, not plugins. A Kubernetes-style sidecar is a helper process. These entries are protocol names and descriptor files. RFC 0104 keeps torrent and other extra fetch experimental. Experimental descriptors must not be unconditional on every `pray publish --root`.

Today's torrent fetch is not a public BitTorrent announce. Defaults have no trackers and DHT off. When the descriptor exists, install pulls HTTP Range pieces from the same origin, then verifies piece and artifact hashes.

## Guide-level explanation

`pray repo init` writes:

```json
{
  "spec": "pray-distribution-config-1",
  "protocols": []
}
```

A private root stays in that shape. A public root that wants piece fetch sets `"protocols": ["torrent"]`.

`pray publish --root` and HTTP or SSH publish read that file on the destination root. They write `.praytorrent.json` only when `torrent` is listed. Clients do not set a Prayfile flag. HTTP 404 on the descriptor uses a normal artifact GET.

To list trackers, add `bootstrap_trackers` on the same file. DHT announce is not implemented. `enable_dht: true` fails.

## Reference-level explanation

`v1/distribution.json` is root protocol policy, not `v1/trust.json`. Trust stays authentication.

Allowed protocol names are a first-party allowlist. v1 allows `torrent` only. Unknown names fail parse. Duplicate `torrent` means enabled once. No executable plugins.

Missing file: treat as `protocols: []`. Present file: `spec` MUST be `pray-distribution-config-1`. `protocols` defaults to `[]`. `bootstrap_trackers` defaults to `[]`. `enable_dht` defaults to false.

`bootstrap_trackers` or `enable_dht: true` without `torrent` in `protocols` fails. `enable_dht: true` fails even with torrent listed. Non-empty `bootstrap_trackers` with torrent listed are copied into the descriptor `trackers` array. Descriptor `sources` stay relative artifact paths on the same origin.

When `torrent` is listed, publish writes `{artifact}.praytorrent.json` with spec `pray-torrent-v1`, piece hashes, artifact hash, and length. HTTP Range `bytes=start-end` on the artifact MUST return 206 and `Content-Range` when the range is satisfiable. Install prefers the descriptor when GET succeeds, verifies each piece hash and the artifact hash, and MUST accept a 200 full body as the whole artifact when a server ignores Range, using that body for every piece without repeating the GET. Descriptor GET 404 uses one artifact GET. Descriptor GET success with a later piece failure does not skip hash verify.

RFC 0061 required-descriptor check applies only when `torrent` is listed. When it is not listed, an unchanged version MAY be current without a descriptor file.

Remote publish GETs `{origin}/v1/distribution.json`. HTTP 404 means no extra protocols. A parse error fails the publish. The torrent descriptor is written only when the parsed list contains `torrent`.

Every later name on this field MUST keep identity as RFC 0050 `artifact_hash`. A missing descriptor stays a normal artifact GET. Private roots MUST NOT announce off-root (DHT, trackers, relays, public gateways as bootstrap) unless this file lists them.

## Implementation notes

`pray-core` `distribution.rs` parses the file. `pray-cli` `publish.rs` and `commands_init.rs` write it. Ruby and TypeScript `repo init` write empty `protocols`. Serve applies Range after static GET. Artifact torrent descriptor write on publish is implemented in the Rust, Ruby, and TypeScript CLIs.

## Security considerations

Default off is fail-closed for extra protocol metadata. Trackers in the file are an explicit off-root announce. Hash verify is unchanged from RFC 0050 and RFC 0060.

## Registrar

File `v1/distribution.json`. Spec `pray-distribution-config-1`. Field `protocols`. Protocol name `torrent`. Artifact suffix `.praytorrent.json`.

## Drawbacks

Roots that relied on the previous always-on torrent descriptor must set `protocols: ["torrent"]`. Piece fetch remains experimental with RFC 0104.

## Rationale and alternatives

`sidecars` was rejected: it names a helper process, not a protocol. `transports` was rejected because RFC 0104 already uses that word for HTTP, SSH, git, and federation wire. This field is optional fetch protocols next to `.praypkg`. Plugins were rejected. `trust.json` was rejected because it is auth. A Prayfile consumer flag was rejected because 404 already works.

## Prior art

BitTorrent `.torrent` next to a payload is origin-local metadata. npm provenance and crates.io `.crate` do not emit extra protocol files unless the registry says so.

## Unresolved questions

Whether `ipfs` is the next allowlist name. Whether private roots must refuse tracker URLs that are not loopback. Whether DHT seeding ever ships.

## Future possibilities

v1 writes `torrent` only: origin-local piece SHA-256 plus HTTP Range. BitTorrent v2, WebTorrent, and magnet URIs stay under that name if swarm announce ships later.

Same field, later names, same private-root rule:

- `ipfs`: CIDv1 plus optional `.car` on the root, DHT off by default. Freeze a UnixFS profile or use a raw-codec CID of the whole `.praypkg` so the CID matches `artifact_hash`.
- `iroh`: BLAKE3 blob hash plus ticket or relay list. Relays default off. Identity stays `sha256:`; BLAKE3 is the fetch key.
- `metalink`: RFC 5854 hashes and HTTP or FTP origins. Extra origins, not a swarm.
- `blossom`: BUD-01 GET `/<sha256>` plus a server list. Matches `artifact_hash` with no extra merkle tree.

RFC 0104 transports, not this field: Syncthing BEP, GNUnet FS, Hyphanet, Tahoe-LAFS, Hypercore, Radicle, Tor, I2P, OCI or ORAS. Chat-channel adapters, Filecoin, and Arweave stay out of this allowlist.
