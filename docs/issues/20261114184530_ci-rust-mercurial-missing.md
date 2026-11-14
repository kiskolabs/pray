# CI rust job fails without Mercurial

## Participants

Andrei Makarov

## Decisions

Install Mercurial in the GitHub Actions rust job so `publish_records_hg_revision_without_configured_backend` can call `hg`. Keep the test; do not skip when `hg` is missing on CI. Name the missing program in the revision test helper panic so a missing binary is obvious.

## Effects

Main CI run 43618954464 failed only on the rust job. `cargo test -p pray-cli --test revision` panicked at `Command::new("hg")` with `NotFound` because ubuntu-latest does not ship Mercurial. Ruby and npm jobs stayed green. Same failure mode appears on earlier main pushes from 2026-11-12 onward. Local check with Mercurial present: `cargo test -p pray-cli --test revision` passes four tests.

## Next

Commit and open a pull request from `patch/ci-install-mercurial`. Confirm the rust job installs Mercurial before `cargo test --workspace` and that the revision suite passes on the PR run.

## Source

https://github.com/kiskolabs/pray/actions/runs/43618954464
Branch `patch/ci-install-mercurial` from `main` at 1fa4c43.
Downstream: `.github/workflows/ci.yml`, `crates/pray-cli/tests/revision.rs`.
