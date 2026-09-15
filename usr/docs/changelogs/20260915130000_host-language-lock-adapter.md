# Host-language lock adapter

## Participants

Andrei Makarov

## Decisions

Specify RFC 0106. Export inspect_locked_destinations from pray_core::embed. Skip Python and Elixir packages until a caller exists.

## Effects

Rust inspect_locked_destinations reports missing dest files, edited spans, and orphan markers from Prayfile.lock without resolve.

## Next

Ruby and TypeScript ports. Mix task and PyO3 bind when requested.

## Source

rfcs/0106-host-language-lock-adapter.md
usr/docs/issues/20260915130000_host-language-lock-adapter.md
crates/pray-core/src/verify/locked_dest.rs
