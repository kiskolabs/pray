# RFC 0020: Resolve and lock

- Feature Name: resolve-and-lock
- Type: Standards Track
- Status: Stable
- Describes: 1.8.1
- Created: 2026-08-17
- Author: Andrei Makarov
- Relates: RFC 0010, RFC 0040, RFC 0050

## Summary

Declared packages become Prayfile.lock through version constraints, source kinds, hash verification, install flags, update, and remove. Resolution is the supply-chain boundary.

## Motivation

A lock that retargets because a tool is installed locally, or because a yanked version appeared, would break auditability.

## Guide-level explanation

`pray install` uses the lock if it satisfies the manifest, otherwise resolves and writes it, then fetches, verifies, and renders. `--locked` MUST fail if the lock would change. `--frozen` MUST also fail if generated files would change. `--offline` MUST NOT touch the network.

`pray update` re-resolves within constraints; `--major` is required to cross a major bound. `pray remove` edits the manifest, re-resolves, and drops managed output for that package.

Yanked versions: new resolve skips them; an existing lock may keep them with a warning; `pray install --strict` fails (changelog 1.8.0).

Tool discovery MUST NOT change resolution unless explicitly requested.

## Reference-level explanation

This section is the product contract for this concern. Where it disagrees with Implementation notes, Implementation notes record what the reference CLI does today. A follow-on RFC records the gap.

### 31. Lockfile

Prayfile.lock is machine-authored.

Recommended format: TOML.

Reasons: readable, stable, small diffs, easy to parse, good for sorted package tables

Users should not edit Prayfile.lock by hand.

#### 31.1 Canonical verification records

Prayfile.lock may include canonical verification records that bind claims to package, render, or confession identities. Verification records are machine-authored and should be stable across implementations.

Recommended format: TOML tables.

Required fields:

- `kind`: Verification subject type: `package`, `render_plan`, `render_output`, or `confession`
- `subject`: Stable subject reference such as package name and version, managed span ID, confession ID, or artifact reference
- `subject_hash`: Expected hash for the subject being verified
- `verifier`: Identity of the client or server that performed the verification
- `method`: Verification method such as `hash`, `signature`, `manual`, `heuristic`, `local_model`, `cloud_model`, or `rule`
- `policy`: Policy or trust rule reference used during verification
- `input_hash`: Hash of the inputs used to produce the claim
- `observed_hash`: Hash actually observed during verification
- `observed_at`: Verification timestamp
- provenance: Origin, source, or federation path for the claim
- `signature`: Optional signature over the canonical record

Render-output records should bind the final injected bytes. Render-plan records should also record selected exports, exclusions, ordering, normalization, and target policy in their provenance or detail fields. Confession records should bind the confession body to the sender, package reference, and replay-prevention data.

---

### 32. Lockfile example

