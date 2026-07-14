# clig.dev CLI audit for pray

## Participants

Andrei Makarov

## Decisions

Fix P0–P2 gaps in this pass: discoverable help, usage errors with suggestions, --no-input, PRAY_NO_COLOR/NO_COLOR helper. Defer clap migration, shell completions, man pages, and exit code 7 for network errors.

Scope: Rust canonical CLI, Ruby gem CLI, npm TypeScript CLI — help and usage-error parity only.

## Effects

Audit completed against https://clig.dev.

Implemented across Rust, Ruby, and npm:
- concise help with getting-started workflow, grouped commands, docs link, exit-code summary
- pray help COMMAND and pray COMMAND --help for top commands
- usage errors (exit 2) with did-you-mean suggestions for unknown commands
- global --no-input flag (sets PRAY_NO_INPUT; trust prompts fail fast with flag named)
- PRAY_NO_COLOR and NO_COLOR helpers in pray-core terminal module and Ruby/npm mirrors

Validation (2026-07-14):

```
cargo test -p pray --test cli_ux
# running 4 tests ... ok. 4 passed

cd rubygems/pray-cli && bundle exec rspec spec/pray/cli_help_spec.rb spec/pray/cli_suggest_spec.rb spec/pray/cli_parse_spec.rb
# 11 examples, 0 failures

cd npmjs/pray-cli && npm run build && tsc -p tsconfig.test.json && node --test dist/cli/help.test.js
# help and suggest suites pass (4 tests)
```

Full npm test suite git integration cases require unsandboxed zstd/git; not run to completion in sandbox.

### Gap matrix (before fixes)

Basics
- pass: exit 0 on success, non-zero on failure (Rust/Ruby); partial npm (exit code map diverged)
- pass: stdout for primary output, stderr for errors
- fail: no argument-parsing library (hand-rolled parser in all three)

Help
- fail: bare command list, no description or workflow example
- fail: no per-subcommand help (install --help failed)
- fail: no pray help install alias
- partial: Ruby help stale vs Rust; npm help missing several flags

Errors
- fail: unknown commands used unsupported feature prefix (exit 8)
- fail: no did-you-mean suggestions

Output
- pass: update --json, manifest JSON on stdout
- partial: verify exits 0 with stderr warnings (undocumented in help)
- fail: PRAY_NO_COLOR in SPEC but unimplemented

Interactivity
- partial: trust prompts TTY-gated in Rust
- fail: no --no-input flag

Configuration
- pass: XDG config path ~/.config/pray/config.toml
- pass: flags override env per project patterns

### Evidence paths

- Rust entry: crates/pray-cli/src/main.rs
- Rust errors: crates/pray-core/src/error.rs
- Ruby parse: rubygems/pray-cli/lib/pray/cli/parse.rb
- npm dispatch: npmjs/pray-cli/src/cli/main.ts
- SPEC exit codes: SPEC.md section 66

## Next

- clap migration with generated help and completions
- man pages via pray help
- align npm exit codes fully with SPEC section 66 in a dedicated pass
- implement exit code 7 for fetch/network failures
- --plain for script-stable human tables

## Source

- https://clig.dev
- Plan: clig.dev audit and CLI UX fixes for pray
- SPEC.md sections 65–66
