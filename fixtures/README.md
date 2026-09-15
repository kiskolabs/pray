# Conformance fixtures

Layout follows RFC 0100. Each pack holds inputs and expected diagnostics or lock/render slices for independent implementations.

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

Start packs here as contracts stabilize. Prefer small, decision-bearing cases. Cross-runtime Prayfile parse parity also lives under `testdata/shared/manifest/` and is exercised by Rust, TypeScript, and Ruby CI suites.

Current packs exercised by all three runtimes:

- `parser/minimal-prayfile`
- `prayspec/minimal-package`
- `lockfile/minimal` and `lockfile/invalid-toml`
- `lockfile/span-matching`, `span-edited`, `span-missing-dest`, `span-removed`, `span-orphan`
- `resolver/path-package` and `resolver/constraint-mismatch`
- `render/compose-fragment`
