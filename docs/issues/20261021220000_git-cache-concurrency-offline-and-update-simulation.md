# Git cache concurrency, offline lazy fill, and update simulation

## Participants

Andrei Makarov

## Decisions

Fact-check of the eight backlog items against 1.21.0-era sources before this change (agent c1a054fd-e848-47a3-8998-3a62f38a2602):

1. `pray update --latest --dry-run` would-conflict path-fork paths: confirmed gap. Dry-run skipped refresh and did not simulate merge conflicts. Destination conflict checks already existed; path-fork merge simulation did not.
2. Optional branding rename map after clean upstream refresh: needs RFC. No implementation in this pass. Keep Mode B fail-closed merge and the reset-then-rebrand operator workflow.
3. `pray clean` / `pray doctor` layout report and `--migrate-git-cache`: confirmed gap for doctor and migrate. Clean still only removes project `.pray/cache`, vendor, and state. Install already quarantines matching full clones. Ship `pray doctor` layout report and optional `--migrate-git-cache` quarantine; leave shared bare deletion to operators.
4. Shallow plus partial for distribution git sources: permanent default until a later RFC changes clone policy. Ensure and fetch before materialize remain the mitigation for missing objects. Documented in doctor help and this note.
5. Update success only after the new lock revision is materializable on the verify catalog path: confirmed gap. Update now smoke-archives the locked revision from the project catalog markers before writing destinations or the lock.
6. Global bare store atomic clone, advisory lock, tighter readiness: confirmed gap. Clone now stages to `*.partial`, marks `.pray-ready`, renames into place, and holds an advisory lock around clone and tip fetch. Readiness requires HEAD, objects, and either `.pray-ready` or a resolvable legacy HEAD.
7. Lazy catalog fill prefer archive or show; markers only at catalog root: partial before. Tree fill already used archive. File fill used work-tree checkout into the catalog. File fill now uses `git show` with a path relative to the marker directory (the materialized catalog root). Lookup may walk up from a distribution subdir such as `prayers/` to find those markers; it does not check out into the shared bare.
8. Offline refuse lazy blob fetch: confirmed gap for promisor fetches during materialize. Offline materialize sets `GIT_NO_LAZY_FETCH=1` on archive and show.

## Effects

Rust, Ruby, and TypeScript shared-bare ensure paths use staging rename, lock, and ready marker. Materialize refuses offline promisor fetches. Update dry-run prints `would-conflict` lines and fails closed when path-fork merge would conflict. Update confirms catalog materializability before write. `pray doctor` reports layout; `--migrate-git-cache` renames matching project full clones to `*.legacy-git`.

Operator surface assessed: `pray update --help`, `pray doctor --help`, dry-run stdout `would-conflict` lines, and offline materialize errors. A person can preview path-fork conflicts without writes and inspect cache layout. Remaining for a human: branding rename map RFC if product wants automated overlay rewrites after a clean refresh.

Visual surface: none beyond CLI help and diagnostic text.

Validation observed:

- `cargo test -p pray-core --offline --test git_source_catalog_layout --test git_source_offline --test git_source_prepare --test git_source_cache_guard -- --test-threads=1` passed
- `cargo test -p pray-cli --offline --test package_upstream -- --test-threads=1` passed (includes would-conflict dry-run)
- `cargo clippy -p pray-core -p pray-cli --offline --tests -- -D warnings` passed
- `cargo fmt --all -- --check` passed
- `make loc-check` passed with warnings only (no hard failures)
- `bundle exec rspec spec/pray/git_pinned_fetch_spec.rb spec/pray/git_clone_spec.rb` passed (5 examples)
- `node --test dist/git-clone.test.js dist/git-pinned-fetch.test.js dist/git-source-cache.test.js` passed (8 tests)

## Next

- Branding rename map: open RFC only if product wants it
- Optional later: `pray clean` shared-bare purge flag remains out of scope
- Audit follow-ups still open: offline env on cat-file probes, dry-run cache side effects, plain update --dry-run conflict plan, atomic catalog materialize, install-time materializable confirm
- Cross-tool bare lock now uses exclusive `*.lock.d` directories in Rust, Ruby, and TypeScript
- Ruby and TypeScript now plan would-conflict path-fork paths on `update --latest --dry-run` (fact-check item 1 parity)

## Source

Maintainer backlog session 2026-10-21.

Fact-check agent: c1a054fd-e848-47a3-8998-3a62f38a2602.

docs/issues/20261021194040_git-cache-legacy-and-path-fork-refresh.md

RFC 0020 resolve and lock. RFC 0114 package upstream.
