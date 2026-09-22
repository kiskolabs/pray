# Offline git clone gate and file origin fallback

## Participants

Andrei Makarov

## Decisions

RFC 0020 already says pray install --offline must use cache or vendor only and must not touch the network. Git clone of a missing project cache was still ungated. Cargo fetch_db refuses a missing db when offline. Align with that: clone from origin only when not offline; seed from the global bare cache remains local and is allowed.

A file:// git source whose cache checkout failed then read the origin worktree. That could honour HEAD instead of the lock pin. Cargo has no such fallback. Bundler local override is explicit config. Remove the silent fallback. A file:// path that is a distribution tree without .git still resolves in place.

A file:// origin that is missing used to skip ensure and return unprepared. Fall through to ensure so a global cache seed can still satisfy offline install.

No new RFC.

## Effects

Finding 4 from usr/docs/issues/20260918204000_install-pinned-git-revision-shallow-cache.md: clone gated. Missing project cache and no global seed now errors: git source {url} is not cached locally and offline mode is enabled.

Finding 5: resolve_git_package_root no longer reads the origin worktree after ensure fails.

prepare_one_git_source only uses a local distribution tree when that tree exists. Otherwise it calls ensure_git_repository.

Operator surface: pray install --offline stderr when the git catalog is absent from project and global cache. Hierarchy is one resolution error. Copy names the source URL and offline mode. Interactive state is retry without --offline after a normal install, or restore cache. Remaining for a human: whether printing the clone URL is enough in a CI log that already redacts tokens.

Tests: crates/pray-core/tests/git_source_offline.rs, npmjs/pray-cli/src/git-pinned-fetch.test.ts, rubygems/pray-cli/spec/pray/git_pinned_fetch_spec.rb.

Confirming checks: cargo test -p pray-core --offline (full crate, including git_source_offline 4 passed and git_source_pinned_fetch 2 passed). cargo test -p pray-cli --offline --test install_git_catalog (2 passed) --test install_git_global_cache (2 passed). cargo clippy -p pray-core --offline --tests -- -D warnings. cargo fmt --all -- --check. bundle exec rspec spec/pray/git_pinned_fetch_spec.rb spec/pray/git_sources_spec.rb spec/pray/git_clone_spec.rb spec/pray/git_distribution_spec.rb (14 examples, 0 failures). bundle exec rubocop on the changed Ruby git files (no offenses). biome check on the changed TypeScript git files (clean). tsc -p tsconfig.test.json and node --test dist/git-pinned-fetch.test.js plus git-source-cache, git-clone, git-refresh (9 passed). make loc-check (167 warnings, 0 failures).

## Next

Closed by usr/docs/changelogs/20260918213000_git-free-catalog-and-unshallow.md.

## Source

rfcs/0020-resolve-and-lock.md pray install --offline

usr/docs/issues/20260918204000_install-pinned-git-revision-shallow-cache.md

usr/docs/issues/20260918205500_bundler-cargo-git-cache.md
