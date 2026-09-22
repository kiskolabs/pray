# Distribution protocol names

## Participants

Andrei Makarov

## Decisions

The opt-in list on v1/distribution.json is protocols, not sidecars. Descriptor files stay next to the artifact. RFC 0062 names later allowlist entries without enabling them.

## Effects

pray repo init writes empty protocols. Publish writes .praytorrent.json only when torrent is listed. A leftover sidecars field fails parse.

## Next

Ships as 1.17.0. See usr/docs/issues/20260916223500_prepare-1-17-0-release.md.

## Source

rfcs/0062-distribution-protocols.md
usr/docs/issues/20260916202600_distribution-protocol-names.md
CHANGELOG.md 1.17.0
