# Unix CLI help and command output audit

## Participants

Andrei Makarov

## Decisions

Audit pray help and command output against Unix tool conventions (git, cargo, curl) and SPEC exit codes. Prioritize discoverability bugs and help that misleads over cosmetic polish. Prefer Usage synopsis, command list, and See also style over Documentation and Exit codes footers on every help screen.

## Effects

Initial audit on installed pray 1.6.0 found Documentation/Exit codes footers, missing per-command help (pray help remove suggested remove), no Usage synopsis, dense global flags line, and unknown-command errors without See pray --help.

Implemented on patch/unix-cli-help-output: Usage/Options/See also reshape; per-command help for listed commands; See pray --help on unknown commands; Rust/Ruby/npm parity; tests updated.

Also on the same branch: docs/cli-exit-codes.md and SPEC section 66 mapping; docs/man/pray.1; pray completion bash|zsh|fish on the Rust CLI.

## Next

1. Done: help reshape, per-command coverage, See pray --help, exit-code docs in SPEC/docs, man page, shell completions (Rust).
2. Optional later: clap migration with generated help/completions/man; Ruby/npm completion parity; cargo-install man packaging.

## Source

- crates/pray-cli/src/help.rs
- crates/pray-core/src/error.rs
- crates/pray-core/src/cli_suggest.rs
- SPEC.md section 66 exit codes
- Prior audit: usr/docs/issues/20260714100000_clig-dev-cli-audit.md
- Comparators: git --help, cargo --help, git status -h
