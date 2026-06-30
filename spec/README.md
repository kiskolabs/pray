# Test coverage

Guidelines for tests in this repository.

## What to test

Test executable logic and user-facing behavior:

- parser acceptance and rejection of valid and invalid input
- canonical model output from Prayfile and prayspec
- resolver decisions, lockfile writes, and hash verification
- managed span ideal checksum and line position checks
- verify and drift detection (custom implementation, removed prayers, orphan markers)
- CLI exit codes and error messages for failure paths

## What not to test

Do not write tests that only assert implementation details.

Avoid:

- file, page, content, ordering, and regex assertions on rendered output unless `SPEC.md` requires exact bytes
- duplicate coverage of the same contract in multiple test files
- tests that pass regardless of behavior change

## Workflow

When fixing or refactoring:

1. Add or update a test that fails on the current bug or missing contract.
2. Implement the fix.
3. Run focused tests for the changed area.
4. Run the full test suite before claiming success.

## Coverage metrics

Tests should affect coverage metrics. Prefer integration and fixture-based tests that exercise real inputs from `fixtures/` when present.

Trivial one-liners need no new test.

## Running tests

From the workspace root:

- `cargo test` for the full suite
- `cargo test -p <crate>` for a focused crate

See `AGENTS.md` for clippy, fmt, and validation reporting.

## Validation reporting

List exact commands run and observed results. Never claim tests pass unless they were executed and passed.
