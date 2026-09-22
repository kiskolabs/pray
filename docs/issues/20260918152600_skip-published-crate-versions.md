# Skip published crate versions on release-all

## Participants

Andrei Makarov

## Decisions

crates.sh skips a crate when that version is already on crates.io. make release-all can continue to later crates, npm, and RubyGems. cargo info --registry crates-io must show a crates.io URL so a local workspace crate does not count as published.

## Effects

PRAY_RELEASE_YES=1 make release-all failed with crate pray-core@1.19.0 already exists on crates.io index.

./scripts/release/crates_status_test.sh printed crates status tests passed. Live cargo info --registry crates-io treated pray-core 1.19.0 as published and pray-core 0.0.0 as missing. make loc-check printed 0 failure(s).

## Next

Resume PRAY_RELEASE_YES=1 make release-all. npm and RubyGems still fail if that version is already published.

## Source

scripts/release/crates.sh
scripts/release/crates_status.sh
usr/docs/changelogs/20260918152600_skip-published-crate-versions.md
