# Publish re-signing and identity labels

## Decisions

Use an explicit --resign flag on the Rust CLI for local distribution roots. Ordinary publish keeps a valid row even when another publisher runs it. Re-signing requires an Ed25519 key and rebuilds from local package source, so a false stored artifact and a matching legacy digest cannot receive a new trusted signature. RFC 0061 describes the optional command contract. The previous signature is replaced; retaining signature history needs a separate contract.

## Effects

An unchanged version can move from a legacy content digest to Ed25519 or rotate to another key while keeping its yank state and first-publish time. --resign rejects a missing key, a server destination, and --dry-run before changing a local root. The distribution package page now says Publisher label for the unauthenticated human name, Package signature for Ed25519, and Content digest for a legacy SHA-256 field.

The regression test first failed because --resign was unknown. A second test then failed when the first implementation signed a self-consistent but false stored artifact. The implementation now rebuilds from source, and both paths pass.

The package page is the person-facing surface. The label change clarifies what the displayed fields mean without changing hierarchy or navigation. The smoke flow covers the package page and a completed install; no new interactive, empty, loading, error, or narrow-width state was added. A human visual review remains open.

Validation: cargo test passed the Rust workspace, including four publish_resign tests and the distribution-page smoke flow. cargo clippy --quiet and cargo fmt --check passed. make loc-check reported 164 warnings and zero failures. rbenv exec bundle exec rspec spec/pray/publish_spec.rb passed five examples. node --test dist/publish/index.test.js passed four tests. The earlier broad npm run passed 256 tests and hit one sandbox localhost-bind error in a registry metadata test; it was not rerun because no TypeScript code changed in this follow-up.

## Next

Decide whether version-level co-signing and independently witnessed history are needed. If so, specify signer authorization and freshness separately from the artifact signature.

## Source

rfcs/0061-publish-version-preservation.md
crates/pray-cli/src/publish.rs
crates/pray-cli/src/publish_version.rs
crates/pray-cli/src/server_html.rs
crates/pray-cli/tests/publish_resign.rs
crates/pray-cli/tests/support/distribution_point_smoke.rb
usr/docs/issues/20260918160709_publish-cross-signer-audit.md
