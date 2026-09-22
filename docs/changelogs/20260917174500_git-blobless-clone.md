# Git blobless catalog clone

## Participants

Andrei Makarov

## Decisions

Clone a used git catalog with depth 1, blob:none, and sparse checkout of package metadata. Fetch a used artifact into the worktree when install reads that file. Leave unused catalog blobs on the origin.

Do not pass blob:none to a local bare copy of the project cache into the global cache. That command hung in tests.

Fall back to a plain depth-1 clone when the origin rejects a filter clone.

TypeScript and Ruby follow the same clone, sparse metadata, fetch, and materialize rules.

## Effects

cargo test -p pray-core --offline --test git_source_fetch_bytes -- --nocapture, Darwin arm64, debug, incompressible unused blobs next to a tiny used package:

- unused 512 KiB: clone cache 34234 bytes, used package still resolves

cargo test -p pray-core --offline --test git_source_fetch_bytes -- --ignored --nocapture:

- unused 256 KiB then 1 MiB: clone cache 34235 then 34237 bytes
- extra 1 MiB blob on refresh: fetch delta 2812 bytes

Clone cache no longer grows with unused artifact size. Refresh does not fetch an unused blob.

## Next

Co-location with other processes on the same machine was not measured.

## Source

usr/docs/issues/20260917154800_multi-source-prayfile-efficiency.md

crates/pray-core/src/resolve_git_clone.rs, resolve_git_materialize.rs, resolve_git.rs, resolve_git_ensure.rs

crates/pray-core/tests/git_source_fetch_bytes.rs

npmjs/pray-cli/src/git/clone.ts, git/materialize.ts, git/cache.ts

rubygems/pray-cli/lib/pray/git_clone.rb, git_materialize.rb, git_cache.rb
