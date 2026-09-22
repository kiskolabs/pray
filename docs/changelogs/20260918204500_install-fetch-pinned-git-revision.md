# Fetch a locked git revision on install

## Participants

Andrei Makarov
Vesa Vänskä

## Decisions

pray install fetches a Prayfile.lock git source revision when that commit is missing from an existing project cache. That fetch is not a version re-resolve. --offline still refuses. The error no longer names --locked.

Rust, TypeScript, and Ruby match. No RFC: RFC 0020 already says install fetches after using the lock.

## Effects

Existing-cache checkout of a pin used refresh as allow_fetch, so install never fetched. Fresh cache already fetched. After this pass, a pin fetches unless offline.

Operator surface: pray install stderr when a pin is missing under --offline. Copy names the pin and offline mode. A person can retry without --offline. Remaining for a human: whether that stderr is enough in a CI log.

Tests: crates/pray-core/tests/git_source_pinned_fetch.rs, npmjs/pray-cli/src/git-pinned-fetch.test.ts, rubygems/pray-cli/spec/pray/git_pinned_fetch_spec.rb.

Confirming checks: cargo test -p pray-core --offline (full crate, including git_source_pinned_fetch 2 passed). cargo test -p pray-cli --offline --test install_git_catalog (2 passed) --test install_git_global_cache (2 passed). cargo clippy -p pray-core --offline --tests -- -D warnings. cargo fmt --all -- --check. bundle exec rspec spec/pray/git_pinned_fetch_spec.rb spec/pray/git_sources_spec.rb spec/pray/git_distribution_spec.rb (11 examples, 0 failures). bundle exec rubocop on the changed Ruby git files (no offenses). tsc -p tsconfig.test.json and node --test dist/git-pinned-fetch.test.js plus git-source-cache, git-clone, git-refresh (7 passed). biome check on the changed TypeScript git files (clean). make loc-check (166 warnings, 0 failures).

## Next

Closed in usr/docs/changelogs/20260918210000_offline-git-clone-and-file-origin.md.

## Source

https://github.com/kiskolabs/pray/issues/31

usr/docs/issues/20260918204000_install-pinned-git-revision-shallow-cache.md

rfcs/0020-resolve-and-lock.md
