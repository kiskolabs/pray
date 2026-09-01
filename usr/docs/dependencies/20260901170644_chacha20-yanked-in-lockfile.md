# Yanked chacha20 0.10.1 in Cargo.lock

## Dependency

chacha20 was locked at 0.10.1 through rand 0.10.2. The workspace does not declare chacha20 directly.

## Symptom

cargo publish --locked for pray-core warned that package chacha20 v0.10.1 in Cargo.lock is yanked on crates.io. make release-all was interrupted during pray-core verify before upload.

## Evidence

crates.io lists chacha20 0.10.1 as yanked by newpavlov on 2026-08-27, with a null yank message. chacha20 0.10.0 is also yanked. chacha20 0.10.2 was published the same day and is not yanked. The 0.10.2 release notes fix use of an SSE4.1 intrinsic in the SSE2 backend of RNG and legacy 64-bit counter variants.

cargo update -p chacha20 --precise 0.10.2 moved Cargo.lock to 0.10.2. cargo deny check advisories completed with advisories ok. cargo publish -p pray-core --dry-run --locked --allow-dirty packaged pray-core 1.9.2 and aborted the upload with no yanked-crate warning.

## Suggested fix

Keep chacha20 at 0.10.2 or later within the existing rand 0.10 constraint. No source patch or direct chacha20 dependency is needed.

## Next

Commit the lockfile before cargo publish --locked. v1.9.2 currently points at d6cb296, which still has chacha20 0.10.1. Move that tag onto the lockfile commit if the GitHub tag should match the crate that gets published.

## Source

crates.io crate chacha20 0.10.1 yank action and 0.10.2 version metadata.
https://github.com/RustCrypto/stream-ciphers/commit/6b236b758a0279f64d777797514813b2cb572c8b
https://github.com/RustCrypto/stream-ciphers/pull/580
Cargo.lock rand 0.10.2.
usr/docs/issues/20260901170644_prepare-1-9-2-release.md
