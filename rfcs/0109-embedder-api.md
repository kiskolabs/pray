# RFC 0109: pray-core embedder API

- Feature Name: embedder-api
- Type: Standards Track
- Status: Experimental
- Created: 2026-09-15
- Author: Andrei Makarov
- Relates: RFC 0070, RFC 0101
- Requires: RFC 0101

## Summary

`pray-core` publishes a documented embedder module. Callers that parse, resolve, lock, render, or verify import `pray_core::embed`. Other public modules stay available to the in-tree CLI until a later RFC shrinks them.

## Motivation

RFC 0101 left public API stability unresolved. crates.io already publishes `pray-core` with dozens of public modules, including CLI helpers. An embedder has no listed names. Ruby and TypeScript already export a smaller library surface. Rust embedders copy CLI imports and break when a helper moves.

## Guide-level explanation

```toml
[dependencies]
pray-core = "1.15"
```

```rust
use pray_core::embed::{
    parse_lockfile, parse_manifest, resolve_project, render_project, verify_project,
};

let manifest = parse_manifest(&std::fs::read_to_string("Prayfile")?)?;
let _ = (manifest, parse_lockfile, resolve_project, render_project, verify_project);
```

`default_manifest_path` and `default_lockfile_path` join `Prayfile` and `Prayfile.lock` under a directory. CLI invocation context stays in `pray-cli`.

## Reference-level explanation

Key words follow RFC 2119.

An implementation that claims the Rust embedder API MUST export the names in `pray_core::embed` with the same signatures as this workspace. New embedders SHOULD import that module. They MAY keep using sibling modules while those remain `pub`.

Supported functions: `parse_manifest`, `read_manifest_text`, `parse_package_spec`, `parse_lockfile`, `serialize_lockfile`, `lockfile_hash`, `read_lockfile`, `write_lockfile`, `write_lockfile_if_changed`, `lockfiles_equivalent`, `build_lockfile`, `resolve_project`, `resolve_project_with_options`, `project_root_from_manifest`, `render_project`, `write_rendered_targets`, `inspect_project`, `inspect_locked_destinations`, `verify_project`, `drift_project`, `default_manifest_path`, `default_lockfile_path`.

Supported types: `PrayError`, `PrayResult`, `Manifest`, `ManifestPackage`, `ManifestSource`, `ManifestTarget`, `ManifestLocal`, `RenderPolicy`, `PackageSpec`, `Lockfile`, `LockSource`, `LockedPackage`, `LockedTarget`, `ManagedSpanRecord`, `ProvisionedFileRecord`, `ResolvedProject`, `ResolvedPackage`, `ResolvedLocalFile`, `ResolveOptions`, `RenderedTarget`, `VerificationReport`, `VerificationFinding`.

`parse_lockfile` MUST reject invalid TOML with `PrayError::Parse` and kind `lockfile`. `serialize_lockfile` MUST write the canonical field order used by `Lockfile::serialized`. Host-language lock adapters (RFC 0106) MAY use only lock and verify names.

Struct fields stay public. A later RFC MAY freeze or hide fields. Until then a field addition is a minor change and a field rename or type change is a major change for crates.io.

## Implementation notes

`crates/pray-core/src/embed.rs` re-exports existing functions. `parse_lockfile`, `serialize_lockfile`, and `lockfile_hash` are thin aliases over lockfile parse and `Lockfile::serialized` / `file_hash`.

## Security considerations

The embedder API performs filesystem and network IO on resolve, render, and lock write. Callers MUST apply the same path-safety and hash-verify rules as the CLI (RFC 0050). This RFC does not add a sandbox.

## Registrar

Module path `pray_core::embed`. Function names listed above.

## Drawbacks

A second import path exists for the same functions. CLI code can keep using sibling modules, so the crate surface stays large.

## Rationale and alternatives

Split crates now: RFC 0101 already rejected that without a second consumer. Hide every module except `embed`: a breaking crates.io change for the in-tree CLI and any out-of-tree caller of `pray_core::registry`. Wrap the `pray` binary: RFC 0070 already rejected that for native packaging.

## Prior art

Ruby `Pray.parse_manifest` and TypeScript `parseManifest` export the same verbs. Cargo exposes `cargo` as a CLI and `cargo` crate internals separately; this RFC names the library verbs without a crate split.

## Unresolved questions

When sibling `pub` modules become `pub(crate)`. Whether PyO3 or other FFI binds `embed` only. Whether struct fields get a frozen subset.

## Future possibilities

RFC 0106 lock adapter as a subset of these names. RFC 0100 fixtures as the behavior contract for the same verbs across languages.
