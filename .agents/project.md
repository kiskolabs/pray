## Additional instructions

### .agents/project.md
Repository for the pray open specification and the reference CLI.

Read `README.md` for project positioning and `rfcs/` for the normative Prayfile, prayspec, lockfile, distribution point, and CLI design (RFC 0111).

## Project intent

- Production readiness. Build the reference CLI and specification together, prioritizing validated contracts, user-facing reliability, and test coverage.
- Problem focus. Inference input is operational. Packaging shapes will keep changing. Prayfile targets reproducible composition, provenance markers, and sync of shared input libraries across repositories, not any one vendor workflow.
- What the tool must do. Resolve declared input dependencies, lock exact versions and hashes, render tool-specific files under defined contracts, cite managed blocks with compact pray markers into `Prayfile.lock`, and keep shared input pinned and updatable through manifest and lockfile semantics.
- Production focus. Prefer contract clarity, production validation, and test coverage over premature implementation.

## Rust workspace

For the pray reference implementation, run from the workspace root:

- `cargo test` for the full suite
- `cargo test -p <crate>` for a focused crate
- `cargo clippy` and `cargo fmt --check` before claiming quality checks pass

Use coverage tooling declared in this repository when validating coverage claims.

Prefer files around 150 lines or fewer when cohesion allows. Treat 300 lines as a hard upper bound for any source file unless a very small exception is clearly justified. When a file approaches that ceiling, split by semantic responsibility into separate modules, folders, or helpers rather than by arbitrary line count.

Enforce with `make loc-check` (warn >=150, fail >300 unless ratcheted in `scripts/loc-limits.allowlist`). Language linters mirror the hard ceiling: RuboCop `Pray/FileLength` (Ruby), Biome `noExcessiveLinesPerFile` (TypeScript). Clippy has no file-LOC lint yet.

Test coverage must follow `spec/README.md` guidelines.

## Pressure benchmarking binding

Consumer binding for `amkisko/pressure-benchmarking`:

- measure: `make pressure-bench` (Rust, Ruby, and npm compat harnesses)
- self-test: `ruby scripts/pressure-bench.rb --self-test` (distinct parse/resolve/render/lock p95)
- ceilings: `scripts/pressure-ceilings.json` (version 2 under `implementations`)
- load axis: package count via `PRAY_PRESSURE_LOADS` (default `1,10,100`)
- implementations: `PRAY_PRESSURE_IMPLS` (default `rust,ruby,npm`)
- samples-per-load: `PRAY_PRESSURE_SAMPLES` (use `>= 3` before gate, propose, or write)
- min episode iterations: 10 optimize/prove cycles at the stressed load
- propose-basis: timing series `max * headroom`; RSS `median * headroom`; headroom default `1.15`
- Memory probe class: `process_lifetime` (compat harness peak RSS)
- flags: `--gate`, `--propose`, `--write`, `--include-ship`

## Shared instructions
