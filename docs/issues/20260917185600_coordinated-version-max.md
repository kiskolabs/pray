# Coordinated version is max, skip one-surface cuts

## Participants

Andrei Makarov

## Decisions

Next version crates.io, npm, and RubyGems may share is the max of workspace, npm, and gem. TypeScript PACKAGE_VERSION must match npm; it is not a fourth surface. A max held by one surface only cannot be aligned, whether that cut was a patch or a minor. The other registries skip that number and jump to the next patch.

1.18.1 is a Ruby-only example. An npm-only or crates-only minor is skipped the same way. Next coordinated cut after the current tree is 1.18.2 or later. Do not publish the other registries as 1.18.1.

Later pass 20260917191000: one-surface skip is not Ruby-only.

## Effects

scripts/release/versions.sh computes that max and the skip for three surfaces. crates.sh, npm.sh, and rubygems.sh may publish when that surface is uniquely ahead. A shared publish refuses a version missing from the root, npm, or Ruby changelog.

./scripts/release/versions_test.sh printed coordinated version tests passed, including gem-only 1.18.1 to 1.18.2, npm-only 1.19.0 to 1.19.1, crates-only 1.19.0 to 1.19.1, and live next coordinated version 1.18.2. ./scripts/release/changelog_test.sh printed changelog notes tests passed. make loc-check printed 0 failure(s).

## Next

Bump all three to 1.18.2 or later on the next shared release.

## Source

docs/releasing.md
scripts/release/versions.sh
scripts/release/crates.sh
scripts/release/npm.sh
scripts/release/rubygems.sh
usr/docs/issues/20260917185100_prepare-1-18-1-release.md
