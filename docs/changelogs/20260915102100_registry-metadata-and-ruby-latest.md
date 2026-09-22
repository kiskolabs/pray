## Decisions

Published registry metadata must not echo spec.upstream. The pin stays in the packaged prayspec. Catalog JSON stays hashes, yank, targets, exports, signer, and derived content annotations.

Ruby pray update --latest rewrites Prayfile constraints that do not admit the registry latest version, using the same operator family as Rust and TypeScript. --latest --dry-run prints that rewrite and does not write. --json and --major stay rejected.

## Effects

RFC 0114 now forbids an upstream field on published package versions. The registry schema already rejects extra version properties.

Ruby pray update --latest rewrites a pessimistic Prayfile pin across a major registry line, then locks the new version. Dry-run prints the planned constraint and leaves Prayfile and the lock unchanged.

## Next

Later pass 20260915103600: this work ships as 1.15.0. See usr/docs/issues/20260915103600_prepare-1-15-0-release.md.

## Source

RFC 0114
RFC 0060
schema/registry.schema.json
CHANGELOG.md 1.15.0
usr/docs/changelogs/20260915090000_align-upstream-publish-clients.md
usr/docs/issues/20260915103600_prepare-1-15-0-release.md
