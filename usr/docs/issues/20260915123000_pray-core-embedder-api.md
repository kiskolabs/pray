# pray-core embedder API audit

## Participants

Andrei Makarov

## Decisions

Name the Rust library surface as pray_core::embed (RFC 0109). Keep sibling pub modules for the in-tree CLI. Add parse_lockfile, serialize_lockfile, and lockfile_hash as aliases over existing lockfile parse and serialize. Do not split crates. Do not hide registry, ssh, or CLI helper modules in this pass.

## Effects

Audit of crates/pray-core/src/lib.rs counted 40 public modules plus error re-exports. pray-cli imports client_trust, registry, ssh, hashing, and CLI helpers from those modules. Ruby Pray and TypeScript package exports already list parse, resolve, render, and verify. Rust had no listed subset. RFC 0101 unresolved question on public API is now pointed at RFC 0109.

Resource and budget: no new IO path; re-exports and lockfile parse aliases only. Shipped crate size change is unmeasured until cargo package. Inference: adding embed.rs is small relative to the 1.15.0 package (151 files, 798.0KiB). Confirming check: cargo package -p pray-core --allow-dirty after this change.

Trace and identification: skipped extra telemetry. parse_lockfile reads caller-supplied text. resolve_project still fetches as before. No new identifiers.

Boundary and control: skipped. This pass does not add actuators or a new command path.

Product surface: skipped. Library module, no presentation.

Privacy: skipped. No new person data.

Performance: skipped. No latency change claimed.

Observability: skipped. No new runtime.

Security: embedder calls the same filesystem and network paths as the CLI. RFC 0109 tells callers to keep RFC 0050 path safety. No new auth.

Contract: RFC 0109 lists names. parse_lockfile rejects invalid TOML with PrayError::Parse kind lockfile.

Learned systems: skipped.

EA-001. Severity medium. Confidence high. Location crates/pray-core/src/lib.rs public modules. Kind observed. Why it matters: crates.io consumers had no listed embedder names and could pin CLI helpers. Smallest credible fix applied: RFC 0109 plus pray_core::embed re-exports. Deeper fix: later RFC makes sibling modules pub(crate).

Missing coverage: no test imported only the listed names. Futile coverage: none claimed. Test added: crates/pray-core/tests/embedder_api.rs and lockfile parse round-trip in lockfile_unit.rs.

## Next

RFC 0100 fixture packs for lock and render so the same verbs cannot drift across Rust, Ruby, and TypeScript. Then RFC 0106 lock adapter as a subset of embed names.

## Source

rfcs/0101-crate-boundaries.md
rfcs/0109-embedder-api.md
crates/pray-core/src/lib.rs
crates/pray-core/src/embed.rs
rubygems/pray-cli/lib/pray.rb
npmjs/pray-cli/src/index.ts
usr/docs/changelogs/20260915123000_pray-core-embedder-api.md
