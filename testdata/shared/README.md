# Shared Prayfile fixture corpus

Fixtures under this tree are inputs (and expected parse slices) exercised by Rust, TypeScript, and Ruby tests. CI already runs each implementation suite; no separate corpus job is required.

## Layout

- `manifest/<case>/Prayfile` — Prayfile text
- `manifest/<case>/expected.json` — destination-focused parse expectations
- `manifest-invalid/<case>/Prayfile` — Prayfile text every implementation must reject

Add cases when a surface or destination contract must stay aligned across runtimes. Prefer small, decision-bearing fixtures over copying full integration trees.

Current cases:

- `compose-tree-file` — compose, tree, and file-bound package surfaces
- `legacy-target` — classic `target` / `output` / unbound package declaration
- `manifest-invalid` — project-relative path boundary violations

Fuller conformance packs also start under `fixtures/` (RFC 0100).
