# Shared Prayfile fixture corpus

Fixtures under this tree are inputs (and expected parse slices) exercised by Rust, TypeScript, and Ruby tests. CI already runs each implementation suite; no separate corpus job is required.

## Layout

- `manifest/<case>/Prayfile` — Prayfile text
- `manifest/<case>/expected.json` — destination-focused parse expectations
- `manifest-invalid/<case>/Prayfile` — Prayfile text every implementation must reject
- `registry-cache/<case>.json` — source identity and expected project-local cache path
- `package-tree/<case>.json` — package files and expected normalized tree hash

Add cases when a surface or destination contract must stay aligned across runtimes. Prefer small, decision-bearing fixtures over copying full integration trees.

Current cases:

- `compose-tree-file` — compose, tree, and file-bound package surfaces
- `legacy-target` — classic `target` / `output` / unbound package declaration
- `manifest-invalid` — project-relative path boundary violations
- `registry-cache/identity-first` — namespaced package cache identity shared by every CLI
- `package-tree/byte-order` — UTF-8 byte ordering shared by every CLI

Fuller conformance packs also start under `fixtures/` (RFC 0100).
