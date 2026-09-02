# RFC 0030: Render, markers, and ownership

- Feature Name: render-markers-ownership
- Type: Standards Track
- Status: Stable
- Describes: 1.8.1
- Created: 2026-08-17
- Author: Andrei Makarov
- Relates: RFC 0010, RFC 0020, RFC 0102, RFC 0031, RFC 0034

## Summary

Locked packages become inference-facing files through opaque pray markers, verify, and drift. Ownership zones live in RFC 0031. The model is full reconstruction of managed spans.

## Motivation

If humans and tools edit the same generated file, lock and render cannot be reconstructed.

## Guide-level explanation

Recipe zone: Prayfile, packages, lock; humans edit via CLI. `.agents` zone: human-owned embeds; pray reads and does not overwrite. Managed zone: generated roots and package-owned skills; pray regenerates.

Root files assemble preamble, embedded `.agents` inputs, managed blocks, then an index of names. Humans change shared guidance by editing Prayfile and running pray. Applications edit `.agents/` files.

`pray plan` / `pray apply` review materialization. `pray drift` reports lock, package, span, and renderer differences. `pray apply` refreshes span line numbers and ideal checksums when the user accepts the new render. `pray verify` MUST be read-only.

Same lock, packages, and listed local files MUST yield the same managed bytes.

## Reference-level explanation

This section is the product contract for this concern. Where it disagrees with Implementation notes, Implementation notes record what the reference CLI does today. A follow-on RFC records the gap.

### 39. Render behavior

Render input: Prayfile.lock, resolved package contents, local files, render policy

Render output: INSTRUCTIONS.md, TOOL_B.md, skill directories, command directories, rule files, target-specific files

Render must be deterministic. Same inputs must produce byte-identical outputs.

---

### 40. Generated file header

Rendered target files may include the ignore marker near the beginning of the file:

```md
<!-- pray:0 ignore-comments -->
```

This marker declares that `pray` comments are render markers and should not be interpreted as instruction content.

The marker is advisory for inference behaviour and binding for Prayfile tooling.

Generated files should not include: timestamps, hostnames, absolute paths, random IDs, or full package graphs unless requested.

The Agent context banner defaults on `AGENTS.md` only (RFC 0108). Other compose dests opt in with `header: true`.

---

### 41. Pray markers

Rendered targets cite lockfile with compact markers; they must not duplicate the dependency graph, hashes, or provenance already in `Prayfile.lock`.

Markdown canonical form (same opaque id opens and closes; own lines; not nested; unmatched invalid):

```md
<!-- pray:p7f3k9m2 -->

...rendered content...

<!-- pray:p7f3k9m2 -->
```

Marker ID: opaque, lowercase ASCII letters and digits, 8–16 characters. Must not encode package, topic, version, hash, path, or labels.

Each id maps to a managed span record in `Prayfile.lock` (ideal checksum, open/close lines). Tooling ignores pray comments for semantic hashes; may also track exact file hashes including markers.

```text
semantic hash  = rendered content without pray markers
file hash      = exact target file bytes including pray markers
```

---

### 42. Churn-minimal rendering

`render churn: :minimal`: stable package order and headings; no timestamps, rewrap, generated package tables, or blank-line noise; preserve local text; normalize line endings only when configured. Prefer small roots and separate skills/templates with stable markers. Avoid giant concatenated instruction files.

---

### 43. Local files

Local files are human-owned.

```
local ".agents/project.md", position: :after
```

Rules:

- store human-owned project context under `.agents/` (for example `.agents/project.md`), not under alternate trees such as `agent/local/`
- never overwritten by pray on disk
- content is re-embedded into rendered root files on each render run
- agents edit local source files, not the embed copy inside INSTRUCTIONS.md
- missing local files are errors unless optional
- paths must stay inside repository unless explicitly allowed
- not package dependencies
- not version locked
- not copied into lockfile

Optional local file:

```
local ".agents/private.md", optional: true
```

Local file hashes may be stored in `.pray/state.json`, not Prayfile.lock.

---

### 44. Render modes

The only render mode is `managed`. Generated files are owned by pray. Edits inside markers fail on write when a previous lock exists. Unmarked text is kept.

Dry-run is `pray plan` or `pray render --check`. Those commands are not a `render mode`. Offline copies are `pray vendor` into `.pray/vendor`. A personal dest is an in-root `compose`, `tree`, or `file:` path, not `mode: :local`.

Parsers MUST reject any `render mode` other than `managed` (RFC 0034).

---

### 45. Conflict policy

Default and only supported value:

```manifest
render conflict: :fail
```

Parsers MUST reject any other value.

On write, with a previous lock, checksum on-disk managed marker bodies against lock `ideal_checksum`. A human edit inside markers is a render error. Unmarked text is kept by `render_patch`. Recovery is `pray verify` / `pray drift`, then `pray apply`.

Package-versus-package compose collision is a different concern. RFC 0031 golden rule 4: there is no three-way merge in v1. Managed blocks are nobody-edits.

---

### 46. Target adapters

Unused (RFC 0034). Destination DSL already names compose, tree, and `file:` paths. Implementations MUST NOT load adapter TOML to map paths or to spell markers. `spec.adapters` MAY parse and MUST stay inert.

---

### 47. Root context strategy

Root files should stay small.

Recommended root file shape:

