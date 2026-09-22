# Distribution protocol names

## Participants

Andrei Makarov

## Decisions

Sidecar names a helper process. The distribution.json field is protocols. The files next to .praypkg are protocol descriptors. transports stays RFC 0104 for HTTP, SSH, git, and federation wire.

v1 allowlist stays torrent only. Later names on the same field: ipfs, iroh, metalink, blossom. BitTorrent v2 and WebTorrent stay under torrent. Syncthing, GNUnet, Hyphanet, Tahoe-LAFS, Hypercore, Radicle, Tor, I2P, and OCI stay RFC 0104 transports. Chat adapters, Filecoin, and Arweave stay off this allowlist.

A leftover sidecars field fails parse. Missing file still means no extra protocols.

## Effects

v1/distribution.json uses protocols: []. protocols: ["torrent"] writes .praytorrent.json.

cargo test -p pray-core --test distribution --test schema_validation: 17 passed.
cargo test -p pray-cli --test init_commands --test install_publish -- publish_skips_torrent publish_writes_torrent repo_init: 3 passed.
cargo clippy -p pray-core -p pray-cli --all-targets -- -D warnings: finished.
bundle exec rspec spec/pray/distribution_spec.rb: 6 examples, 0 failures.
node --test dist/distribution.test.js: 6 passed.

## Next

Whether ipfs is the next allowlist name. Private-root tracker URL policy and DHT seeding stay open.

## Source

rfcs/0062-distribution-protocols.md
usr/docs/issues/20260916184000_distribution-sidecar-opt-in.md
schema/distribution.schema.json
