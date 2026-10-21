# Git cache legacy quarantine and path-fork conflict UX

## Participants

Andrei Makarov

## Decisions

Stop project git-cache lookup from selecting leftover full clones. Prefer the ready shared bare, then a stub `.pray-git-dir`. On ensure, rename matching project full clones to `*.legacy-git`. Before archive or lazy checkout, require the locked revision with `cat-file -e` and fail with `git_revision_unfetched` or `git_cache_stale` recovery text. Path-fork merge conflicts name `upstream_merge_conflict` and the reset-then-rebrand workflow in the message and in `pray update --help`.

Parity across Rust, Ruby, and TypeScript.

## Effects

Tests: crates/pray-core/tests/git_source_legacy_coexistence.rs, updated git_source_subdir.rs, rubygems/pray-cli/spec/pray/git_sources_spec.rb, npmjs/pray-cli/src/git-source-cache.test.ts, package_upstream unit message assertion.

Confirming checks: cargo test -p pray-core --test git_source_legacy_coexistence --test git_source_subdir --test git_source_catalog_layout (6 passed). cargo test -p pray-core --lib package_upstream (7 passed under filter). bundle exec rspec spec/pray/git_sources_spec.rb (6 examples, 0 failures). npm test in npmjs/pray-cli (270 passed).

## Next

Dry-run would-conflict for path forks. Doctor/migrate for mixed layouts. Update-success smoke that the new lock revision archives.

## Source

docs/issues/20261021194040_git-cache-legacy-and-path-fork-refresh.md
