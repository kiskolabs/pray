# Publish stdout feedback and global quiet/verbose

## Participants

Andrei Makarov

## Decisions

Live `pray publish` must print the same line shape as `--dry-run`, with verbs `published` and `unchanged`, plus a footer `N published, M unchanged`. Exit codes stay unchanged. Global `-q`/`--quiet` suppresses non-error stdout; `-v`/`--verbose` adds artifact paths and the unchanged skip reason. Help, man page, and completion list the flags. Parity in Rust, Ruby, and TypeScript CLIs.

## Effects

Operator surface assessed: `pray publish` stdout, `pray --help` Options, `pray help publish`, `docs/man/pray.1`. Hierarchy is per-package status lines then a count footer. Interactive flags are global quiet/verbose. Empty stdout is the quiet path. A person can tell whether packages were written or skipped. Remaining for a human: whether vendor and other silent writers should adopt the same policy.

Visual surface: none beyond CLI text.

Confirming checks: cargo test -p pray-cli --test publish_feedback (4 passed), --test publish_idempotency (1 passed), --test cli_ux (10 passed). cargo clippy -p pray-cli --tests -- -D warnings. bundle exec rspec spec/pray/publish_spec.rb (5 examples, 0 failures). npm test in npmjs/pray-cli (270 passed).

## Next

Optional: apply quiet/verbose to other silent writers such as vendor. JSON machine output for publish stays a separate request.

## Source

Maintainer report: pray publish succeeds with no stdout (2026-10-21).

docs/cli-exit-codes.md

crates/pray-cli/src/publish.rs

crates/pray-cli/src/publish_plan.rs

crates/pray-cli/src/cli_output.rs
