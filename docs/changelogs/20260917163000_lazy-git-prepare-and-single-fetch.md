# Lazy git prepare and single fetch

## Participants

Andrei Makarov

## Decisions

Prepare a git source when the first package needs it, including transitives discovered during resolve. Unused git catalogs are not cloned or fetched.

On refresh, fetch origin into the project cache once, then rebuild the shared git cache from that worktree. Do not fetch origin a second time into the shared cache.

TypeScript and Ruby follow the same prepare-on-need and single-origin-fetch rules.

## Effects

A Prayfile with a path package and unused git sources no longer clones those catalogs.

A used git source still clones. Warm update fetches origin once per used source.

Unused git source scaling (tiny file:// catalogs, debug, Darwin arm64, 2026-09-17): 2 to 8 sources stayed on the path-package path. Cold 1.3 ms to 0.67 ms. Warm 0.20 ms to 0.51 ms. Peak RSS 7.4 MiB. Before this pass the same fixture paid about 80 ms cold and 120 ms warm per unused catalog.

Stale shared-cache seed still refreshes from the clone URL on update and on install when the constraint needs a newer catalog.

## Next

Fetch-byte measurement and same-URL different-subdir cache identity moved to usr/docs/changelogs/20260917165500_git-subdir-cache-and-fetch-bytes.md.

## Source

usr/docs/issues/20260917154800_multi-source-prayfile-efficiency.md

crates/pray-core/src/resolve_git_source_set.rs, resolve_git.rs, resolve_git_sources.rs

npmjs/pray-cli/src/git/sources.ts, git/cache.ts

rubygems/pray-cli/lib/pray/git_sources.rb
