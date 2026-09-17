# Multi-source Prayfile efficiency

## Participants

Andrei Makarov

## Decisions

HTTP registry sources are demand-driven. Git sources are not. prepare_git_sources clones or fetches every git source in the Prayfile before any package is resolved, including sources no package names. An unused HTTP origin is not contacted.

Keep path-source matching as it is. Implied source lookup is linear in source count per package and is not the cliff.

Do not parallelize HTTP origins until serial origin RTT is measured on a real multi-registry Prayfile. Skip unused git sources, or prepare a git source when the first package needs it, before changing fetch protocol.

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

prepare_git_sources walks every source with kind git. update sets refresh_source_revisions, so each of those sources is cloned on first use of the cache and then fetched with depth 1 on later updates. refresh_global_git_cache runs a second fetch into the global git cache when that cache exists. Warm update can do two fetches per git source.

cargo test -p pray-core --test git_source_prepare:

Prayfile with a path package plus two git sources, only the path package declared. Both git catalogs were cloned into .pray/cache/git. The unused catalog held a 64 KiB blob. After clone the unused cache was at least 32 KiB.

cargo test -p pray-bench --test source_scaling -- --ignored --nocapture, tiny file:// catalogs, no declared git packages, debug:

- 2 sources: cold 162 ms, warm 246 ms
- 8 sources: cold 614 ms, warm 976 ms
- cold ratio 0.95, warm ratio 0.99 versus a 4x source count
- peak RSS 7.0 MiB, CPU-seconds 0.078

Wall time is about 80 ms cold and 120 ms warm per tiny git source. CPU is a small fraction of wall. The wait is git subprocess IO. A real catalog of tens of megabytes times source count is the first dramatic cliff, including catalogs that no package uses.

Path sources are not cloned. Several path sources add parse and implied-source lookup only.

### Shared git URL

git_source_cache_directory keys only on clone URL. Two source names with the same URL share one cache. A later sparse-checkout for source.subdir would change that worktree for both names. That is a correctness risk, not a size cliff. It was not exercised in this pass.

### Coverage

Missing before this pass: request accounting for several HTTP origins and an unused origin, clone of a git source with no declared package, scaling of update resolve against git source count, RSS and CPU-seconds on that fixture.

Added: crates/pray-core/tests/registry_update_fetch.rs two-origin case, crates/pray-core/tests/git_source_prepare.rs, crates/pray-bench/tests/source_scaling.rs.

Futile: none of these assert rendered file order.

## Next

Prepare a git source when the first package needs it, so unused git catalogs are not cloned or fetched on update.

Avoid a second fetch into the global git cache when the project cache was just refreshed.

Measure git fetch bytes against a catalog that stores artifacts. Tiny file:// blobs do not show that cliff.

Same-URL sources with different subdir still need a fixture.

Co-location with other processes on the same machine was not measured.

## Source

crates/pray-core/src/resolve_project.rs, resolve_git_sources.rs, resolve_git.rs, resolve_package_root.rs, resolve_implied_source.rs

usr/docs/issues/20260917152100_update-and-index-efficiency.md

Commands run:

- cargo test -p pray-core --test registry_update_fetch --test git_source_prepare: passed, including two_registry_origins_fetch_per_origin_not_a_shared_index and update_clones_git_sources_that_have_no_declared_package
- cargo test -p pray-bench --offline --test source_scaling -- --ignored --nocapture: passed, numbers above
- cargo fmt: passed
