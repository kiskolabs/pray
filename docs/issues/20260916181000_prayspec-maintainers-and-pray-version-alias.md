# RFC 0011 maintainers parse and pray_version alias

## Participants

Andrei Makarov

## Decisions

RFC 0011 grammar listed spec.maintainers. Parsers rejected it. The parser now stores a string array and sorts it on canonicalize, same as authors.

RFC 0011's worked example used spec.pray_version. The grammar and canonical JSON already used prayfile_version. The example now writes prayfile_version. Parsers also accept pray_version as an alias that stores prayfile_version. Render emits prayfile_version.

## Effects

A prayspec with spec.maintainers and spec.pray_version parses in Rust, Ruby, and TypeScript. Canonical JSON may include maintainers when non-empty. schema/package.schema.json allows that field.

Confirming checks: cargo test --offline -p pray-core --test parser parses_package_spec_maintainers_and_pray_version_alias (1 passed). cargo test --offline -p pray-core --lib package_spec_render (2 passed). cargo test --offline -p pray-core --test schema_validation parsed_example_package (1 passed). cargo clippy --offline -p pray-core --all-targets -- -D warnings (exit 0). cargo fmt --all -- --check (exit 0). bundle exec rspec spec/pray/parser_spec.rb spec/pray/package_spec_render_spec.rb (37 examples, 0 failures). npm test in npmjs/pray-cli (203 pass, 0 fail). npx biome check on the changed package-spec TypeScript files (no fixes). make loc-check (152 warnings, 0 failures).

## Next

None for this contract. Enforce prayfile_version as a CLI constraint later if a product RFC asks for it. Today the field is stored only.

## Source

rfcs/0011-prayspec-and-package.md
crates/pray-core/src/package_spec.rs
crates/pray-core/tests/parser.rs
schema/package.schema.json
CHANGELOG.md 1.17.0
usr/docs/changelogs/20260916181000_prayspec-maintainers-and-pray-version-alias.md
