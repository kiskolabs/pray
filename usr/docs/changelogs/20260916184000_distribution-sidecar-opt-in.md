# Distribution sidecar opt-in

## Participants

Andrei Makarov

## Decisions

Publish writes .praytorrent.json only when v1/distribution.json lists torrent. Repo init defaults to empty sidecars. Serve returns 404 for missing files and 206 for Range.

## Effects

Private roots no longer emit swarm metadata. Opt-in piece fetch still verifies hashes. A server that ignores Range still installs from one full body.

## Next

Covered by RFC 0062 unresolved questions.

## Source

usr/docs/issues/20260916184000_distribution-sidecar-opt-in.md
rfcs/0062-distribution-protocols.md
CHANGELOG.md 1.17.0
