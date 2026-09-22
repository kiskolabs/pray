# Ruby and TypeScript publish skip torrent descriptor

## Participants

Andrei Makarov

## Decisions

Match Rust publish: write .praytorrent.json when protocols includes torrent. Skip when the file is missing or protocols is empty. Copy bootstrap_trackers into descriptor trackers. RFC 0061 required-descriptor check applies only when torrent is listed.

Do not add only or except on local trees. RFC 0115 stays Rejected.

## Effects

Ruby and TypeScript publish --root write .praytorrent.json when protocols includes torrent. Empty protocols skip it. Republish after listing torrent writes a missing descriptor.

Confirming checks: bundle exec rspec spec/pray/publish_spec.rb spec/pray/torrent_manifest_spec.rb spec/pray/distribution_spec.rb (13 examples, 0 failures). biome check src/publish (0 errors). tsc -p tsconfig.test.json then node --test dist/publish/index.test.js dist/publish/torrent-manifest.test.js dist/distribution.test.js (12 pass, 0 fail). make loc-check (157 warnings, 0 failures).

## Next

SSH publish torrent write remains on the Rust CLI.

## Source

usr/docs/issues/20260916184000_distribution-sidecar-opt-in.md
rfcs/0062-distribution-protocols.md
crates/pray-cli/src/registry_ops.rs
