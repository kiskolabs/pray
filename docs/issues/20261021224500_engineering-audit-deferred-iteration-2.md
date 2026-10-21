# Engineering audit iteration 2 on deferred security batch

## Participants

Andrei Makarov

## Decisions

Run a second multi-model engineering audit after implementing the deferred items from docs/issues/20261021220000_engineering-audit-1-22-release.md. Fact-check high-certainty findings and ship the smallest fixes in the same Unreleased train.

## Effects

Addressed in this iteration:

- Ruby and TypeScript git catalog unpack now use the praypkg tar validators instead of system `tar -x`.
- TypeScript `spawnSync` sets `maxBuffer` to the archive and blob ceilings and treats `ENOBUFS` as an integrity failure.
- `pray doctor` redacts URL userinfo; tests cover quiet and redaction.
- Ruby and TypeScript recovery hints pass offline state and cover `git_cache_stale`.

Confirmed still open (do not block this batch without separate notes):

- Size checks after full process buffering for `git archive` and `git show`.
- Lock directory stale recovery and longer clone wait.
- Catalog readers skipping the materialize lock; non-atomic mid-swap window.
- Federation and torrent absolute artifact URLs.
- Marker readers that do not re-check the expected bare path.
- Remaining quiet writers on update and outdated listings.

## Next

Open separate live-work notes only for residuals that product wants in the next cut.

## Source

docs/issues/20261021220000_engineering-audit-1-22-release.md

Audit agents: git cache security, quiet and recovery tests.
