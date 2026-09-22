# CLI help names comparison pairs

## Participants

Andrei Makarov

## Decisions

Keep verify, drift, render --check, and plan as separate commands. Dry-run of a write is pray plan. Local compose re-embed stays on pray install.

## Effects

Packages list describes update. Help first sentences name dest versus lock, dest versus fresh render, and dest write. Rust and TypeScript render help match dest and lock write. Ruby render writes dest only.

Confirming checks: cargo test --offline -p pray-cli --test cli_ux (9 passed). bundle exec rspec spec/pray/cli_help_spec.rb from rubygems/pray-cli (7 examples, 0 failures). node --test dist/cli/help.test.js after npm run build (help suite 6 passed). cargo fmt --all -- --check (exit 0). cargo clippy --offline -p pray-cli --all-targets -- -D warnings (exit 0). make loc-check (152 warnings, 0 failures).

## Next

Ships as 1.17.0. See usr/docs/issues/20260916223500_prepare-1-17-0-release.md. Alias of render --check onto drift can wait.

## Source

usr/docs/issues/20260916180300_install-update-verify-local-compose.md
usr/docs/issues/20260916180700_verify-drift-render-check.md
crates/pray-cli/src/help_text.rs
rubygems/pray-cli/lib/pray/cli/help.rb
npmjs/pray-cli/src/cli/help.ts
README.md
