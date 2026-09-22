# VCS-backed revisioning and optional remote storage for prayers

## What changed
- Added revision hooks to `pray publish` and `pray sync` so managed roots can be recorded through a configured Git, Mercurial, or command-based backend.
- Auto-detects Git and Mercurial roots when no explicit revision config is present.
- Supports optional remote push for Git and Mercurial backends, and arbitrary configured commit/push commands for custom backends.
- Added integration tests for Git push, Mercurial commit, and custom command-backed sync revisioning.

## Why this matters
Teams often want prayer content to live in the same history system as the rest of the repository. This keeps review, rollback, and remote replication in the normal VCS workflow instead of inventing a separate storage layer.

## Validation
- `cargo test -p pray --test revision`
