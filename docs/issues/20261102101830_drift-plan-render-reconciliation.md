# Align drift with the accepted rendered destination

## Participants

Andrei Makarov (@amkisko)

## Decisions

Plain `pray drift` compares the destination with current managed content after applying the same reconciliation used by plan and write. Line-ending normalization happens after reconciliation. Preserved unmarked text is outside managed ownership and does not create `renderer_drift` by itself.

Verification reads every freshly rendered target, including a target with no managed spans. Rust, Ruby, and TypeScript use the same contract.

## Effects

A regression with local compose content failed before the correction: install and apply preserved an unmarked line, plan succeeded, and plain drift exited 6 with `renderer_drift`. The corrected Rust test passes.

A second regression failed before the correction for an empty `compose "AGENTS.md"` target. The target was current but had no `managed_span` lock record, so drift did not read it from disk. Verification now includes fresh targets in the target scan, and the regression passes.

The final TypeScript review also replaced a truth-value check with an explicit missing-value check, so an existing empty destination is not classified as absent.

The comparison reuses the destination bytes already read for verification. It does not add a second file read or managed-content scan. Existing symlink, UTF-8, and 32 MiB destination checks remain on the read boundary.

The full Rust workspace passed with `cargo test --offline`. `cargo clippy --offline --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check` passed. The complete Ruby suite passed with 386 examples and no failures. TypeScript core passed 259 tests and the CLI passed 70 tests. Biome and circular-dependency checks passed for both TypeScript packages. `make loc-check` reported 138 existing warnings and no hard-limit failures.

The corrected Rust 1.26.0 binary ran the repository fixture with exit 0 from `plan`, `verify`, `drift --semantic`, and plain `drift`. Plan reported `Prayfile.lock unchanged` and `AGENTS.md unchanged`.

## Next

- Ship the correction in the next coordinated CLI release.

## Source

- [Render consistency dependency record](../dependencies/20261030124204_pray-cli-render-consistency-and-retired-markers.md)
- [RFC 0030](../../rfcs/0030-render-markers-ownership.md)
- `crates/pray-core/src/verify/mod.rs`
- `crates/pray-cli/tests/install_drift.rs`
- `rubygems/pray-cli/spec/pray/render_reconciliation_spec.rb`
- `npmjs/pray-cli/src/render-check.integration.test.ts`
