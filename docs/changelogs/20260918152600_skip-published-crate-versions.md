# Skip published crate versions on release-all

## Participants

Andrei Makarov

## Decisions

Skip cargo publish when cargo info --registry crates-io reports the crate version URL. Treat the cargo already-exists index message as skip if publish still races.

## Effects

./scripts/release/crates_status_test.sh printed crates status tests passed. Live cargo info --registry crates-io treated pray-core 1.19.0 as published and pray-core 0.0.0 as missing.

## Next

Resume PRAY_RELEASE_YES=1 make release-all.

## Source

scripts/release/crates.sh
scripts/release/crates_status.sh
usr/docs/issues/20260918152600_skip-published-crate-versions.md
