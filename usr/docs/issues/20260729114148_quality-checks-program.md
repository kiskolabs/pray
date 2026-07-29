# Quality checks program

## Participants

- Andrei Makarov

## Decisions

- No template-level import recursion exists in Prayfile; cycle work covers package-graph edges from `spec.add_dependency` only.
- Package dependency cycles are rejected at resolve time with a clear resolution error.
- Parser fuzzing ships as `proptest` in CI plus a separate `fuzz/` cargo-fuzz harness for local nightly runs.
- Quality improvements land as one program branch covering parser fuzz, cycle reject, archive path safety, shared corpus growth, conformance fixture skeleton, CLI exit-code suites, signature negative cases, Rust coverage soft floor, mutation smoke, and network failure injection.

## Effects

- Added `dependency_graph` cycle detection and resolve rejection.
- Added parser property tests and cargo-fuzz targets for Prayfile, prayspec, and package path validation.
- Added archive path-escape coverage for `.praypkg` unpack.
- Grew `testdata/shared/manifest` with `legacy-target` and started `fixtures/` conformance layout.
- Added CLI exit-code tests including cycle failure, signature mismatch/replay/tamper tests, network unreachable failure tests, `cargo llvm-cov` soft floor in CI, and scoped `cargo mutants` smoke (non-blocking).

## Next

- Raise the llvm-cov floor after baselines stabilize.
- Make mutation testing blocking once survivors are cleaned up.
- Expand `fixtures/` resolver/render/verify packs and shared corpus cases.
- Optionally wire nightly fuzz runs in CI.
- Remove deprecated `target` / `output` / `agent` Prayfile forms in version 2.

## Source

- Plan: quality checks program (parser fuzz, cycle contract, corpus, coverage, goldens, mutants, archive/network/trust)
- Branch: feature/quality-checks-program
