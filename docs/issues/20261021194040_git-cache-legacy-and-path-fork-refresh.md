# Git cache legacy coexistence and path-fork refresh UX

## Participants

Andrei Makarov

## Decisions

Treat two failure modes as separate products.

Mode A is git source cache layout coexistence: a project-local full clone under `.pray/cache/git` can sit beside a stub catalog and shared bare. Lookup must not select that leftover `.git` for trust or object resolution. Install and update quarantine matching full clones by renaming them to `*.legacy-git` before writing the stub. Materialize refuses a missing revision with a pray-level `git_revision_unfetched` or `git_cache_stale` message instead of raw git fatals alone. Catalog match also requires `cat-file -e` on the locked revision so a stub cannot skip rematerialize when objects are gone.

Mode B is path-fork three-way merge policy. Branded overlays that rewrite README and fragment text conflict on meaningful upstream bumps. That is intentional fail-closed merge, not cache corruption. Conflict messages and `pray update --help` now name the operator workflow: reset conflicting content to the locked upstream, run `pray update --latest`, re-apply overlays, bump the fork version. Error text includes `upstream_merge_conflict`.

Exact field argv `git checkout --force <rev>` was not present in 1.21.0 sources. Current materialize uses `git archive` and `git checkout <rev> -- <path>`.

Shallow plus partial clones (`--depth 1 --filter=blob:none`) stay the long-term default for now. Ensure/fetch before materialize remains the mitigation. Fuller clone policy and `pray doctor --migrate-git-cache` stay open.

## Effects

Rust, Ruby, and TypeScript CLIs stop scanning project `.pray/cache/git` for leftover full clones. Trust and diagnostics use the ready shared bare or a stub `.pray-git-dir`. Install quarantines leftover full clones for the same URL. Missing revision and failed lazy checkout print recovery hints.

Operator surface assessed: `pray update --help` and upstream merge conflict stderr. Hierarchy is action, usage, path-fork refresh note, then flags. A person can finish a branded fork refresh from that help plus the conflict message. Remaining for a human: whether to add `pray doctor` layout listing and dry-run would-conflict path simulation.

Visual surface: none beyond CLI help and error text.

Workarounds remain valid.

Mode A:

```
rm -rf .pray/cache/git
pray install --locked
pray verify
```

Also remove the shared bare entry under the machine git cache when the project wipe is not enough.

Mode B: reset conflicting content files to the locked upstream tree, run `pray update --latest`, re-apply overlays, bump fork `spec.version`, then install/apply/verify.

## Next

Backlog items moved to docs/issues/20261021220000_git-cache-concurrency-offline-and-update-simulation.md and implemented there, except branding rename map (still needs RFC).

## Effects note

Follow-up from engineering audit and Mode B fact-check: a fork that deleted an upstream file no longer gets that file silently restored when upstream later changes it. That case is a merge conflict. Both sides deleting the same path omits it. Trust lookup no longer takes leftover project full clones (already shipped in this issue).

## Source

Maintainer report: pray cache migration and path-fork refresh UX (session 2026-10-21).

Fact-check Mode A: agent transcript 86edbc1e-b98a-44e9-bc6e-72defdad1ed6.

Fact-check Mode B: agent transcript f38a965a-07c9-4d81-8fc2-3d2589f0b482.

Engineering audit: agent transcript c603befa-b354-4bd3-891b-a4e29f229925.

RFC 0114 package upstream. RFC 0116 path-fork first material.

docs/changelogs/20260918213000_git-free-catalog-and-unshallow.md

crates/pray-core/src/resolve_git_lookup.rs

crates/pray-core/src/resolve_git_legacy.rs

crates/pray-core/src/resolve_git_materialize.rs

crates/pray-core/src/package_upstream.rs
