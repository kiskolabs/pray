# Accept legacy published_at catalog values

## Decisions

Registry and federation readers accept three migration forms for published_at: a decimal numeric string, an RFC 3339 string, and JSON null treated as absent. New catalogs still write a JSON integer or omit the field. The JSON schema still rejects strings and null.

## Effects

Catalogs written by pray 1.13 as a numeric string already loaded. Catalogs written by the Ruby and TypeScript CLIs as RFC 3339, and rows that stored null under the 1.13 Option default, failed every command that deserialized RegistryPackageMetadata. The Rust, Ruby, and TypeScript readers now share those migration forms. The next metadata write normalizes to the canonical integer or omits an unknown value.

RFC 0061 documents the null MAY. The person-facing surface is the terminal parse error that those commands printed before the catalog could be used.

A CLI yank against a one-package static root succeeded for null, RFC 3339, and a numeric string.

Validation: cargo test -p pray-core -- --test-threads=1 passed the crate, including registry_timestamp 2 tests and schema_validation 10 tests. cargo test -p pray-transport --test package_version passed 2 tests. cargo clippy --workspace --all-targets --all-features -- -D warnings passed. cargo fmt --all --check passed. make loc-check reported 164 warnings and 0 failures. bundle exec rspec spec/pray/registry_spec.rb passed 15 examples. npx tsc -p tsconfig.test.json and node --test dist/registry/timestamp.test.js passed 2 tests. biome check on the changed TypeScript files passed. bundle exec rubocop on the changed Ruby files found no offenses. git diff --check passed. cargo test -p pray-cli --test cli_upgrade failed upgrade_notice_uses_changelog_link because pray list in this workspace requires interactive consent for an untrusted git source; that path does not deserialize published_at.

## Next

None.

## Source

usr/docs/issues/20260922152000_published-at-legacy-reader.md
rfcs/0061-publish-version-preservation.md
crates/pray-core/src/registry_timestamp.rs
rubygems/pray-cli/lib/pray/registry.rb
npmjs/pray-cli/src/registry/index.ts
