## Participants

Andrei Makarov

## Decisions

Fold the catalog `derived_metadata` publish fix into the existing dated 1.22.0 section on main. No v1.22.0 tag exists yet, so this cut stays 1.22.0 dated 2026-10-22 instead of opening 1.23.0. Preparation stays separate from tagging and publishing.

## Effects

Root, npm, and Ruby changelogs list the derived metadata bullet under 1.22.0. Unreleased is empty again. Versions remain 1.22.0 across Cargo, npm, and Ruby.

Quality fixes during this preparation pass:

- Biome format and import order for TypeScript derived-metadata and publish modules.
- Split the historical derived_metadata publish test out of index.test.ts to stay under the 300-line file ceiling.
- RuboCop AbcSize split in derived_metadata.rb.
- UTF-8 File.read in the upstream_refresh catalog metadata assertion after summaries include a non-ASCII join.

Validation completed with these commands and observed results:

- ./scripts/release/changelog_test.sh: passed.
- ./scripts/release/versions_test.sh: passed.
- ./scripts/release/crates_status_test.sh: passed.
- ./scripts/release/github.sh --notes-only --version 1.22.0: printed the dated 1.22.0 heading including the derived_metadata bullet.
- cargo fmt --all --check: passed.
- make loc-check: 173 warnings at or above 150 lines and 0 unratcheted failures above 300 lines.
- git diff --check: passed.
- cd rubygems/pray-cli && bundle exec rspec spec/pray/publish_spec.rb spec/pray/upstream_refresh_spec.rb:75: 8 examples, 0 failures.
- cd rubygems/pray-cli && bundle exec rubocop lib/pray/derived_metadata.rb: no offenses.
- cd npmjs/pray-cli && npm run lint: checked 248 files; no circular dependency across 169 analyzed files.
- cd npmjs/pray-cli && npm test: 280 tests passed.
- ./scripts/release/all.sh --dry-run --skip-distribution: crates dry-run finished with expected unpublished-dependency fallbacks for transport and cli; npm packed pray-cli 1.22.0; built pray-cli-1.22.0.gem with no push; orchestration finished dry-run.

## Next

Tag v1.22.0, publish crates.io then npm and RubyGems, create the GitHub Release, and bump Homebrew.

## Source

docs/issues/20261022091000_prepare-1-22-0-release.md
docs/issues/20261022065311_catalog-derived-metadata-publish-loss.md
docs/dependencies/20261022065311_pray-ruby-npm-catalog-derived-metadata-loss.md
docs/changelogs/20261022065311_preserve-catalog-derived-metadata.md
