# RFC 0100 resolver fixtures

## Participants

Andrei Makarov

## Decisions

Ship a path-package lock slice and a constraint-mismatch reject pack. Leave git, registry, and cycle packs for later.

## Effects

Rust, Ruby, and TypeScript resolve fixtures/resolver/path-package to the same name, version, path, tree_hash, artifact, and exports. They reject fixtures/resolver/constraint-mismatch.

## Next

Git and registry packs. Cycle reject in Ruby and TypeScript.

Later pass 20260915144000: this work ships as 1.16.0. See usr/docs/issues/20260915144000_prepare-1-16-0-release.md.

## Source

usr/docs/issues/20260915134500_conformance-resolver-fixtures.md
rfcs/0100-conformance.md
fixtures/resolver/path-package/expected.json
crates/pray-core/tests/conformance_resolve.rs
CHANGELOG.md 1.16.0
usr/docs/issues/20260915144000_prepare-1-16-0-release.md
