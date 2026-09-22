# GitHub Release notes from one CHANGELOG version

## Participants

Andrei Makarov

## Decisions

Operator GitHub Releases take title vX.Y.Z and the matching CHANGELOG.md heading plus bullets. Empty names are invalid. --all edits existing releases only.

## Effects

scripts/release/github.sh reads CHANGELOG.md. make release-github and make release-github-sync wrap publish. changelog_test.sh covers one-version extract, missing version, and non-empty title.

PRAY_RELEASE_YES=1 ./scripts/release/github.sh --publish --all edited 16 existing GitHub Releases. v1.17.0 name is v1.17.0. Body is the 1.17.0 CHANGELOG section only.

## Next

Use make release-github after the next tag.

## Source

usr/docs/issues/20260917074700_github-release-notes.md
docs/releasing.md
