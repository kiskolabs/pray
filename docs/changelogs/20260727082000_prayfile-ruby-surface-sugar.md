# Prayfile Ruby surface sugar and library parity

## Participants

- Andrei Makarov

## Decisions

- Keep canonical Prayfile form as newline-separated `do` / `end` with space-separated symbol assignments.
- Accept optional Gemfile-like sugar: brace blocks, top-level semicolons, and call parentheses on keywords and symbol assignments.
- Keep interpolation, constants, variables, and method chaining out of scope.
- Normalize sugar before the existing statement dispatcher so Rust, TypeScript, and Ruby share one surface contract.

## Effects

- Rust `statement_surface` expands sugar into canonical statements; parser and unit tests cover brace, semicolon, and paren forms.
- TypeScript `statement-surface` mirrors the same expansion with tests.
- Brace expansion is limited to keyword call forms so Package::Specification maps like `spec.exports = { … }` stay intact.
- Ruby gem gains symbols, substitute, surface expansion, and `pray` package aliases; compose/tree destination DSL was still a later parity gap at that point.

## Next

- Done in usr/docs/changelogs/20260727083000_ruby-destination-dsl-shared-corpus.md: Ruby destination DSL port and shared fixture corpus.

## Source

- SPEC.md pray symbols surface sugar note
- crates/pray-core/src/statement_surface.rs
- npmjs/pray-cli/src/literal/statement-surface.ts
- rubygems/pray-cli/lib/pray/statement_surface.rb
- rubygems/pray-cli/lib/pray/substitute.rb
