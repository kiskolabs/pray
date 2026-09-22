# Git subdir cache and fetch bytes

## Participants

Andrei Makarov

## Decisions

Project git cache identity is clone URL plus subdir. Empty subdir stays URL-only. The shared global cache stays URL-only so a later subdir can seed from the first clone.

Do not add tar or zstd to pray-bench. Artifact-catalog byte measurement lives in pray-core tests that already pack praypkg files.

Do not switch clone to blob:none in this pass. The measurement is the gate.

TypeScript and Ruby use the same cache identity.

## Effects

Two sources that share a git URL and name different subdir values keep separate project worktrees. Packages from both subdirs resolve. Order left, then right, then left again does not hide the first subdir.

A used git catalog that stores an unused incompressible artifact still clones that blob. Measured 2026-09-17 on Darwin arm64, debug, file:// catalogs:

- unused 256 KiB: clone cache 555873 bytes
- unused 512 KiB: clone cache 1080239 bytes
- unused 1 MiB: clone cache 2128977 bytes
- extra 1 MiB blob then refresh: fetch delta 1050020 bytes

Clone cache stays above half the unused artifact. Fetch of a later unused blob is about one-to-one with that blob. Sparse-checkout does not cut first-clone bytes.

## Next

Partial clone and blob:none stay behind a later pass. Trust import-repo lookup and shared object stores for subdir worktrees moved to usr/docs/changelogs/20260917171000_git-worktree-share-and-import-repo.md.

## Source

usr/docs/issues/20260917154800_multi-source-prayfile-efficiency.md

crates/pray-core/src/resolve_git_paths.rs, resolve_git.rs

crates/pray-core/tests/git_source_subdir.rs, git_source_fetch_bytes.rs

npmjs/pray-cli/src/git/cache.ts

rubygems/pray-cli/lib/pray/git_sources.rb
