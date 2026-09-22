# Render fixtures and lock adapter ports

## Participants

Andrei Makarov

## Decisions

Commit a shared compose dest under fixtures/render/compose-fragment so Rust, Ruby, and TypeScript must emit the same bytes. Add lockfile span packs that exercise inspect_locked_destinations finding kinds without resolve. Export that function from the Ruby and TypeScript libraries using the RFC 0106 name. Leave PyO3 unbound until RFC 0109 freezes fields. Leave Mix unbuilt until a Phoenix repo asks.

## Effects

fixtures/render/compose-fragment holds Prayfile, a path package, and expected/INSTRUCTIONS.md. crates/pray-core/tests/conformance_render.rs, rubygems/pray-cli/spec/pray/conformance_fixtures_spec.rb, and npmjs/pray-cli/src/conformance-fixtures.test.ts compare render output to that dest.

fixtures/lockfile/span-matching, span-edited, span-missing-dest, span-removed, and span-orphan carry Prayfile.lock plus dest (or no dest) and expected.json finding_kinds. The same three suites call inspect_locked_destinations and compare kinds, not messages.

Ruby Pray.inspect_locked_destinations lives in lib/pray/verify_locked_dest.rb. TypeScript inspectLockedDestinations lives in src/verify/locked-dest.ts and is exported from src/index.ts.

Confirming checks: cargo test -p pray-core --test conformance_render (6 passed), bundle exec rspec spec/pray/conformance_fixtures_spec.rb (10 examples, 0 failures), node --test dist/conformance-fixtures.test.js (10 passed).

Resource and budget: dest reads still use the 32 MiB destination limit. Render of the fixture is one fragment. No network. Confirming check: destination size path unchanged; new tests do not raise the limit.

Trace and identification: fixture hashes are synthetic. No person identifiers.

Boundary and control: skipped. File reads under the fixture directory.

Product surface: skipped. Library names only.

Privacy: skipped.

Performance: skipped as unmeasured. Inference: span inspect stays cheaper than verify_project because it does not resolve or re-render.

Observability: skipped.

Security: dest paths still come from the lock. Span packs use repository-relative INSTRUCTIONS.md.

Contract: expected dest bytes for render. expected.json finding_kinds for span inspect. RFC 0100 remains Experimental until a resolver pack exists.

Learned systems: skipped.

Python and Elixir: still no in-repo caller. A fourth resolver stays rejected. Mix and pytest should spawn pray or bind inspect_locked_destinations later.

EA-002. Severity medium. Confidence high. Location fixtures/render was empty while RFC 0100 listed a render pack. Kind observed. Why it matters: compose dest bytes could drift across the three CLIs. Smallest credible fix applied: one compose-fragment dest committed and compared. Deeper fix: resolver expected lock still missing.

Missing coverage: resolver packs. Futile coverage: none added; span tests compare finding kinds, not diagnostic strings.

## Next

Git and registry resolver packs. Polyglot dependency-cycle reject once Ruby and TypeScript refuse cycles the way Rust resolve does. PyO3 bind of embed after RFC 0109 field freeze. Mix task when a Phoenix repo asks.

Later pass 20260915144000: this work ships as 1.16.0. See usr/docs/issues/20260915144000_prepare-1-16-0-release.md.

## Source

rfcs/0100-conformance.md
rfcs/0106-host-language-lock-adapter.md
fixtures/render/compose-fragment/expected/INSTRUCTIONS.md
fixtures/lockfile/span-matching/
rubygems/pray-cli/lib/pray/verify_locked_dest.rb
npmjs/pray-cli/src/verify/locked-dest.ts
usr/docs/issues/20260915124500_conformance-lockfile-fixtures.md
usr/docs/issues/20260915130000_host-language-lock-adapter.md
usr/docs/changelogs/20260915133000_render-fixtures-and-lock-adapter-ports.md
usr/docs/issues/20260915134500_conformance-resolver-fixtures.md
usr/docs/issues/20260915144000_prepare-1-16-0-release.md
