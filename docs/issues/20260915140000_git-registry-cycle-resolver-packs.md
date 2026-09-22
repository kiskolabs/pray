# RFC 0100 git, registry, and cycle resolver packs

## Participants

Andrei Makarov

## Decisions

Add fixtures/resolver/git-distribution and fixtures/resolver/registry-distribution as committed local v1/packages trees with a .praypkg artifact. Prayfile uses git+file://distribution and source dist, distribution relative to the project root. Copy those fixtures to a temp directory before resolve so .pray/cache is not written into fixtures. Compare name, version, tree_hash, and exports. Add fixtures/resolver/dependency-cycle and port reject_dependency_cycles to Ruby and TypeScript. Join relative file:// git URLs and relative registry paths with project_root. Route a non-http registry source in Rust through resolve_local_registry_package_root.

## Effects

Rust, Ruby, and TypeScript refuse the two-package cycle. They resolve the git and registry packs offline false against the committed distribution.

Confirming checks: cargo test -p pray-core --test conformance_resolve --test conformance_resolve_distribution --test dependency_cycle (9 passed). bundle exec rspec spec/pray/conformance_resolve_spec.rb spec/pray/conformance_fixtures_spec.rb (15 examples, 0 failures). npx tsc -p tsconfig.test.json and node --test dist/conformance-resolve.test.js dist/conformance-resolve-distribution.test.js (5 passed). cargo clippy -p pray-core --all-targets -- -D warnings. make loc-check (0 failures).

Resource and budget: git pack skips clone when the path is a distribution root without .git. Registry pack unpacks one .praypkg into project .pray/cache. No HTTP. Confirming check: fixtures use relative local URLs.

Trace and identification: tree_hash is a content digest of listed package files. Cache directory names hash the source URL. No person identifiers.

Boundary and control: skipped. Local filesystem only.

Product surface: skipped.

Privacy: skipped.

Performance: skipped as unmeasured.

Observability: skipped.

Security: relative file:// and registry paths stay under the copied project root.

Contract: expected.json must_reject for dependency-cycle. Git and registry expected packages fields listed above. Messages are not compared. RFC 0100 remains Experimental.

Learned systems: skipped.

## Next

PyO3 bind of embed after RFC 0109 field freeze. Mix task when a Phoenix repo asks.

Later pass 20260915144000: this work ships as 1.16.0. See usr/docs/issues/20260915144000_prepare-1-16-0-release.md.

## Source

rfcs/0100-conformance.md
fixtures/resolver/git-distribution/
fixtures/resolver/registry-distribution/
fixtures/resolver/dependency-cycle/
crates/pray-core/tests/conformance_resolve.rs
crates/pray-core/tests/conformance_resolve_distribution.rs
usr/docs/issues/20260915134500_conformance-resolver-fixtures.md
usr/docs/issues/20260915144000_prepare-1-16-0-release.md
