# Multi-source Prayfile efficiency

## Participants

Andrei Makarov

## Decisions

HTTP registry sources are demand-driven. Git sources now prepare when the first package needs that source, including transitives discovered during resolve. Unused git catalogs are not cloned. An unused HTTP origin is not contacted.

Keep path-source matching as it is. Implied source lookup is linear in source count per package and is not the cliff.

Project git cache identity is clone URL plus subdir. Empty subdir stays URL-only. The shared global cache stays URL-only. A subdir checkout is a linked git worktree of the URL-only clone.

Do not parallelize HTTP origins until serial origin RTT is measured on a real multi-registry Prayfile. Fetch protocol changes stay behind that measurement. Partial clone and blob:none stay behind a later pass. The fetch-byte numbers in this note are the gate.

## Effects

Measured 2026-09-17 on Darwin arm64, debug tests. Energy is CPU-seconds from getrusage. No joule meter.

### HTTP origins

cargo test -p pray-core --test registry_update_fetch two_registry_origins_fetch_per_origin_not_a_shared_index:

Two used origins, one unused, one package each on the used origins, update-shaped resolve:

- unused origin: 0 index, 0 metadata, 0 artifact, 0 distribution
- each used origin: 0 index, 1 metadata, 1 artifact, 1 distribution

Network is per used package and per used origin, not per declared source. Two origins do not share an index GET. Distribution JSON stays cached per origin URL.

Person wait on a slow network is serial GETs times used origins. At 50 ms RTT, two used origins is about 300 ms of round trips for metadata, distribution, and artifact, not counting body time. Unused origins do not add that.

### Git sources

Later pass on 2026-09-17: prepare on first package need, and one origin fetch per used source on refresh. Unused git catalogs are not cloned. Warm update copies the shared cache from the project worktree after that fetch.

cargo test -p pray-core --test git_source_prepare after that pass:

Path package plus two git sources, no git package: neither catalog is cloned.

One used git package plus one unused git source: only the used catalog is cloned.

Warm update of a used git source fetches origin once.

cargo test -p pray-bench --test source_scaling -- --ignored --nocapture after unused skip, tiny file:// catalogs, no declared git packages, debug:

- 2 sources: cold 1.3 ms, warm 0.20 ms
- 8 sources: cold 0.67 ms, warm 0.51 ms
- warm ratio 0.62 versus a 4x unused source count
- peak RSS 7.4 MiB

Unused git source count no longer drives wall time. A used catalog of tens of megabytes remains the size cliff for that one source.

Before the change the same unused fixture paid about 80 ms cold and 120 ms warm per tiny catalog, and warm update could fetch origin twice per source.

Path sources are not cloned. Several path sources add parse and implied-source lookup only.

### Shared git URL

Project git cache identity is clone URL plus subdir. Empty subdir stays URL-only. The shared global cache stays URL-only. A subdir checkout is a linked git worktree of the URL-only clone.

cargo test -p pray-core --test git_source_subdir after that split:

Two sources, same file:// URL, subdir left and right. Packages sample/one and sample/three from left, sample/two from right, declared in that order. All three resolve. Left and right keep distinct worktrees. Both share the URL-only object store. git_source_cached_repository returns that URL-only path.

A leftover subdir checkout with origin set is found when the URL-only path is missing.

### Fetch bytes against stored artifacts

cargo test -p pray-core --test git_source_fetch_bytes, Darwin arm64, debug, incompressible unused blobs next to a tiny used package:

- unused 512 KiB: clone cache 1080239 bytes, used package still resolves
- ignored scaling: unused 256 KiB clone 555873 bytes, unused 1 MiB clone 2128977 bytes, then an extra 1 MiB blob and refresh fetch delta 1050020 bytes

Clone cache stays above half the unused artifact. A later unused blob fetches about one-to-one. git clone --depth 1 still transfers unused blobs in the commit. Sparse-checkout does not cut those first-clone bytes.

### Coverage

Missing before this pass: request accounting for several HTTP origins and an unused origin, clone of a git source with no declared package, scaling of update resolve against git source count, RSS and CPU-seconds on that fixture, same-URL different subdir, clone bytes against a catalog that stores unused artifacts, trust import-repo after a subdir-only clone, shared object store across subdir worktrees.

Added: crates/pray-core/tests/registry_update_fetch.rs two-origin case, crates/pray-core/tests/git_source_prepare.rs, git_source_subdir.rs, git_source_fetch_bytes.rs, crates/pray-bench/tests/source_scaling.rs.

Futile: none of these assert rendered file order.

## Next

Partial clone and blob:none stay behind a later pass.

Co-location with other processes on the same machine was not measured.

## Source

crates/pray-core/src/resolve_project.rs, resolve_git_sources.rs, resolve_git_source_set.rs, resolve_git.rs, resolve_git_paths.rs, resolve_git_ensure.rs, resolve_git_lookup.rs, resolve_package_root.rs, resolve_implied_source.rs

usr/docs/issues/20260917152100_update-and-index-efficiency.md

usr/docs/changelogs/20260917165500_git-subdir-cache-and-fetch-bytes.md

usr/docs/changelogs/20260917171000_git-worktree-share-and-import-repo.md

Commands run:

- cargo test -p pray-core --test registry_update_fetch --test git_source_prepare: passed, including two_registry_origins_fetch_per_origin_not_a_shared_index, update_does_not_clone_git_sources_that_have_no_declared_package, update_clones_only_the_git_source_a_package_uses, and update_fetches_origin_once_per_used_git_source
- cargo test -p pray-bench --offline --test source_scaling -- --ignored --nocapture: passed, unused-source numbers above
- cargo fmt: passed
- later pass: cargo test -p pray-core --offline; cargo test -p pray-cli --offline --test install_git_global_cache --test install_git_catalog; cargo clippy -p pray-core --offline --tests -- -D warnings; make loc-check; TypeScript git integration tests; Ruby git_distribution, git_sources, conformance_resolve, upstream_refresh. All observed passing.
- fetch-byte pass: cargo test -p pray-core --offline --test git_source_subdir --test git_source_fetch_bytes --test git_source_prepare: passed (subdir fixture, 512 KiB unused clone 1080239 bytes, unused skip and single fetch)
- cargo test -p pray-core --offline --test git_source_fetch_bytes -- --ignored --nocapture: passed, unused 256 KiB clone 555873 bytes, unused 1 MiB clone 2128977 bytes, fetch delta 1050020 bytes
- this pass: cargo test -p pray-core --offline --test git_source_subdir --test git_source_prepare --test git_source_fetch_bytes: passed, including cached_repository_finds_a_subdir_worktree_when_url_only_cache_is_missing and shared git-common-dir
- cargo clippy -p pray-core -p pray-cli --offline --tests -- -D warnings: passed
- cargo fmt --check: passed
- make loc-check: 0 failures
- cargo test -p pray-cli --offline --test install_git_global_cache --test install_git_catalog: passed
- npmjs/pray-cli npm run lint: passed; node --test dist/git-source-cache.test.js: 2 passed
- rubygems/pray-cli bundle exec rspec spec/pray/git_sources_spec.rb spec/pray/git_distribution_spec.rb: 9 examples, 0 failures
- bundle exec rubocop on changed Ruby files: no offenses
