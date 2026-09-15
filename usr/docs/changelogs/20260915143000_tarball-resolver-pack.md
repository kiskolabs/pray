# RFC 0100 tarball resolver pack

## Participants

Andrei Makarov

## Decisions

Ship a tarball-package lock slice. Resolve tarball: to a local .praypkg in Rust, Ruby, and TypeScript. Keep that path working offline.

## Effects

All three libraries unpack sample/base 1.0.0 from packages/sample-base-1.0.0.praypkg and pin the same tree_hash.

## Next

PyO3 after RFC 0109 field freeze. Mix when a Phoenix repo asks.

Later pass 20260915144000: this work ships as 1.16.0. See usr/docs/issues/20260915144000_prepare-1-16-0-release.md.

## Source

usr/docs/issues/20260915143000_tarball-resolver-pack.md
rfcs/0100-conformance.md
fixtures/resolver/tarball-package/expected.json
crates/pray-core/src/resolve_tarball.rs
CHANGELOG.md 1.16.0
usr/docs/issues/20260915144000_prepare-1-16-0-release.md
