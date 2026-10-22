# Audit residuals: size probes, lock recovery, quiet listings

## Participants

Andrei Makarov

## Decisions

Ship the iteration-2 residual list in Unreleased with the deferred security batch rather than opening a separate cut.

## Effects

Catalog materialize probes sizes before buffering git stdout. Locks recover after a ten-minute owner stamp and wait longer for live holders. Catalog readers take the materialize lock and publish blobs atomically. Federation and torrent absolute artifact URLs are refused. Marker readers require a hex revision under the global git cache. Update and outdated status lines honor quiet.

## Next

None for this residual list.

## Source

docs/issues/20261021224500_engineering-audit-deferred-iteration-2.md