```toml
prayfile_lock = "1"
spec = "0.1"
generated_by = "pray 0.1.0"
manifest_hash = "sha256:..."

[[source]]
name = "default"
kind = "registry"
url = "https://agents.example.com"

[[source]]
name = "sample"
kind = "git"
url = "git+ssh://git@example.com/agent-context/index.git"

[[package]]
name = "sample/base"
version = "1.4.3"
source = "sample"
tree_hash = "sha256:..."
artifact_hash = "sha256:..."
artifact = "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg"
exports = [
  "working-agreements",
  "testing-basics",
  "security-basics",
]

[[package]]
name = "sample/webapp"
version = "2.1.5"
source = "sample"
tree_hash = "sha256:..."
artifact_hash = "sha256:..."
artifact = "v1/artifacts/sample/webapp/2.1.5/sample-webapp-2.1.5.praypkg"
dependencies = [
  "sample/base",
]
exports = [
  "webapp-review",
  "data-layer",
  "testing",
  "live-pages",
]

[[target]]
name = "tool_a"
outputs = [
  "INSTRUCTIONS.md",
  ".tool-a/",
]

[[managed_span]]
id = "p7f3k9m2"
target = "INSTRUCTIONS.md"
open_line = 14
close_line = 20
ideal_checksum = "sha256:abc123..."
package = "sample/base"
export = "testing-basics"
source_checksum = "sha256:def456..."
silenced = false

[[managed_span]]
id = "q8g4h1j6"
target = "INSTRUCTIONS.md"
open_line = 24
close_line = 30
ideal_checksum = "sha256:789abc..."
package = "sample/webapp"
export = "webapp-review"
source_checksum = "sha256:012def..."
silenced = false

[[verification_record]]
kind = "package"
subject = "sample/webapp@2.1.5"
subject_hash = "sha256:..."
verifier = "prayers.kisko.dev"
method = "signature"
policy = "registry-default"
input_hash = "sha256:..."
observed_hash = "sha256:..."
observed_at = "2026-06-29T14:07:56Z"
provenance = "registry"
signature = "ed25519:..."

[[verification_record]]
kind = "render_plan"
subject = "INSTRUCTIONS.md#p7f3k9m2"
subject_hash = "sha256:..."
verifier = "pray 0.1.0"
method = "rule"
policy = "render-managed"
input_hash = "sha256:..."
observed_hash = "sha256:..."
observed_at = "2026-06-29T14:07:56Z"
provenance = "sample/base -> INSTRUCTIONS.md; exports=testing-basics; exclusions=[]"

[[verification_record]]
kind = "render_output"
subject = "INSTRUCTIONS.md#p7f3k9m2"
subject_hash = "sha256:..."
verifier = "pray 0.1.0"
method = "hash"
policy = "render-managed"
input_hash = "sha256:..."
observed_hash = "sha256:..."
observed_at = "2026-06-29T14:07:56Z"
provenance = "final injected bytes"

[[verification_record]]
kind = "confession"
subject = "sample/webapp@2.1.5"
subject_hash = "sha256:..."
verifier = "example-maintainer"
method = "signature"
policy = "confession-default"
input_hash = "sha256:..."
observed_hash = "sha256:..."
observed_at = "2026-06-29T14:07:56Z"
provenance = "publisher"
signature = "ed25519:..."

[[target]]
name = "tool_b"
outputs = [
  "NOTES.md",
  ".tool-b/",
]
```

---

### 32.1 Managed span records

Each managed span (a prayer between pray markers) must have a lockfile record.

Required fields:

- `id`: Opaque pray marker ID
- `target`: Target file path
- `open_line`: Line number of opening marker
- `close_line`: Line number of closing marker
- `ideal_checksum`: Semantic hash of managed body between markers
- provenance: Package, export, source fragment checksum, silenced flag

`ideal_checksum` is computed from the managed body only:

- exclude opening and closing pray marker comment lines
- normalize line endings according to target policy
- apply the same semantic hashing rules as RFC 0030 (pray comments ignored for semantic hash)

`open_line` and `close_line` are 1-based line numbers in the target file after materialization.

Managed span records are updated by `pray apply`, `pray install`, and other materialization commands that explicitly refresh render state. They are not updated by read-only commands.

---

### 32.2 Verify and drift contract

#### `pray verify`

Read-only. Compare on-disk target files to managed span records.

For each lockfile managed span:

1. locate the marker pair by `id` in `target`
2. fail if either marker is missing (removed prayer)
3. compute semantic checksum of the managed body
4. compare body checksum to `ideal_checksum` (custom implementation when different)
5. compare current marker line numbers to `open_line` / `close_line` (position drift when checksum matches but lines differ)

`pray verify` reports mismatches. It must not modify `Prayfile.lock` or target files.

Also checks lockfile integrity, package checksums, signatures, cache validity, confession references, and any recorded render digests or annotation provenance.

#### `pray apply`

Materializes planned changes, then refreshes managed span records:

- rewrite target files when needed
- recompute `ideal_checksum` for each managed span
- recompute `open_line` and `close_line`
- add, update, or remove managed span records

#### `pray drift`

Superset of verify. Reports:

- `custom_implementation`: Marker pair exists, but body checksum ≠ `ideal_checksum`
- `removed_prayer`: Lockfile record exists, marker pair missing from target
- `position_drift`: Body checksum matches `ideal_checksum`, but marker lines moved. Report one finding per target: group uniform shifts, cite first marker lock vs file lines, and when unmarked preamble differs from fresh composition cite `path:line` cause (prefer compose local source when attributable).
- `renderer_drift`: On-disk file matches lock, but fresh render from current inputs would change ideals
- `orphan_marker`: Marker pair in target file has no lockfile managed span record

