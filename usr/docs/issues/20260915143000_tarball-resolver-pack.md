# RFC 0100 tarball resolver pack

## Participants

Andrei Makarov

## Decisions

Add fixtures/resolver/tarball-package with a committed local .praypkg and Prayfile tarball: packages/sample-base-1.0.0.praypkg. Copy the fixture to a temp directory before resolve so .pray/cache is not written into fixtures. Compare name, version, tree_hash, and exports. Local tarball resolve works offline. HTTP or HTTPS tarball URLs fetch when offline is false. git: and oci: on a package declaration stay unsupported.

Move resolve_package_root out of resolve.rs so the tarball branch does not grow that file past its loc ratchet.

## Effects

Rust, Ruby, and TypeScript unpack the committed archive into .pray/cache/tarball keyed by the first 16 hex digits of the artifact sha256.

Confirming checks: cargo test -p pray-core --test conformance_resolve_distribution --offline (3 passed, including resolver_tarball_package_fixture). cargo test -p pray-core --offline (full crate suite passed). cargo clippy -p pray-core --all-targets -- -D warnings. cargo fmt -p pray-core -- --check. bundle exec rspec spec/pray/conformance_resolve_spec.rb (6 examples, 0 failures, including tarball-package). bundle exec rubocop lib/pray/resolve.rb lib/pray/resolve_tarball.rb (no offenses). npx tsc -p tsconfig.build.json, npx tsc -p tsconfig.test.json, and node --test dist/conformance-resolve-distribution.test.js (3 passed, including tarball-package). node_modules/.bin/biome check --write src/resolve/tarball.ts src/resolve/package-root.ts src/conformance-resolve-distribution.test.ts (Checked 3 files). make loc-check (149 warnings, 0 failures).

Resource and budget: unpack uses the existing 64 MiB archive ceiling. Local resolve reads the .praypkg from disk and does not use HTTP. Confirming check: the fixture uses offline true.

Trace and identification: tree_hash is a content digest of listed package files. Cache directory names are a hash of artifact bytes. No person identifiers.

Boundary and control: skipped. Local filesystem for the pack. HTTP tarball fetch is the existing bounded client.

Product surface: skipped.

Privacy: skipped.

Performance: skipped as unmeasured.

Observability: skipped.

Security: relative tarball paths join project_root. Remote tarball has no separate artifact_hash in Prayfile; lock pins tree_hash after resolve.

Contract: expected.json packages fields listed above. Messages are not compared. RFC 0100 remains Experimental.

Learned systems: skipped.

## Next

PyO3 bind of embed after RFC 0109 field freeze. Mix task when a Phoenix repo asks.

## Source

rfcs/0100-conformance.md
fixtures/resolver/tarball-package/
crates/pray-core/src/resolve_tarball.rs
usr/docs/issues/20260915140000_git-registry-cycle-resolver-packs.md
