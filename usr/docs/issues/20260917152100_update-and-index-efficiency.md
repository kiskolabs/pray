# Update and prayers index efficiency

## Participants

Andrei Makarov

## Decisions

pray update does not read v1/index.json. It re-resolves declared packages from per-package metadata, then renders and writes destinations. pray update --latest does that after a first resolve used only to learn registry latest versions and rewrite constraints. Index layout changes are not an update optimization.

Keep v1/index.json as a name list (RFC 0060). Do not shard, paginate, or embed summaries until search or sync measurements at a real catalog size exceed the budgets below. The first real cliff is search with summaries: one metadata GET per match, and the CLI always asks for summaries.

Do not rewrite the resolver for speed. The measured path is linear in package count. Cache package metadata inside a command, and skip the second resolve when --latest has no constraint change, before changing the distribution tree.

## Effects

Measured 2026-09-17 on Darwin arm64, debug tests unless named as cargo bench (release). Energy is CPU-seconds from getrusage. No joule meter.

### Update versus update --latest

update_resolve_options sets ignore_locked_versions when no package name is given, and always sets refresh_source_revisions. That is one full resolve. --latest runs constraint_preview_options first (same flags), rewrites Prayfile and path-upstream pins, then resolve_after_latest_upstream, which may resolve a third time after a path-fork refresh.

cargo test -p pray-bench update_scaling -- --ignored --nocapture, path packages, debug:

- 5 packages: 0.81 ms wall
- 40 packages: 6.61 ms wall, scaling ratio 1.02 versus an 8x input
- two resolves (latest shape): 13.15 ms, 1.99 times one update resolve
- peak RSS 8.6 MiB, CPU-seconds 0.049

cargo test -p pray-core --test registry_update_fetch, HTTP fixture, two packages:

- update-shaped resolve twice: 0 GETs of v1/index.json, 4 metadata GETs, 2 artifact GETs, 2 torrent sidecar 404s
- second resolve of a cached package: metadata GET again, no artifact GET
- search with summaries: 1 index GET plus 1 metadata GET per match, 0 artifacts

Those HTTP counts were before the cache and torrent gate. After this pass, the same fixture expects 2 metadata GETs across two resolves (not 4), 0 sidecar GETs, and 1 distribution GET. A second resolve of a cached package expects 1 metadata GET.

Network for a warm-cache update is one metadata GET per resolved package. Sidecar GET runs only when v1/distribution.json lists torrent. --latest reuses in-process metadata, so a second resolve does not GET package JSON again. When constraints and upstream pins do not change, --latest skips that second resolve and still refreshes path-fork files if needed. There is still no conditional GET.

Git sources on update fetch origin with depth 1 for the whole catalog. That was not benched here. It is independent of index.json size.

### Index CPU, memory, storage, network

cargo test -p pray-bench index_scaling and cargo bench -p pray-bench --bench index -- --sample-size 20.

Compact JSON is about 17 bytes per name. Pretty publish (serde_json::to_string_pretty) is about 1.3 times that on a 10k-name sample generated the same way, about 22 bytes per name.

- 10_000 names: 170 KiB compact, parse 431 us release, name search 387 us
- 50_000 names: parse 2.19 ms release, name search 1.94 ms, about 25 million names per second
- debug parse 5_000 to 40_000 names: ratio 1.00, name-string heap 70 KiB to 620 KiB, peak RSS 11.3 MiB
- MAX_HTTP_RESPONSE_BYTES is 64 MiB. Compact cliff about 3.95 million names. Pretty cliff about 3.0 million names. Client GET fails closed there.
- pray serve reads the whole file into memory per GET (server_static::static_file_response). MAX_SERVE_BODY_BYTES (16 MiB) limits upload bodies, not GET of index.json. Concurrent GET times index size is the serve RAM bound (32 connections).

Name search and parse stay linear through 50k. They do not degrade dramatically at catalogs this project will host soon.

### Search summaries

CLI search always sets include_summary true. Local debug: 20 matches 0.45 ms, 160 matches 3.58 ms, ratio 1.00 in match count. Names-only at 160 matches: 0.10 ms. Summary search was 35 times the names-only scan. Release: 12 us per local match, steady from 20 to 160.

Over HTTP that is serial GETs. At 50 ms RTT, 20 matches is about 1 s plus index fetch; 160 matches is about 8 s. That is the first dramatic cliff, and it is search, not update.

Version select over 50 to 400 published versions: ratio 1.13, 400-version metadata JSON 100 KiB. Release 1000 versions: 383 us. History growth is linear and far from the 64 MiB GET ceiling.

### Coverage

Missing before this pass: request accounting for update versus index, index size versus the HTTP GET ceiling, scaling of index parse and name search, summary N+1 cost, --latest as two resolves, version-history select cost, RSS and CPU-seconds on those fixtures.

Added: crates/pray-core/tests/registry_update_fetch.rs, registry_index_size.rs, extra registry_search cases; crates/pray-bench index helpers, index_scaling.rs, update_scaling.rs, benches/index.rs.

Futile: none of these assert rendered file order or implementation-only strings.

Shipped in this pass: in-process metadata and distribution caches, --latest skip when constraints and upstream pins do not change, torrent sidecar GET gated on distribution protocols.

## Next

Search: names-only by default, or copy latest non-yanked summary onto the index entry. Either change is an RFC 0060 amendment if the index schema grows.

Pretty versus compact index.json is a small storage tax, not the cliff. Compact write is optional.

Git catalog refresh size still needs a fixture that counts fetch bytes against a tree that stores artifacts.

Multi-source git catalogs are cloned even when no package uses them. See usr/docs/issues/20260917154800_multi-source-prayfile-efficiency.md.

Co-location with other processes on the same machine was not measured.

## Source

crates/pray-cli/src/commands_update.rs, commands_update_latest.rs, commands_search.rs, server_static.rs, registry_ops.rs

crates/pray-core/src/registry.rs, registry_search.rs, registry_http.rs, registry_http_cache.rs, resource_limits.rs, fetch.rs, resolve_git.rs

rfcs/0060-distribution.md, docs/static-distribution.md

Commands run:

- cargo test -p pray-core --test registry_update_fetch --test registry_index_size --test registry_search --offline: passed (7 tests in those files after the HTTP search case)
- cargo test -p pray-bench --offline: passed
- cargo test -p pray-bench --offline -- --ignored --nocapture: passed, numbers above
- cargo bench -p pray-bench --bench index -- --sample-size 20: passed, numbers above
- cargo clippy -p pray-core -p pray-cli --offline --tests -- -D warnings: passed
- cargo clippy -p pray-transport --offline --tests -- -D warnings: passed
- cargo test -p pray-core --offline: passed
- cargo test -p pray-cli --offline -- update: passed
- cargo test -p pray-transport --offline --test torrent_piece_fetch --lib torrent: passed
- cargo test -p pray-cli --offline --test update_destinations -- --test-threads=1: passed
- cargo fmt --check: passed
- make loc-check: 0 failures
- npmjs/pray-cli node --test metadata, update flags, update-destinations, package-upstream: 18 passed
- rubygems/pray-cli bundle exec rspec spec/pray/registry_spec.rb spec/pray/update_latest_spec.rb: 19 examples, 0 failures
- bundle exec rubocop on changed Ruby files: no offenses
