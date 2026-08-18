# RFC 0100: Conformance and polyglot fixtures

- Feature Name: conformance-polyglot
- Type: Standards Track
- Status: Experimental
- Created: 2026-08-17
- Author: Andrei Makarov
- Relates: RFC 0070, RFC 0010
- Requires: RFC 0001

## Summary

Rust, Ruby, and TypeScript CLIs, and any later implementation, MUST fail the same fixtures. RFC 0070 sketches levels and a `fixtures/` tree. The tree is too small to carry that claim.

## Motivation

`testdata/shared/manifest/` has two cases. `fixtures/` has one parser Prayfile and one prayspec. Three shipped CLIs can drift on resolve, render, or verify without a shared expected lock and expected bytes.

Absorbed contract text does not yet use RFC 2119 keywords uniformly.

## Guide-level explanation

A Level N implementation MUST pass every fixture directory labeled for that level. Extra CLI verbs MAY exist; they MUST use exit 8 when unsupported.

A new implementation runs `fixtures/parser/minimal-prayfile` and compares `expected.json`. CI for this repo runs the same path for Rust, Ruby, and TypeScript. Schema files under `schema/` validate the JSON view of lock and manifest.

Grow `testdata/shared` for parse slices. Keep fuller packs under `fixtures/`.

This RFC stays Experimental until the packs listed below exist and CI runs them.

## Reference-level explanation

This section is the product contract for this concern. Where it disagrees with Implementation notes, Implementation notes record what the reference CLI does today. A follow-on RFC records the gap.

### 72. Conformance levels

- 0: Parser: Can parse Prayfile and *.prayspec; `pray manifest`
- 1: Installer: Can resolve local/path/tarball packages and produce lockfile; `pray install`, `pray verify`
- 2: Renderer: Can render at least one target; `pray render`, `pray render --check`
- 3: Package manager: Supports distribution point, Git, update, drift, verify; `pray update`, `pray drift`, `pray verify`
- 4: Publisher: Can pack and publish packages to static registry; `pray package`, `pray publish`

The reference implementation should target Level 3 first.

---

### 73. Test fixtures

The open specification should include conformance fixtures:

```
fixtures/
  parser/
  prayspec/
  resolver/
  lockfile/
  render/
  verify/
  registry/
  packages/
```

Each fixture should contain: input Prayfile, input prayspec files, package sources, expected canonical manifest, expected Prayfile.lock, expected rendered files, expected diagnostics

This allows independent implementations.

The reference tree also keeps a smaller shared corpus at `testdata/shared/manifest/` (Prayfile plus destination-focused `expected.json`) exercised by Rust, TypeScript, and Ruby CI suites. Grow that tree for cross-runtime parse contracts; reserve fuller conformance packs for the `fixtures/` layout above.

---

## Implementation notes

No resolver, render, or verify fixture packs yet.

## Drawbacks

Fixture maintenance. Prefer small decision-bearing cases.

## Rationale and alternatives

One official binary only: simpler, abandons native gem/npm installs. Snapshot tests per language: they already exist and still drift. This RFC adds a language-neutral expected tree.

## Unresolved questions

Who is source of truth when Rust and Ruby disagree on an undocumented edge: RFC 0070 says Rust until this matrix exists; after Stable, the fixture wins.

Whether lockfile expected files are TOML, JSON, or both.
