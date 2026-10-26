# Skip published npm gem jsr versions on release-all

## Participants

Andrei Makarov

## Decisions

npm.sh, jsr.sh, and rubygems.sh skip when that package version is already on the registry. make release-all can resume after crates.io, npm, jsr.io, or RubyGems succeed. Registry lookup uses HTTP 200 from the package version URL. Publish races that still hit an already-published error also skip.

## Effects

PRAY_RELEASE_YES=1 make release-all did not check that npmjs pray-cli 1.23.0 was already released. crates.io already skipped published versions; npm, jsr.io, and RubyGems did not.

## Next

Resume PRAY_RELEASE_YES=1 make release-all after this change lands. Confirm npm, jsr.io, and RubyGems print skip for 1.23.0 when those versions remain published.

## Source

scripts/release/npm.sh
scripts/release/jsr.sh
scripts/release/rubygems.sh
scripts/release/registry_status.sh
docs/issues/20260918152600_skip-published-crate-versions.md
docs/changelogs/20261026135454_skip-published-npm-gem-jsr-versions.md
