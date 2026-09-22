# Optional auth SQLite storage

## Participants

Andrei Makarov and Cursor.

## Decisions

The `auth` feature remains enabled by default in pray-core and the pray CLI. It enables rusqlite with bundled SQLite for the registry authentication store.

Slim CLI builds disable default features. They retain remote `pray login` and ordinary install, plan, apply, and render workflows, while omitting `pray serve`.

`PRAY_SESSION_TOKEN` requires local auth storage. A slim build reports that it was compiled without auth support instead of silently ignoring the token.

## Effects

pray-core and pray now compile without rusqlite when default features are disabled. Server and SQLite-dependent integration tests require the auth feature.

Validated with `cargo test -p pray-core`, `cargo test -p pray-core --no-default-features`, `cargo test -p pray --no-default-features`, and `cargo test -p pray`. The CLI test commands set `commit.gpgsign=false` through the environment because the host Git signing agent was unavailable.

`cargo build -p pray --release` produced a 7.6 MB binary. `cargo build -p pray --release --no-default-features` produced a 5.7 MB binary. `cargo tree -p pray --no-default-features -i rusqlite` reported no matching package, confirming the slim dependency graph excludes rusqlite.

## Source

Local branch patch/optional-auth-sqlite.
