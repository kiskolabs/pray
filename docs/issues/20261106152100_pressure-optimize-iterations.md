# Pressure optimize iterations

## Decisions

Run at least 20 pressure measure/optimize/prove cycles against Rust compat_bench. Use engineering-audit hot-path clues. Keep tree-hash digests identical. Tighten `scripts/pressure-ceilings.json` after prove. Timing ceilings use sample max times headroom so one noisy spike does not write an unbeatable gate.

Measure command: `PRAY_PRESSURE_LOADS=... PRAY_PRESSURE_SAMPLES=3 PRAY_BENCH_ITERATIONS=40 PRAY_BENCH_WARMUP=3 ruby scripts/pressure-bench.rb`.

Network stays `not_in_scope` for local compat probes.

## Effects

Baseline warm load 100 (n=3): wall_ms=330.41 cpu_ms=329.04 peak_rss_bytes=15220736. Early resolve_p95 figures near 450 us were wrong: Ruby `/m` made `.` cross newlines, so pressure-bench read lock p95 as resolve p95.

Parser fix: match `^resolve ([^\n]+)` and `^resources ([^\n]+)`. Self-test now includes parse/resolve/render/lock lines with distinct p95 values.

After parser fix, rewrite ceilings and re-gate (n=3): load 100 wall_ms~266-289 cpu_ms~285 resolve_p95_us~5885-7152; gate PASS. Wall/cpu gains from the optimize loop remain real. Correct resolve_p95 is roughly an order of magnitude above the mislabeled series.

Iteration log (each kept unless noted):

1. `PackageSpec::into_canonicalized` avoids clone inside parse
2. Stream `tree_hash_from_file_bytes` over ordered BTreeMap; drop redundant sort/string builder; equivalence test kept
3. Drop second `.canonicalized()` after `parse_package_spec` in resolve/registry/derived paths; move `spec` into `ResolvedPackage`
4. `normalize_line_endings` skips replace work when no CR
5. `build_skill_file_index` early empty return
6. `reject_dependency_cycles` short-circuit when no required deps
7. `validate_optional_referrals` short-circuit when no optional deps
8. Skip `read_lockfile` open when `Prayfile.lock` is absent
9. `Manifest::into_canonicalized` for hash path clarity
10. Scale `ContentBuilder` capacity and reserve managed spans
11. `read_declared_file` opens the leaf with O_NOFOLLOW instead of an extra metadata probe
12. Pressure parser first narrowed field sources; later fixed Ruby `/m` dot-newline so resolve p95 cannot swallow lock/render lines
13. `find_top_level` walks `char_indices` without a Vec
14. `ResolveQueue::resolve_all` stops cloning declarations before resolve
15. `find_prayspec_file` tracks one path without a Vec
16. Reuse one `parse_call` via `package_from_call` in compose pray statements
17. `skill_relative_file` strip_prefix without format alloc
18. Scoped compose package lookup via HashMap index
19. `push_declaration` fewer redundant name clones
20. Resolve queue maps use HashMap/HashSet for declaration lookup
21. Propose ceilings from sample max for timing series; rewrite ceilings; gate PASS
22. Clippy needless-borrow fix in hex update helper

Validation observed:

- `ruby scripts/pressure-bench.rb --self-test`: PASS
- `cargo test -p pray-core --lib`: PASS (135 tests)
- `cargo test -p pray-core --test source_symlink`: PASS (4)
- `cargo test -p pray-core --lib tree_hash_matches_legacy`: PASS
- `cargo clippy -p pray-core --all-targets -- -D warnings`: PASS
- `make bench-scaling`: PASS (near-linear guards)
- `ruby scripts/pressure-bench.rb --self-test` after newline-safe parse: PASS (resolve p95 kept at 100584.04 against lock 4714.04)
- `ruby scripts/pressure-bench.rb --gate` after ceiling rewrite: PASS (gate_failures=[])

Ceilings in `scripts/pressure-ceilings.json` (headroom 1.15, corrected resolve series): load 1 wall 6 / cpu 6 / rss 4635034 / resolve 200; load 10 wall 35 / cpu 34 / rss 5746688 / resolve 1168; load 100 wall 342 / cpu 335 / rss 17541530 / resolve 8301.

## Next

- Consider CI `--gate` on release path once runner variance is calibrated
- With true resolve_p95 visible, next cuts are still syscall/parse heavy (audit list: lockfile into_canonicalized, hex digest formatting, read buffer reserve, substitute Cow); re-measure before claiming resolve gains
- Raise `PRAY_PRESSURE_SAMPLES` to 3 before publishing version-2 ceilings as release evidence

## Effects (tri-impl follow-up)

Pressure-bench now defaults to `PRAY_PRESSURE_IMPLS=rust,ruby,npm`, asserts shared `output_sha256` across implementations, and writes version-2 ceilings under `implementations`.

Parity hot-path ports: Ruby/TS dependency short-circuits, normalize_line_endings CR skip, find_prayspec single-path scan; Ruby streaming tree_hash; npm compat-bench imports fixed to `pray-core/dist`.

Observed short ladder (n=1, loads 1 and 10, iterations 15): rust/ruby/npm completed; digest parity held; `--propose --write` produced version-2 ceilings.

## Source

- docs/issues/20261106151705_pressure-benchmarking-loop.md
- engineering-audit resource/performance modes and hot-path review
- crates/pray-bench/examples/compat_bench.rs
