# Repository layouts keep sources under prayers/

## Participants

Andrei Makarov

## Decisions

Keep prayer sources under prayers/. Consumer trees use prayers/<name>/. A publisher keeps those sources beside prayers/v1/. A catalog of many prayers uses the same tree.

prayers/v1/packages/ is catalog metadata. Root packages/, dist/, and shared/ are not pray source conventions.

A later pass on 2026-09-20 dropped the catalog-only packages/ source shape and moved this repository's prayer-publisher tree under prayers/.

## Effects

docs/repository-layouts.md. RFC 0117 guide-level. RFC 0002 naming. pray help prayer and pray help repo. This repository and examples use prayers/ for path sources. scripts/release/distribution.sh publishes path-owned packages from the project Prayfile.

Observed:

cargo fmt --all -- --check (exit 0)

cargo test --offline -p pray-cli --test init_hybrid --test cli_ux (12 passed)

cargo test --offline -p pray-core --test schema_validation (11 passed)

bundle exec rspec spec/pray/cli_help_spec.rb from rubygems/pray-cli (9 examples, 0 failures)

npx tsc -p tsconfig.test.json and node --test dist/cli/help.test.js from npmjs/pray-cli (7 passed)

## Next

Ships with the next CLI release. Downstream product repos can keep a one-line pointer.

## Source

usr/docs/issues/20260920112300_repository-layouts.md
rfcs/0117-local-prayers.md
docs/repository-layouts.md