`pray drift` does not refresh the lockfile.

---

### 33. Lockfile churn rules

To reduce git churn, lockfiles must avoid:

timestamps, absolute paths, local usernames, hostnames except declared sources, cache paths, random IDs, fetch duration, OS-specific path separators, generated file content duplication, machine-specific tool discovery

Stable ordering:

- sources sorted by name
- packages sorted by name/source/version
- targets sorted by name
- arrays sorted unless order is semantic

The lockfile should record: manifest hash, resolved package versions, source identity, artifact hashes, tree hashes, selected exports, dependency graph, and managed span records (ideal checksums and marker line positions per prayer).

Per-target `render_hash` may summarize an entire output file. Managed span records are the authoritative per-prayer contract for verify and drift.

It should not duplicate full generated file content. Strict audit mode may optionally record per-file hashes in addition to managed span records.

---

### 34. Manifest hash

`manifest_hash` is a normalized hash of Prayfile.

Normalization process:

1. parse DSL
2. convert to canonical manifest model
3. sort unordered fields
4. preserve semantically meaningful order
5. serialize canonical model
6. hash serialized bytes

Whitespace-only changes should not change `manifest_hash`.

Comment-only changes should not change `manifest_hash`.

---

### 35. Resolver behavior

Resolver input: Prayfile, existing Prayfile.lock if present, available sources, target list from manifest, package metadata, cache

Resolver output: resolved package graph, selected versions, selected exports, source identities, artifact hashes, tree hashes, target render plan, Prayfile.lock

Resolution rules:

1. Read manifest.
2. Validate syntax.
3. Load existing lockfile if present.
4. Prefer locked versions when they satisfy manifest constraints.
5. Resolve unlocked or changed packages.
6. Resolve transitive dependencies.
7. Reject incompatible versions.
8. Reject missing exports.
9. Reject incompatible targets unless optional.
10. Fetch package artifacts.
11. Verify artifact hash.
12. Verify normalized tree hash.
13. Write lockfile only if resolution changed.

---

### 36. Install behavior

#### pray install

Default behavior:

- if lockfile exists and satisfies manifest, use it
- if lockfile missing, resolve and create it
- if manifest changed, minimally re-resolve only necessary packages
- fetch packages
- verify packages
- render target files

#### pray install --locked

- require existing Prayfile.lock
- fail if lockfile needs update
- fetch and verify packages
- render only from locked state

#### pray install --frozen

- same as `--locked`
- fail if generated files are stale
- fail if verify checks fail
- intended for CI

#### pray install --offline

- use cache or vendor directory only
- no network access
- fail if packages unavailable locally

---

### 37. Update behavior

```
pray update
pray update sample/webapp
```

Updates all packages within manifest constraints, or selected package and only dependencies required by that update.

Default update should minimize churn.

Update summary should show: package name, old version, new version, source, exports affected, targets affected, rendered files affected, warnings

Major updates should require explicit intent:

```
pray update sample/webapp --major
```

---

### 38. Remove behavior

```
pray remove sample/webapp
```

Expected behavior:

- remove package declaration from Prayfile
- re-resolve dependency graph
- update Prayfile.lock
- remove generated sections/files no longer needed
- preserve local files
- show diff

---

## Implementation notes

Modules: `resolve`, `resolve_deps`, `resolve_exports`, `resolve_git*`, `resolve_queue`, `constraint`, `dependency_graph`. Cycle rejection shipped in changelog 1.6.0.

Fetch and registry HTTP/SSH/torrent live in `crates/pray-core` (`fetch.rs`, `registry_*.rs`) and in `crates/pray-transport`.

`pray unlock` exists on the CLI (RFC 0040). Inference: document `pray unlock` here or remove the verb.

## Security considerations

Fail closed on hash mismatch. Path traversal in archives MUST be rejected (unpack tests in changelog 1.6.0). Unsigned remote packages MAY be refused when `pray trust set-require-signed-packages` is set (RFC 0050).

## Unresolved questions

How git revision pinning and `host_key_fingerprint` on lock sources interact with SSH transport auth versus package signatures.

Whether minimal re-resolve of only affected packages matches `resolve_queue` under tests.
