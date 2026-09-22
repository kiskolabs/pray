# Coordinated version is max, skip one-surface cuts

## Participants

Andrei Makarov

## Decisions

Encode max-of-all as the next shared version. Skip a one-surface max for the other registries, whether that max is a patch or a minor, and whether it is gem, npm, or crates.

Later pass 20260917191000: one-surface skip is not Ruby-only.

## Effects

release_next_coordinated_from takes crates, npm, and gem. Gem-only 1.18.1 yields 1.18.2. Npm-only or crates-only 1.19.0 yields 1.19.1. Two surfaces at the max keep that number so the third can catch up.

./scripts/release/versions_test.sh printed coordinated version tests passed.

## Next

Use 1.18.2 or later for the next shared publish.

## Source

usr/docs/issues/20260917185600_coordinated-version-max.md
docs/releasing.md
