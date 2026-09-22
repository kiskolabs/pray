# Publish cross-signer audit

## Decisions

Follow-up on 2026-09-18: add an explicit Rust pray publish --resign operation for local roots. Rebuild the artifact from the current package source before signing, because a self-consistent stored hash and legacy digest do not prove that stored exports match the current package. Keep ordinary cross-signer publishes unchanged. Treat human publisher labels as catalog claims on the package page and in RFC 0061. This changes the initial audit's open next step into implemented work; signature history remains a separate design question.

## Effects

Reviewed the diff from main to patch/publish-cross-signer-preservation. The changed local-root publish path preserves a version row when its stored artifact and signature validate. Focused Rust, TypeScript, and Ruby publish tests passed. A direct CLI reproduction confirmed both findings below. No production code changed during this audit.

The broad TypeScript test command passed 256 tests and failed one unrelated metadata test because the sandbox denied a localhost bind. The first Ruby spec attempt used system Ruby and could not find the locked Bundler version; rerunning through rbenv passed five examples. Cargo formatting, Clippy, diff whitespace, and the LOC gate passed. The LOC gate reported 163 warnings and no failures.

Follow-up implementation adds key upgrade, key rotation, forged stored artifact, and invalid option tests in publish_resign.rs. The local-root re-sign path rebuilds from source; the package page labels the human publisher name and distinguishes Ed25519 signatures from legacy content digests.

Follow-up validation: cargo test, cargo clippy --quiet, and cargo fmt --check passed. make loc-check reported 164 warnings and zero failures. The focused Ruby and TypeScript publish suites passed five examples and four tests respectively.

## Next

Signature history remains open. A later contract can define multiple version-level attestations and a trusted key-to-identity binding if co-signing is required.

If co-signing is required, bind each attestation to one package name, version, artifact hash, and tree hash. Keep package-level signer authorization separate from version attestations. A list stored solely at the distribution root does not preserve history against an operator who can rewrite that root; tamper-evident history requires an independent checkpoint or witness.

## Source

rfcs/0050-security-and-trust.md
rfcs/0061-publish-version-preservation.md
crates/pray-cli/src/publish.rs
crates/pray-cli/src/publish_integrity.rs
crates/pray-core/src/package_integrity.rs
npmjs/pray-cli/src/publish/integrity.ts
rubygems/pray-cli/lib/pray/publish.rb

## Findings

Severity high. Confidence high. Location local-root publish skip in crates/pray-cli/src/publish.rs and stored_publish_matches in crates/pray-cli/src/publish_integrity.rs. Kind observed. Why it matters: a valid legacy digest causes publish with --signing-key to skip the version. The row remains sha256 with no public key, so supplying a signing key does not upgrade the artifact's attestation. A second Ed25519 key likewise does not rotate an existing signature. Reproduction: publish the simple project without a key, then publish unchanged with --signing-key; the row remains sha256 and retains the first signer. Publishing once with key A and again with a different key B leaves the complete version JSON byte-identical. Smallest fix: provide an explicit re-sign or add-attestation operation for unchanged content and test both digest upgrade and key rotation.

Severity medium. Confidence high. Location RFC 0061 security considerations and the Ed25519 branch of verify_package_signature in crates/pray-core/src/package_integrity.rs. Kind observed. Why it matters: the signature authenticates artifact_hash and tree_hash, while signer and signer_fingerprint are mutable catalog claims. Reproduction: edit the signer field on a valid Ed25519 row, then republish unchanged; the edited label remains and signature verification succeeds. The RFC's claim that keeping signer and signature together makes the pair internally consistent overstates what the signature verifies. Smallest fix: state that the public key authenticates package hashes, while human signer labels need an independently trusted key-to-identity binding. Make any future co-signing record bind the intended signer identity to the signature.

Severity medium. Confidence high. Location new cross-signer tests in Rust, TypeScript, and Ruby. Kind observed missing coverage. The tests cover a second publisher with unchanged content and changed content, but not explicit --signing-key on an unchanged legacy or already signed row, nor catalog signer-label tampering. This is missing coverage, not futile coverage. Smallest fix: add one end-to-end key-upgrade case and one provenance-label case before changing the signing contract.

## Audit scope

Pipeline and boundary: publisher reads a local artifact and catalog row, validates stored bytes, then either preserves or writes. The root operator controls catalog metadata; an Ed25519 key controls the signed hash statement. No service or worker implementation changed in this branch. A distribution operator can omit or replace metadata; a client needs a trust anchor outside that operator for stronger provenance.

Scope limit: RFC 0061 specifies local roots. Rust publish_to_server still creates a new row on every run, and server merge identity includes signer and signature, so a cross-signer repeat through --server follows a different path. If remote preservation is desired, it needs its own contract and test.

Resource and budget: local no-op checks read each selected artifact and inspect its archive, so work grows with selected artifact bytes. No RSS, CPU, or disk benchmark was run; any cost claim remains inference. No new network request is introduced by the changed local-root path.

Trace and identification: signer label and optional fingerprint remain in registry metadata; the branch adds no field. No packet or log capture was run, so claims about outside observers remain inference. Privacy: those fields can identify a publisher, and retention remains under distribution-root control. Product surface: the RFC prose is the changed person-facing document; its structure is clear, but the identity wording above needs correction. No interactive, narrow-screen, loading, or error state changed. Observability and learned-systems modes do not apply to this local CLI change. Contract and security modes apply through the signing and registry metadata rules.
