# Path-fork first material and overlay files

## Participants

Andrei Makarov

## Decisions

A path package with spec.upstream is a writable fork of a catalog package, not a git clone overlay on the distribution point. An identity-only prayspec has empty spec.files. pray install and pray update copy the resolved upstream content files into that path. Extra listed files in the same tree stay through later update merge. pray outdated lists changed, local-only, and missing content paths against the locked upstream. Locked and frozen install do not write the path tree.

A later pass stopped listing the prayspec in spec.files.

## Effects

RFC 0116 claimed. Rust, Ruby, and TypeScript install copy an empty fork. Update keeps overlay extras. Outdated prints fork drift lines.

cargo test -p pray-cli --test package_upstream --test cli_ux: 16 passed.
cargo clippy -p pray-core -p pray-cli --all-targets -- -D warnings: finished.
cargo test -p pray-core --lib package_upstream: 7 passed.
make loc-check: 0 failures.
bundle exec rspec spec/pray/upstream_refresh_spec.rb spec/pray/upstream_spec.rb spec/pray/cli_help_spec.rb: 24 examples, 0 failures.
npmjs biome check on the touched TypeScript files: clean.
node --test dist/resolve/upstream.test.js dist/cli/help.test.js dist/package-upstream.integration.test.js: 21 passed.

## Next

Ships as 1.17.0. See usr/docs/issues/20260916223500_prepare-1-17-0-release.md. pray plan would-copy list stays open. Unlisted extras are not offered for spec.files adoption.

## Source

rfcs/0116-path-fork-first-material.md
rfcs/0114-package-upstream.md
