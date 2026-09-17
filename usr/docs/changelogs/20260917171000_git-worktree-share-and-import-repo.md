# Git worktree share and import-repo

## Participants

Andrei Makarov

## Decisions

Clone once into the URL-only project cache. Subdir sources are linked git worktrees from that clone, each with its own sparse-checkout. The global cache stays URL-only.

pray trust import-repo looks up the URL-only cache first. If that checkout is missing, it scans sibling project git caches for a matching origin URL so an older subdir-only layout still imports.

TypeScript and Ruby follow the same clone, worktree, and lookup rules.

## Effects

Two sources that share a git URL and name different subdir values keep separate working trees and one object store. git rev-parse --git-common-dir matches the URL-only clone.

pray trust import-repo finds that URL-only clone after a subdir-only resolve. A leftover subdir checkout with origin set still matches when the URL-only path is absent.

The URL-only clone still has a full working tree. Linked worktrees do not copy git objects.

## Next

Partial clone and blob:none stay behind a later pass. Co-location with other processes was not measured.

## Source

usr/docs/issues/20260917154800_multi-source-prayfile-efficiency.md

crates/pray-core/src/resolve_git_ensure.rs, resolve_git_lookup.rs

crates/pray-core/tests/git_source_subdir.rs

crates/pray-cli/src/trust_command.rs

npmjs/pray-cli/src/git/cache.ts, git/lookup.ts

rubygems/pray-cli/lib/pray/git_cache.rb
