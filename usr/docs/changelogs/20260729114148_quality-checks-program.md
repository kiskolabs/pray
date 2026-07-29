# Quality checks program

## Participants

- Andrei Makarov

## Decisions

- Reject package dependency cycles during resolve.
- Keep fuzz harness out of the main workspace test job; cover parsers with proptest in CI.

## Effects

- `pray install` / `pray tree` and other resolve paths fail on cyclic `add_dependency` graphs.
- CI runs Rust line coverage with a soft 20 percent floor and a non-blocking mutation smoke pass.
- Shared corpus and conformance fixture trees grew for cross-runtime parse contracts.

## Next

- Publish follow-up PRs only if this branch is split; otherwise continue raising coverage and corpus depth on trunk.

## Source

- Issue: usr/docs/issues/20260729114148_quality-checks-program.md
- Branch: feature/quality-checks-program
