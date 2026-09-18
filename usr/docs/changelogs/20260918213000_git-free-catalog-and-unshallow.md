# Git-free project catalog and unshallow pin fetch

## Participants

Andrei Makarov

## Decisions

Keep git objects only in the global bare store under PRAY_CACHE/git. Materialize the project catalog as a git-free tree of v1/packages, prayers/v1/packages, or {subdir}/v1/packages, plus .pray-revision and .pray-git-dir markers. Used artifacts still check out from the bare store into that tree.

When a lock pin is missing from a shallow global store, fetch --unshallow. A depth-1 fetch of the pin SHA remains only if the pin is still missing after unshallow. Trust and pray trust import-repo use the bare store, including for subdir sources.

No new RFC. RFC 0070 names the registry cache, not git working trees. RFC 0020 still covers lock pins, install fetch, and --offline.

## Effects

Project .pray/cache/git no longer keeps a .git directory. An existing worktree there is deleted on rematerialize.

Operator surface: none new. Install and update still print the same resolve errors. A person browsing .pray/cache/git now sees package metadata files, not a git checkout. Remaining for a human: whether to document PRAY_CACHE layout in docs/.

Visual surface: none. This pass has no screen, document preview, or other person-facing layout.

Tests: crates/pray-core/tests/git_source_catalog_layout.rs, crates/pray-core/tests/git_source_pinned_fetch.rs, crates/pray-core/tests/git_source_offline.rs, crates/pray-core/tests/git_source_prepare.rs, crates/pray-core/tests/git_source_subdir.rs, npmjs/pray-cli/src/git-clone.test.ts, npmjs/pray-cli/src/git-pinned-fetch.test.ts, rubygems/pray-cli/spec/pray/git_clone_spec.rb, rubygems/pray-cli/spec/pray/git_pinned_fetch_spec.rb.

Confirming checks: cargo test -p pray-core --offline -- --test-threads=1 (full crate passed, including git_source_catalog_layout 2 passed, git_source_pinned_fetch 2 passed, git_source_offline 4 passed, git_source_prepare 3 passed, git_source_subdir 2 passed, git_source_fetch_bytes 1 passed 1 ignored). cargo test -p pray-cli --offline --test install_git_catalog (2 passed) --test install_git_global_cache (2 passed). cargo clippy -p pray-core --offline --tests -- -D warnings. cargo fmt --all -- --check. bundle exec rspec spec/pray/git_pinned_fetch_spec.rb spec/pray/git_clone_spec.rb spec/pray/git_sources_spec.rb (10 examples, 0 failures). bundle exec rspec spec/pray/git_distribution_spec.rb (4 examples, 0 failures). bundle exec rubocop on the changed Ruby git files (no offenses). biome check on the changed TypeScript git files (clean after write). tsc -p tsconfig.test.json and node --test dist/git-clone.test.js dist/git-pinned-fetch.test.js dist/git-source-cache.test.js dist/git-refresh.test.js (9 passed). node --test dist/git-catalog.integration.test.js dist/git-distribution.integration.test.js (4 passed). make loc-check (164 warnings, 0 failures).

## Next

Ship in 1.20.0. Close GitHub issue 31 after release.

PRAY_CACHE is still process environment, so parallel test binaries can race that variable. Tests use a mutex inside one binary and unique cache roots.

Leftover project worktrees with .git remain a fallback for pray trust import-repo when the global store is missing.

## Source

usr/docs/issues/20260917190500_git-cache-without-dot-git.md

usr/docs/issues/20260918205500_bundler-cargo-git-cache.md

usr/docs/issues/20260918204000_install-pinned-git-revision-shallow-cache.md

rfcs/0020-resolve-and-lock.md

rfcs/0070-reference-implementation.md

crates/pray-core/src/resolve_git_store.rs

crates/pray-core/src/resolve_git_materialize.rs

npmjs/pray-cli/src/git/store.ts

rubygems/pray-cli/lib/pray/git_store.rb
