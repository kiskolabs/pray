# Distribution-root torrent sidecar should be opt-in

## Participants

Andrei Makarov

## Decisions

Treat .praytorrent.json as an artifact sidecar on the distribution root, not as RFC 0033 provisioned dest files in a consumer project.

Publish always writes that sidecar today. That is not required by RFC 0060 static layout. RFC 0104 keeps torrent experimental. A distribution root SHOULD opt in before any swarm sidecar is written. Private roots default off.

Do not add executable transport plugins for this. Named first-party sidecar writers on the root config are enough. Other networks later are more names on the same list, not dlopen.

Do not implement the config in this note.

## Effects

Outcome: partially supported. The target is publish sidecars on the distribution root, not consumer file: or tree: provisioning.
Source: crates/pray-cli/src/publish.rs write_torrent_manifest on every root publish; crates/pray-cli/src/publish.rs upload of torrent_manifest_path on HTTP publish; rfcs/0033-provisioned-destination-safety.md; rfcs/0060-distribution.md recommended layout (index, package json, .praypkg only).

Outcome: supported as a missing control. Unsupported as already configurable.
Source: crates/pray-cli/src/registry_ops.rs write_torrent_manifest always; TorrentConfig::default used only for piece_size and empty bootstrap_trackers; v1/trust.json is auth policy (crates/pray-core/src/trust.rs), not sidecar policy; pray repo init writes index and trust only (crates/pray-cli/src/commands_init.rs).

Outcome: partially supported today. Supported as a fail-closed default before swarm announce exists.
Source: crates/pray-transport/src/torrent.rs TorrentConfig default enable_dht false, bootstrap_trackers empty; crates/pray-core/src/registry_torrent.rs fetch uses HTTP Range on the registry origin when sources are relative; crates/pray-core/src/fetch.rs prefers the sidecar when GET succeeds, else HTTP GET of the artifact.
Notes: Today's sidecar is piece hashes next to the artifact. Trackers are empty. Pieces are Range GETs of the same host. That is not a public DHT announce. It still publishes package name, version, size, and hashes at a guessable URL. If that URL is reachable without the same auth as the .praypkg, that is a leak. If trackers, DHT, or absolute public sources are later filled from a global default, a private package would be announced off-root. That is the private-root failure. RFC 0060 already wants public, private, local, and optional peer distribution with the same verify rules.

Outcome: supported.
Source: crates/pray-core/src/registry_torrent.rs fetch_torrent_manifest returns none on HTTP 404; crates/pray-core/src/fetch.rs then http_get the artifact; crates/pray-core/tests/registry.rs torrent fallback fixture.

## Next

Add a distribution-root sidecar list. Candidate file: v1/distribution.json next to index.json, not trust.json. trust.json stays authentication. Example field: sidecars: [] by default, sidecars: ["torrent"] to write .praytorrent.json.

pray publish --root and HTTP or SSH publish read that file. Missing file means no swarm sidecars.

When torrent is on, keep DHT and public trackers off unless the same file sets them. Refuse a private root that lists public tracker URLs until an explicit private-swarm field exists.

Install stays hash-verified over HTTP when the sidecar is absent. Do not require consumer Prayfile flags.

Do not ship a plugin ABI for sidecar writers. RFC 0104 transport adapters stay in-tree. A later sidecar name (for example ipfs) is another first-party writer gated by the same list.

Product RFC before the default flips from always-on to off. Current tests expect the sidecar (crates/pray-cli/tests/install_publish.rs publish_writes_torrent_manifest_sidecar_for_registry_artifacts).

Shipped in RFC 0062 and usr/docs/issues/20260916184000_distribution-sidecar-opt-in.md.

## Source

rfcs/0060-distribution.md
rfcs/0104-federation-transports.md
rfcs/0033-provisioned-destination-safety.md
crates/pray-cli/src/publish.rs
crates/pray-cli/src/registry_ops.rs
crates/pray-core/src/fetch.rs
crates/pray-core/src/registry_torrent.rs
crates/pray-transport/src/torrent.rs
usr/docs/issues/20260626183000_torrent_seeding_and_collective_dht_distribution.md
