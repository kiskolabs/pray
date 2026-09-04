# Registry cache cleanup

## Participants

Andrei Makarov

## Decisions

Release the cache parity and cleanup work as 1.11.0.

Use `.pray/cache/registry/<namespace>/<name>/<version>/<source-hash>` in the Rust, Ruby, and TypeScript CLIs. The source hash is the first 16 lowercase hexadecimal characters of SHA-256 over the exact source key. Artifact and tree hashes continue to validate cached content.

Keep `pray clean` as a full project-local wipe. Add `pray clean --unused` to validate the complete lockfile, retain its registry package paths, and remove other project-local registry entries without following symbolic links. Preserve Git caches, vendor output, project state, and global caches.

## Effects

RFC 0070 now specifies the shared registry cache path. RFC 0040 now specifies both clean modes.

All three CLIs reject malformed registry package identities and versions before building a cache path. A shared fixture proves the same path in every runtime.

All three CLIs implement strict `clean --unused` parsing and lockfile-driven pruning. Regression coverage includes locked and stale entries, legacy paths, staging paths, malformed or missing lockfiles, symbolic links, and unrelated local state.

Workspace, Rust crate, Ruby gem, npm package, generated lockfile, and version-sensitive tests now use 1.11.0. Regenerating `Prayfile.lock` changed its generated-by value and registry cache paths while retaining package content hashes.

Validation:

- `cargo check --workspace` finished successfully.
- `bundle install` finished successfully.
- `npm version 1.11.0 --no-git-tag-version` updated package metadata without creating a tag.
- `cargo run -p pray-cli --locked -- install` finished successfully and regenerated `Prayfile.lock`.
- `cargo test -p pray-core registry_cache && cargo test -p pray-cli --test cache_cleaning` passed 2 cache path tests and 4 cleanup tests.
- `bundle exec rspec spec/pray/registry_spec.rb spec/pray/clean_spec.rb spec/pray/cli_parse_spec.rb spec/pray/install_spec.rb` passed 25 examples.
- `npm run build && ./node_modules/.bin/tsc -p tsconfig.test.json && node --test dist/registry/cache.test.js dist/vendor/clean.test.js dist/cli/commands/clean.test.js` passed 6 tests.
- The first `cargo fmt --all --check` reported two formatting differences. `cargo fmt --all` applied them. The final `cargo fmt --all --check` finished successfully.
- `make loc-check` finished with 134 warnings for existing or allowlisted files and 0 failures.
- `cargo clippy --workspace --all-targets -- -D warnings` finished successfully.
- `cargo test --workspace` finished successfully.
- The first two `make lint` runs reported formatting in `lib/pray/cache_clean.rb`. After the method was reformatted, the final `make lint` inspected 130 Ruby files with no offenses and validated RBS signatures.
- `POLYRUN_COVERAGE=1 bundle exec polyrun parallel-rspec` finished successfully after both implementation and formatting passes.
- `bundle exec rspec spec/pray/clean_spec.rb` passed 3 examples after the final Ruby formatting change.
- The first `npm run lint` reported import ordering and formatting in five changed files. `./node_modules/.bin/biome check --write src/cli/main.ts src/registry/cache.test.ts src/registry/cache.ts src/vendor/clean.test.ts src/vendor/clean.ts` fixed them. The final `npm run lint` checked 154 files and found no circular dependency.
- `npm run test:coverage` passed 139 tests with 70.31 percent line coverage after both implementation and formatting passes.
- `make release-dry-run` finished successfully. It packaged pray-core 1.11.0, checked pray-transport and pray-cli through the expected unpublished-dependency fallback, passed 139 npm tests, packed a 207.2 kB npm package, passed 184 Ruby examples, built pray-cli-1.11.0.gem, and performed no publish.

## Next

Publish and tag 1.11.0 through the manual release process when approved.

## Source

rfcs/0040-cli-surface.md
rfcs/0070-reference-implementation.md
testdata/shared/registry-cache/identity-first.json
usr/docs/issues/20260904071400_registry-cache-layout-parity.md
CHANGELOG.md
