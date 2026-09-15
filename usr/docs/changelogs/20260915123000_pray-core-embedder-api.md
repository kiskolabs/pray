# pray-core embedder API

## Participants

Andrei Makarov

## Decisions

Publish RFC 0109 listing pray_core::embed. Re-export parse, resolve, lock, render, and verify names. Add parse_lockfile, serialize_lockfile, and lockfile_hash aliases. Keep other pub modules for the CLI.

## Effects

crates/pray-core/src/embed.rs re-exports the listed names. lockfile.rs parse_lockfile shares the TOML error path with read_lockfile. crates/pray-core/README.md shows the embed import. CHANGELOG.md Unreleased names the library surface.

## Next

Grow RFC 0100 lock and render fixtures. Claim RFC 0106 when a host-language lock reader is specified.

## Source

rfcs/0109-embedder-api.md
usr/docs/issues/20260915123000_pray-core-embedder-api.md
crates/pray-core/src/embed.rs
crates/pray-core/tests/embedder_api.rs
