# RFC 0101: Crate boundaries

- Feature Name: crate-boundaries
- Type: Informational
- Status: Stable
- Describes: 1.8.1
- Created: 2026-08-17
- Author: Andrei Makarov
- Relates: RFC 0070

## Summary

A former crate diagram suggested many crates. The workspace uses `pray-core` plus CLI, transport, and bench. This RFC analyzes that split. No crate move is authorized until a follow-on Standards Track RFC lists the public API.

## Motivation

Contributors grep for `pray-render` and miss `render_compose.rs`. Transport types exist in two places: `pray-core` registry modules and `pray-transport`. Duplicate HTTP/SSH/torrent paths will diverge.

## Guide-level explanation

Add behavior in `pray-core` modules under 150–300 LOC files. `pray-cli` stays command parsing, help, serve, publish, and user reports. `pray-transport` holds transport traits and adapters.

A new crate needs a second in-tree consumer or a documented embedder API. Moving code keeps behavior proven by existing tests first.

A future split is justified when compile times hurt, when embedders need resolve without serve, or when transport must be usable without the DSL parser.

## Implementation notes

`pray-core/src/lib.rs` lists parser, resolve, render, verify, registry, ssh, torrent. `crates/pray-transport/src/lib.rs` exports `FederationTransport`, `HttpTransport`, `P2PTransport`, `SshTransport`, `TorrentTransport`.

Smallest fix: pick one owner per transport. Recommendation: traits and adapters in `pray-transport`; `pray-core` calls the registry.

## Rationale and alternatives

Split everything now to match that diagram: large diff, no new user behavior. Ignore those crate names: readers stay confused. This RFC updates the story: modules inside `pray-core` are the v1 architecture; those crate names become a future map.

## Unresolved questions

Public Rust API stability for `pray-core`. Whether Ruby/TS will ever call into Rust via FFI (if yes, crate split and C ABI are a different RFC).
