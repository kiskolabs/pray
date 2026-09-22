# RFC 0100 fixture audit

## Participants

Andrei Makarov

## Decisions

Keep RFC 0100 Experimental. Wire existing parser and prayspec packs into Ruby and TypeScript as well as Rust. Add a lockfile parse pack with one valid document and one invalid TOML document. Do not add resolver, render, or verify packs in this pass.

## Effects

Audit: fixtures/README.md listed parser, prayspec, resolver, lockfile, render, verify, registry, and packages. Only parser/minimal-prayfile and prayspec/minimal-package existed. crates/pray-core/tests/conformance_fixtures.rs was the only consumer. Ruby and TypeScript already ran testdata/shared/manifest, not fixtures/. RFC 0100 Implementation notes said no resolver, render, or verify packs; that was still true and hid that Level 0 was Rust-only.

Resource and budget: skipped new runtime cost. Tests read small files. Confirming check: cargo test -p pray-core --test conformance_fixtures, bundle exec rspec spec/pray/conformance_fixtures_spec.rb, npm test filter on conformance-fixtures.

Trace and identification: fixture hashes are synthetic. No person identifiers.

Boundary and control: skipped. Parse-only.

Product surface: skipped.

Privacy: skipped.

Performance: skipped.

Observability: skipped.

Security: invalid TOML is rejected. Path-safe fixture directory under fixtures/. No package unpack.

Contract: expected.json fields are prayfile_version, package_names, target_names for parser; name, version, files for prayspec; prayfile_lock, spec, package_names, managed_span_ids for lockfile. Messages are not compared.

Learned systems: skipped.

EA-001. Severity high. Confidence high. Location fixtures/ consumed only by Rust. Kind observed. Why it matters: RFC 0100 claimed polyglot fixtures while Level 0 ran in one runtime. Smallest credible fix applied: same packs in Ruby and TypeScript tests. Deeper fix: resolver and render expected trees.

Missing coverage: resolver, render, verify. Futile coverage: none added; expected.json avoids exact TOML text and error strings.

## Next

Git and registry resolver packs. Polyglot dependency-cycle reject once Ruby and TypeScript refuse cycles the way Rust resolve does. PyO3 bind of embed after RFC 0109 field freeze. Mix task when a Phoenix repo asks.

Later pass 20260915144000: this work ships as 1.16.0. See usr/docs/issues/20260915144000_prepare-1-16-0-release.md.

## Source

rfcs/0100-conformance.md
fixtures/README.md
crates/pray-core/tests/conformance_fixtures.rs
rubygems/pray-cli/spec/pray/conformance_fixtures_spec.rb
npmjs/pray-cli/src/conformance-fixtures.test.ts
usr/docs/changelogs/20260915124500_conformance-lockfile-fixtures.md
usr/docs/issues/20260915133000_render-fixtures-and-lock-adapter-ports.md
usr/docs/issues/20260915134500_conformance-resolver-fixtures.md
usr/docs/issues/20260915144000_prepare-1-16-0-release.md
