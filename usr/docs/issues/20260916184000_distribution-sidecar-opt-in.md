# Distribution torrent sidecar is opt-in

## Participants

Andrei Makarov

## Decisions

v1/distribution.json on the distribution root lists swarm sidecars. Missing file or sidecars: [] writes none. sidecars: ["torrent"] writes .praytorrent.json. trust.json stays authentication. No transport plugins.

pray repo init writes empty sidecars. DHT stays unimplemented. Trackers are written only when that file lists them with torrent enabled.

pray serve returns 404 for a missing static file so install and remote publish can treat a missing sidecar or distribution.json as absent. A Range header on a 200 body becomes 206.

Install still prefers a present sidecar, verifies piece hashes, and accepts a full 200 body when the origin ignores Range.

## Effects

Before: every pray publish --root wrote .praytorrent.json. pray serve returned 500 for a missing sidecar or distribution.json.

After: default publish does not write the sidecar. Opt-in writes it. Missing static GET is 404. Piece fetch works when Range is honored and when it is ignored.

Confirming checks: cargo test --offline -p pray-core --test distribution (6 passed). cargo test --offline -p pray-core --test registry torrent (2 passed). cargo test --offline -p pray-core --test rfc_ids (5 passed). cargo test --offline -p pray-core --test schema_validation distribution_config (1 passed). cargo test --offline -p pray-cli --test install_publish publish_skips_torrent (1 passed). cargo test --offline -p pray-cli --test install_publish publish_writes_torrent (1 passed). cargo test --offline -p pray-cli --test install_publish publish_recovers_after_server (1 passed). cargo test --offline -p pray-cli --test init_commands repo_init (1 passed). cargo test --offline -p pray-cli --bin pray apply_byte_range (2 passed). cargo test --offline -p pray-cli --bin pray missing_static (1 passed). cargo clippy --offline -p pray-core -p pray-cli --all-targets -- -D warnings (exit 0). cargo fmt --all -- --check (exit 0). make loc-check (152 warnings, 0 failures).

A later pass renamed the JSON field from sidecars to protocols. See usr/docs/issues/20260916202600_distribution-protocol-names.md.

## Next

Whether ipfs is the next allowlist name. Whether private roots must refuse non-loopback tracker URLs. DHT seeding stays unimplemented.

## Source

usr/docs/issues/20260916180900_distribution-torrent-sidecar-opt-in.md
rfcs/0062-distribution-protocols.md
rfcs/0060-distribution.md
rfcs/0104-federation-transports.md
crates/pray-core/src/distribution.rs
crates/pray-cli/src/publish.rs
crates/pray-cli/src/server_static.rs
crates/pray-cli/src/server_listen.rs
crates/pray-core/src/registry_torrent.rs
CHANGELOG.md 1.17.0
usr/docs/changelogs/20260916184000_distribution-sidecar-opt-in.md
