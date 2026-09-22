## Participants

Andrei Makarov

## Decisions

Ship three user-visible passes together: environment-aware rendering, clig.dev CLI UX fixes, and install-time git source refresh fallback.

Environment groups filter rendered managed spans and provisioned files only. Resolution and lock entries stay complete for every declared package.

CLI project context precedence: CLI flag, process environment, project `.env`, then defaults.

Git refresh fallback runs on install when not `--locked` or `--frozen` and resolution fails with a message that may benefit from refreshing git sources.

## Effects

Rust, TypeScript, and Ruby CLIs gained shared project context, dotenv loading, group membership on manifest packages, environment validation, render filtering, lockfile environment serialization, concise help, per-command help, usage suggestions, `--no-input`, color helpers, and git refresh fallback on install.

SPEC.md, README.md, and JSON schemas updated for groups, environment fields, and project invocation variables.

## Next

Run full CI matrix after merge.

## Source

usr/docs/issues/20260714120000_environment-aware-pray.md
usr/docs/issues/20260714100000_clig-dev-cli-audit.md
Branch feature/install-git-source-refresh-fallback
