# Publish torrent descriptor on Ruby and TypeScript

## Participants

Andrei Makarov

## Decisions

Ruby and TypeScript publish write {artifact}.praytorrent.json when v1/distribution.json lists protocols torrent. Missing file or empty protocols write none. Remote HTTP publish GETs that file and PUTs the descriptor only when torrent is listed. Republish of an unchanged version rewrites the descriptor if torrent is newly listed.

only and except on local trees stay out of RFC 0115.

## Effects

Ruby TorrentManifest and TypeScript torrent-manifest.ts build pray-torrent-v1 with 16KiB piece hashes. CHANGELOG Unreleased no longer says Rust CLI only. RFC 0062 implementation notes name all three CLIs.

Confirming checks: bundle exec rspec spec/pray/publish_spec.rb spec/pray/torrent_manifest_spec.rb spec/pray/distribution_spec.rb (13 examples, 0 failures). biome check src/publish (0 errors). tsc -p tsconfig.test.json then node --test dist/publish/index.test.js dist/publish/torrent-manifest.test.js dist/distribution.test.js (12 pass, 0 fail). bundle exec rubocop on changed Ruby files (no offenses). make loc-check (157 warnings, 0 failures).

## Next

SSH publish torrent write remains on the Rust CLI.

## Source

usr/docs/issues/20260916184000_distribution-sidecar-opt-in.md
usr/docs/changelogs/20260916194500_cli-parity-alignment.md
rfcs/0062-distribution-protocols.md
CHANGELOG.md 1.17.0
