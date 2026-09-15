# RFC 0106: Host-language lock adapter

- Feature Name: host-language-lock-adapter
- Type: Standards Track
- Status: Experimental
- Created: 2026-09-15
- Author: Andrei Makarov
- Relates: RFC 0002, RFC 0020, RFC 0030, RFC 0109
- Requires: RFC 0020, RFC 0030

## Summary

A host-language adapter reads `Prayfile.lock` and checks dest files named by managed spans. It does not resolve packages or render. Resolve and render stay in the CLI or in `pray_core::embed` resolve and render functions.

## Motivation

RFC 0002 reserved this adapter so a Rake task, Mix task, pytest plugin, or editor can prove dest files still match the lock without fetching packages. Full `verify_project` re-renders. That needs a resolved tree. CI that only needs span checksums should not pay that cost.

## Guide-level explanation

```rust
use pray_core::embed::{inspect_locked_destinations, read_lockfile};

let lockfile = read_lockfile(&std::path::Path::new("Prayfile.lock"))?;
let report = inspect_locked_destinations(std::path::Path::new("."), &lockfile)?;
if !report.is_clean() {
    return Err(report.findings[0].message.clone().into());
}
```

A Python or Elixir caller MAY spawn `pray verify` until it binds these names. It MUST NOT reimplement resolve.

## Reference-level explanation

Key words follow RFC 2119.

An adapter MUST parse `Prayfile.lock` with the same field meaning as RFC 0020. It MUST reject invalid TOML. It MUST NOT fetch packages, write dest files, or change the lock.

`inspect_locked_destinations(project_root, lockfile)` MUST, for each distinct `managed_span.target`:

- report `verify_error` when the dest file is missing under `project_root`;
- report `removed_prayer` when a locked span id has no matching marker pair;
- report `custom_implementation` when the reconstructed body checksum differs from `ideal_checksum`;
- report `orphan_marker` when a dest marker id is not in the lock for that target.

It MUST NOT re-render. It MUST NOT compare package `tree_hash` to a live checkout. Those checks stay on `inspect_project` / `verify_project`.

Marker grammar stays RFC 0030 HTML comment `pray:` ids. Checksums use the same body normalization as render.

## Implementation notes

Rust: `pray_core::embed::inspect_locked_destinations`. Ruby: `Pray.inspect_locked_destinations`. TypeScript: `inspectLockedDestinations`. Unit tests in `crates/pray-core/tests/locked_dest.rs`. RFC 0100 packs under `fixtures/lockfile/span-*` run in all three suites.

## Security considerations

The adapter reads dest files under `project_root`. Dest paths in the lock MUST remain repository-relative (RFC 0033). The adapter MUST use the same dest-file size limit as destination reads.

## Registrar

Function `inspect_locked_destinations`.

## Drawbacks

Two verify paths exist. Full verify still needs resolve. Adapters that skip render will not see renderer drift.

## Rationale and alternatives

Spawn the CLI only: enough for Mix and Rake, weaker for in-process tests. Copy `verify_project` into each language: duplicates render. FFI of the whole crate: later, and still needs this subset named.

## Prior art

Cargo `cargo metadata --offline` reads the lock without a full build. Terraform plan still talks to providers; this adapter is closer to checking recorded state against files on disk.

## Unresolved questions

Whether provisioned exclusive files are in this adapter or stay on full verify. Whether position drift against locked line numbers is in scope without a fresh render.

## Future possibilities

PyO3 and Mix bindings that call only these names.
