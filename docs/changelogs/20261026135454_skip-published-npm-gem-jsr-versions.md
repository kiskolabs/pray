# Skip published npm gem jsr versions on release-all

## Participants

Andrei Makarov

## Decisions

Skip npm publish, jsr publish, and gem push when the registry already has that version. Treat npm and gem already-published publish errors as skip if a race still hits them.

## Effects

./scripts/release/registry_status_test.sh printed registry status tests passed. Live HTTP probes treated pray-cli 1.23.0 as published on npmjs, RubyGems, and jsr.io, and 9.9.9 as missing. PRAY_RELEASE_YES=1 ./scripts/release/npm.sh --publish printed skip pray-cli@1.23.0 (already on npmjs). The same skip message shape appeared for jsr.io and RubyGems. .github/workflows/publish-jsr.yml skips install and publish when that jsr.json version is already on jsr.io.

## Next

Resume PRAY_RELEASE_YES=1 make release-all; npm, jsr.io, and RubyGems should skip 1.23.0.

## Source

scripts/release/registry_status.sh
scripts/release/registry_status_test.sh
scripts/release/npm.sh
scripts/release/jsr.sh
scripts/release/rubygems.sh
docs/issues/20261026135454_skip-published-npm-gem-jsr-versions.md
