# Conformance lockfile fixtures

## Participants

Andrei Makarov

## Decisions

Exercise fixtures/parser, fixtures/prayspec, and fixtures/lockfile in Rust, Ruby, and TypeScript. Add a minimal lockfile and an invalid TOML lockfile. Leave RFC 0100 Experimental.

## Effects

crates/pray-core/tests/conformance_fixtures.rs now parses through pray_core::embed. Ruby spec/pray/conformance_fixtures_spec.rb and TypeScript src/conformance-fixtures.test.ts load the same expected.json files.

## Next

RFC 0106 lock adapter. Resolver and render packs remain open.

## Source

rfcs/0100-conformance.md
usr/docs/issues/20260915124500_conformance-lockfile-fixtures.md
fixtures/lockfile/minimal/Prayfile.lock
