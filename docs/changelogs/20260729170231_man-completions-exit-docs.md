# Man page, completions, and exit-code docs

## Participants

Andrei Makarov

## Decisions

No clap. Ship static man page, pray completion bash|zsh|fish on the Rust CLI, and durable exit-code docs in SPEC and docs/. Keep exit codes out of concise help.

## Effects

- SPEC.md section 66 maps codes to reference CLI error classes; docs/cli-exit-codes.md for operators.
- docs/man/pray.1 with SYNOPSIS, OPTIONS, COMMANDS, EXIT STATUS, COMPLETIONS.
- crates/pray-cli/src/completion.rs; Command::Completion; help Meta entry.
- README and crates/pray-cli/README install hints; docs/releasing.md man install note.
- Tests: cli_ux completion cases; completion unit tests.

## Next

- Optional: clap migration; Ruby/npm completion parity; package man with Homebrew or distro packs.

## Source

- Issue: usr/docs/issues/20260729163923_unix-cli-help-output-audit.md
- Prior: usr/docs/issues/20260714100000_clig-dev-cli-audit.md
- Branch: patch/unix-cli-help-output
