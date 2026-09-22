# Unix-style CLI help output

## Participants

Andrei Makarov

## Decisions

Reshape root and per-command help toward git/cargo conventions: Usage synopsis, grouped commands with blank lines, Options block, See also line. Drop Documentation URL and Exit codes footers from default help. Cover every listed command with per-command help. Append See pray --help on unknown-command usage errors.

## Effects

- Rust: crates/pray-cli/src/help.rs plus help_text.rs; cli_suggest adds See pray --help.
- Ruby and npm help/suggest mirrors updated for parity.
- Tests: crates/pray-cli/tests/cli_ux.rs, pray-core cli_suggest unit tests, Ruby cli_help/cli_suggest specs, npm help.test.ts.

## Next

- Follow-up landed in usr/docs/changelogs/20260729170231_man-completions-exit-docs.md (man page, completion command, exit-code docs).

## Source

- Issue: usr/docs/issues/20260729163923_unix-cli-help-output-audit.md
- Branch: patch/unix-cli-help-output
