# madge removed; minimatch development advisory cleared

## Dependency

Former: `madge` 8.0.0 as a development dependency of `npmjs/pray-core` and `npmjs/pray-cli`, pulling `minimatch` through `dependency-tree` / `precinct` / `@typescript-eslint/typescript-estree`.

Replacement: `npmjs/scripts/check-import-cycles.mjs` wired to `lint:cycles` in both packages.

## Symptom

`npm audit` (including development) reported seven moderate findings for GHSA-g3hv-75wg-m249 via the madge graph. Production `npm audit --omit=dev` was already clean.

## Evidence

- `node --test npmjs/scripts/check-import-cycles.test.mjs` passed: synthetic A->B->A cycle detected; pray-core and pray-cli entrypoints reported no cycles (parity with prior madge runs).
- After removing madge and refreshing the lockfile, `npm audit` and `npm audit --omit=dev` both reported found 0 vulnerabilities.
- `npm install` reported removed 135 packages relative to the madge graph.

## Suggested fix

Landed in-tree: drop madge; use the local relative-import cycle checker for lint.

## Next

- Optional: spike TypeScript major versions now that the madge peerOptional constraint is gone.
- Mark older minimatch and madge-peer notes as superseded by this record when reviewing the dependencies tree.

## Source

Upstream advisory: GHSA-g3hv-75wg-m249. Prior assessments: `docs/dependencies/20261102115928_npm-minimatch-development-graph-advisory.md`, `docs/dependencies/20260829222001_madge-typescript-peer.md`. Downstream: `docs/issues/20261106100000_dependency-optimization-audit.md`.
