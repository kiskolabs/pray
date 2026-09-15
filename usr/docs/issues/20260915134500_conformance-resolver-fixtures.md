# RFC 0100 resolver fixture pack

## Participants

Andrei Makarov

## Decisions

Add fixtures/resolver/path-package so Rust, Ruby, and TypeScript resolve the same path package and produce the same lock slice: name, version, path, tree_hash, artifact, exports. Add fixtures/resolver/constraint-mismatch as a reject pack. Compare lock slices in expected.json, not full Prayfile.lock TOML. Do not add a dependency-cycle pack in this pass because Ruby and TypeScript resolve_project still accept two-package cycles that Rust reject_dependency_cycles refuses.

## Effects

crates/pray-core/tests/conformance_resolve.rs, rubygems/pray-cli/spec/pray/conformance_fixtures_spec.rb, and npmjs/pray-cli/src/conformance-resolve.test.ts call resolve_project offline then build_lockfile with no rendered dests.

Confirming checks: cargo test -p pray-core --test conformance_resolve (2 passed), bundle exec rspec spec/pray/conformance_fixtures_spec.rb (12 examples, 0 failures), node --test dist/conformance-resolve.test.js after tsc.

Resource and budget: path resolve reads the package files listed in spec.files. No network. Confirming check: offline true; constraint-mismatch fails before hashing a second package.

Trace and identification: tree_hash is a content digest of listed package files. No person identifiers.

Boundary and control: skipped. Local path packages only.

Product surface: skipped.

Privacy: skipped.

Performance: skipped as unmeasured.

Observability: skipped.

Security: path stays repository-relative under packages/base.

Contract: expected.json packages fields listed above. must_reject for constraint-mismatch. Messages are not compared. RFC 0100 remains Experimental until git and registry packs exist.

Learned systems: skipped.

EA-003. Severity medium. Confidence high. Location fixtures/resolver was empty while RFC 0100 listed a resolver pack. Kind observed. Why it matters: path resolve and lock pins could drift across CLIs. Smallest credible fix applied: path-package lock slice plus constraint-mismatch reject. Deeper fix: git and registry expected trees; cycle reject in Ruby and TypeScript.

Missing coverage: git, registry, tarball, dependency cycle as a polyglot reject. Futile coverage: none added; full lock TOML is not compared.

## Next

PyO3 bind of embed after RFC 0109 field freeze. Mix task when a Phoenix repo asks. Tarball resolver pack still open.

## Source

rfcs/0100-conformance.md
fixtures/resolver/path-package/
fixtures/resolver/constraint-mismatch/
crates/pray-core/tests/conformance_resolve.rs
usr/docs/changelogs/20260915134500_conformance-resolver-fixtures.md
usr/docs/issues/20260915133000_render-fixtures-and-lock-adapter-ports.md
