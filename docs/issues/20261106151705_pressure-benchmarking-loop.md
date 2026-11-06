# Pressure benchmarking loop

## Decisions

Add an iterative pressure loop for this repository: measure CPU, resident memory, storage, and network (or mark network not in scope), raise load, lower resource ceilings by a named headroom factor, then optimize until the product stays under the new ceilings at the higher load.

Keep the executable ladder in `scripts/pressure-bench.rb` on top of the existing Rust compat harness and `make size-check`. Consume the shared procedure from `amkisko/pressure-benchmarking` (~> 1.0); do not keep a local path prayer for the skill. Bind measure, self-test, ceilings, load axis, samples, propose-basis, and Memory probe class in `.agents/project.md`.

## Effects

Added `scripts/lib/pressure_bench.rb`, `scripts/pressure-bench.rb`, and `make pressure-bench`. Removed local `prayers/pressure-benchmarking`. Prayfile now declares `pray "amkisko/pressure-benchmarking", "~> 1.0"` under `.agents/skills`. Lockfile pins `amkisko/pressure-benchmarking` 1.0.0. Consumer binding lives in `.agents/project.md` and composes into `AGENTS.md`.

Default ladder uses package counts from `PRAY_PRESSURE_LOADS` (default 1,10,100) and implementations from `PRAY_PRESSURE_IMPLS` (default rust,ruby,npm). Cross-implementation fixture digests must match. Local compat probes mark network as `not_in_scope`. Ship storage joins the report with `--include-ship`.

Validation observed:

- `ruby scripts/pressure-bench.rb --self-test`: PASS
- `pray install`: PASS; skill rendered from `amkisko/pressure-benchmarking` 1.0.0 under `.agents/skills/pressure-benchmarking`
- earlier propose path: `PRAY_PRESSURE_LOADS=1,10 PRAY_BENCH_ITERATIONS=20 PRAY_BENCH_WARMUP=2 ruby scripts/pressure-bench.rb --propose`: PASS; load 1 median wall_ms=2.11 cpu_ms=2.11 peak_rss_bytes=3784704 resolve_p95_us=6.17; load 10 median wall_ms=19.6 cpu_ms=18.7 peak_rss_bytes=4866048 resolve_p95_us=45.12; proposed ceilings at headroom 1.15 printed; no ceilings file written

## Next

- Run a full default ladder, then `PRESSURE_ARGS='--propose --write'` after a prove pass to commit `scripts/pressure-ceilings.json`
- Decide whether CI should gate `--gate` or only `size-check` on the release path
- Optional: raise `PRAY_PRESSURE_SAMPLES` to 3 before claiming cheaper or smaller

## Source

- `amkisko/pressure-benchmarking` 1.0.0 (skill + pressure-loop + known-practices)
- `.agents/skills/engineering-audit/resource-and-budget.md`
- `.agents/skills/dependency-audit/references/upgrade-resource-compare.md`
- `crates/pray-bench/examples/compat_bench.rs`
- `scripts/check-package-sizes.rb`
