# Production readiness milestone plan

Goal: move `pray` from a working prototype to a production-ready app with clear support boundaries, end-to-end validation, and hardened trust and recovery behavior.

## Milestone 1: Beta readiness

### Must-have
- full CLI path coverage for the currently shipped commands
- real end-to-end multi-process tests across clean directories
- lockfile integrity checks for stale, corrupted, and drifting state
- render and verify hardening for orphan markers, removed spans, and marker corruption
- offline and local-path behavior with explicit safe-source limits
- auth and signing basics for register, verify, passkey, SSH key, and publish identity
- documentation that matches current behavior

### Success criteria
- a new user can install, render, verify, and inspect drift without fixture hacks
- failure paths produce clear errors and stable exit codes
- the test suite proves the supported beta workflows from start to finish

### Suggested first test slices
- `crates/pray-cli/tests/install.rs`
  - add one cross-process happy-path test that starts from an empty workspace and exercises install → verify → drift → format
  - add one negative-path test for corrupted or stale lock state
- `crates/pray-cli/tests/auth.rs`
  - add replay / invalid-signature failure coverage
- `crates/pray-core/tests/parser.rs`
  - add one malformed input acceptance/rejection case for a user-facing contract

## Milestone 2: Production readiness

### Must-have
- security review of trust boundaries
- recovery and rollback from `Prayfile.lock`
- upgrade and migration behavior for state, cache, and lock formats
- real distributed usage across separate machines or environments
- operational robustness: logs, timeouts, retries, conflict handling
- cross-platform validation on the supported OS set
- beta-to-production support policy

### Success criteria
- a team can use `pray` together without manual repair steps
- interrupted or partial operations can be recovered safely
- trust, publish, and sync behavior is predictable under real network conditions

### Suggested first test slices
- add restart/recovery tests around lockfile and rendered output
- add network-failure tests for `serve`, `publish`, and `sync`
- add migration tests whenever a persisted format changes

## Milestone 3: Post-launch improvements

### Nice-to-have
- federation expansion
- DNS discovery
- P2P / DHT / torrent-style distribution
- derived metadata and confessions
- better UX and richer diagnostics
- broader ecosystem integrations

## Recommended execution order
1. close the beta E2E gaps
2. harden security and recovery behavior
3. define and test upgrade/migration guarantees
4. expand distribution/federation only after the core app is stable

## Current best next slice
Start with the beta E2E gap: one clean, reproducible install/render/verify flow plus one corruption or drift failure path.

That slice gives the fastest feedback on whether the current app is safe to broaden.