```markdown
<!--

Edit `Prayfile`, not this file.
-->
## Agent context
### Additional instructions
...
### Shared instructions
...
### Available capabilities
- code-review
- schema-migration
```

Do not dump every capability body into root files if target supports capabilities.

---

### 48. Example generated instruction file

```markdown
<!-- pray:0 ignore-comments -->

## Input context

Do not edit managed blocks in `AGENTS.md` or skills under `.agents/`.
To change shared guidance, update `Prayfile` and run `pray`.

<!-- pray:l3m8n2p4 -->

### Additional instructions
This repository uses a web stack, test framework, UI library, and relational database.

<!-- pray:l3m8n2p4 -->

### Shared instructions

<!-- pray:p7f3k9m2 -->

#### Testing basics
Prefer focused tests near the changed code before broad suites.

<!-- pray:p7f3k9m2 -->

<!-- pray:q8g4h1j6 -->

#### Web application review
Check migrations, callbacks, authorization boundaries, background jobs, and data consistency.

<!-- pray:q8g4h1j6 -->

### Available skills
- code-review
- schema-migration
```

---

### 49. Example generated capability

Path: `generated/capabilities/code-review.md`

```markdown
## Code review
### Purpose
Review application changes for correctness, maintainability, and safety.
### When to use
Use when a task changes application code, data models, migrations, jobs, services, or tests.
### Process
1. Inspect changed files.
2. Identify behavior changes.
3. Check tests.
4. Check data and authorization boundaries.
5. Report risks before proposing broad rewrites.
```

---

### 52. Verify checks

`pray verify` is read-only. It must not modify `Prayfile.lock` or target files.

#### Managed span checks

For each `[[managed_span]]` record:

- opening and closing markers with `id` exist in `target`
- managed body semantic checksum equals `ideal_checksum`
- current `open_line` and `close_line` equal lockfile line positions
- report removed prayer when lock record exists but marker pair is absent
- report custom implementation when markers exist but body checksum ≠ `ideal_checksum`
- report position drift when body checksum matches `ideal_checksum` but line positions differ

#### Repository checks

`pray verify` should also detect:

- missing Prayfile.lock
- lockfile incompatible with manifest
- package hash mismatch
- missing package source
- missing local files
- orphan pray marker pairs with no lockfile managed span record
- manual edits outside allowed marker regions in managed root files
- duplicate skill names
- unsupported target
- unresolved package source
- path traversal attempt
- vendored package mismatch

Strict mode: `pray verify --strict` turns warnings into errors.

---

### 53. Drift behavior

`pray drift` includes all `pray verify` managed span checks and adds renderer comparison.

Required drift kinds:

- `custom_implementation`: Marker pair present; body checksum ≠ `ideal_checksum`
- `removed_prayer`: Lock record present; marker pair absent
- `position_drift`: Body checksum = `ideal_checksum`; marker lines moved (one finding per target; see section 32.2)
- `renderer_drift`: On-disk state matches lock; fresh render would change ideals or spans
- `orphan_marker`: Marker pair present; no lock managed span record

Required report sections: Lockfile changes, Package changes, Managed span changes, Rendered file changes, Removed prayers, Orphan markers, Warnings

Semantic diff: `pray drift --semantic`

Example output:

```
managed_span q8g4h1j6 INSTRUCTIONS.md
  kind: custom_implementation
  ideal_checksum: sha256:789abc...
  actual_checksum: sha256:111222...
managed_span p7f3k9m2 INSTRUCTIONS.md
  kind: removed_prayer
  expected lines: 14-20
renderer_drift
  sample/webapp 2.1.4 -> 2.1.5 would change 2 managed spans
```

`pray drift` must not refresh the lockfile. Run `pray apply` to accept intentional materialization and refresh managed span records.

## Implementation notes

`render_compose`, `render_write`, `render_provisioned`, `render_patch`, `render_conflict`. Verify splits format, integrity, and position modules. Parsers reject any conflict value other than `fail` (`manifest_validate.rs`). On write, `render_conflict` checksums managed marker bodies against lock `ideal_checksum`.

`crates/pray-core/src/hashing.rs` `marker_id` takes the first eight hex characters of SHA-256 of a seed. `verify/mod.rs` `parse_marker` accepts `<!-- pray:` plus lowercase ASCII letters or digits plus ` -->`, with no length check. The ignore form is `<!-- pray:0 ignore-comments -->`.

Group membership filters which spans are rendered for a selected environment; lock still lists all packages (RFC 0020).

## Security considerations

A rewritten span is an integrity failure. Silence flags on spans are explicit; they MUST appear in the lock.

## Drawbacks

Opaque ids make `git blame` of policy harder without the lock. Provenance lives in the lock.

## Unresolved questions

Names `warn`, `append`, `last_wins`, and `target_specific` appeared in an earlier snapshot. They are unspecified merge policy. Keep them here until an Experimental RFC specifies conflict kinds (folder-path exclusivity from RFC 0011, same section id, duplicate skill) with algorithms and fixtures. RFC 0102 is destination DSL and is not the design home.

`warn` would skip the checksum reject and overwrite anyway. `append` / `last_wins` would concatenate or drop package bytes and break lock reconstructability. `target_specific` needs a second grammar.

Marker id generation stability across equivalent renders (reserved RFC 0110).
