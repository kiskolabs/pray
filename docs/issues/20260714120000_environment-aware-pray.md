## Participants

Andrei Makarov

## Decisions

Group blocks are render selectors, not organizational sugar only. Package resolution and lock entries stay complete for every declared package. Only managed spans and provisioned files filter by the selected environment.

Project invocation uses PRAY_PATH, PRAY_FILE_PATH, and PRAY_ENV with CLI equivalents --path, --file-path, and --env or --environment. Precedence is CLI option, process environment, project .env, then defaults.

Prayfile.lock records the selected environment when set. Old lockfiles without environment remain valid.

## Effects

Rust, TypeScript, and Ruby CLIs gained shared project context, dotenv loading, group membership on manifest packages, environment validation, render filtering, and lockfile environment serialization.

SPEC.md, README.md, and JSON schemas were updated for groups and environment fields.

## Next

Run full CI matrix after merge. Document environment switching cleanup for inactive provisioned outputs if additional operator guidance is needed.

## Source

Environment-aware Pray configuration plan in Cursor workspace.
