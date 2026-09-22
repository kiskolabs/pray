# GitHub Release notes from one CHANGELOG version

## Participants

Andrei Makarov

## Decisions

Ship a Makefile target that creates or edits a GitHub Release with title vX.Y.Z and notes from that version's CHANGELOG.md section only. v1.17.0 had an empty name, which Discord rendered as Release - undefined, and pasted the whole CHANGELOG as the body. Keep --all as edit-only so missing historical tags do not publish new releases.

## Effects

Added scripts/release/github.sh, scripts/release/changelog.sh, and scripts/release/changelog_test.sh. ./scripts/release/changelog_test.sh printed changelog notes tests passed. make release-github publishes the workspace version. make release-github-sync rewrites existing GitHub Releases. CI runs changelog_test.sh.

PRAY_RELEASE_YES=1 ./scripts/release/github.sh --publish --all exited 0. It edited 16 GitHub Releases (v1.17.0 through v1.2.1) and skipped CHANGELOG versions with no GitHub Release (v1.6.0, v1.5.2, v1.5.1, v1.5.0, v1.4.0, v1.3.0, v1.2.0, v1.1.0, v1.0.0). gh release view v1.17.0 now has name v1.17.0 and an 11-line body that starts with ## 1.17.0 (2026-09-16) and contains one version heading.

## Next

Use make release-github after the next tag. Leave tags without GitHub Releases unpublished unless a later pass asks to create them.

## Source

CHANGELOG.md
docs/releasing.md
scripts/release/README.md
usr/docs/changelogs/20260904154800_backfill-v1-9-1-and-v1-10-0.md
