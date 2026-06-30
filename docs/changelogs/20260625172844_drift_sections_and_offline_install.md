# Drift sections and offline install

Implemented the next reference CLI iteration:

- `pray install --offline` now accepts explicit local path packages and rejects derived package paths
- `pray drift` now groups findings into lockfile, package, managed span, rendered file, and warning sections
- added regression tests for renderer drift and position drift reporting
