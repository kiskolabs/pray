# Engineering audit iteration 2 on deferred security batch

## Participants

Andrei Makarov

## Decisions

Run a second multi-model engineering audit after implementing the deferred items from docs/issues/20261021220000_engineering-audit-1-22-release.md. Fact-check high-certainty findings and ship the smallest fixes in the same Unreleased train.

Implement the iteration-2 residuals in the same Unreleased train rather than deferring them again.

## Effects

Addressed in the first pass of this iteration:

- Ruby and TypeScript git catalog unpack now use the praypkg tar validators instead of system `tar -x`.
- TypeScript `spawnSync` sets `maxBuffer` to the archive and blob ceilings and treats `ENOBUFS` as an integrity failure.
- `pray doctor` redacts URL userinfo; tests cover quiet and redaction.
- Ruby and TypeScript recovery hints pass offline state and cover `git_cache_stale`.

Addressed in the residual pass:

- Size probes via `git cat-file -s` and `git ls-tree -r -l` before buffering `git show` / `git archive` bodies.
- Git cache lock waits up to about 60s, writes an owner stamp, clears stamps older than 10 minutes, and names the lock path in the timeout message.
- Catalog file fill takes the catalog lock, re-checks under the lock, and publishes blobs through a partial rename.
- Catalog tree materialize re-checks `catalog_matches` under the lock.
- Federation HTTP and torrent manifests refuse absolute artifact URLs; RFC 0104 experimental wire text matches.
- Marker readers accept only hex revisions and shared bares under the global git cache root.
- Update, outdated, and would-conflict status writers honor global quiet in Rust, Ruby, and TypeScript.

## Next

None for this residual list. Open a new live-work note only if a later audit finds a distinct gap.

## Source

docs/issues/20261021220000_engineering-audit-1-22-release.md

Audit agents: git cache security, quiet and recovery tests.
