# RFC 0050: Security and trust

- Feature Name: security-and-trust
- Type: Standards Track
- Status: Stable
- Describes: 1.8.1
- Created: 2026-08-17
- Author: Andrei Makarov
- Relates: RFC 0020, RFC 0060, RFC 0104

## Summary

Inference input is a supply chain. This RFC describes the security model already claimed by the specification and exercised by the reference CLI: static packages, hash verification, optional ed25519 signatures, yank, trust policy, and annotations as claims.

## Motivation

A package that runs code, or metadata treated as verified bytes, would teach models from unverified claims.

## Guide-level explanation

Implementations MUST NOT execute package code. Remote artifacts MUST be checked against lock hashes before render.

Publish with `--signing-key` or `PRAY_SIGNING_KEY`. Install verifies `signer_public_key` when the lock or registry metadata carries an `ed25519:` signature. Signatures are OPTIONAL in v1.

`pray trust set-require-signed-packages` refuses unsigned remotes under a source prefix (changelog 1.8.0). `pray yank` flips metadata; bytes stay immutable.

`pray confess` sends signed acceptance or rejection. Consumers treat confess notes as claims until verified against hashes or rendered-byte digests.

## Reference-level explanation

This section is the product contract for this concern. Where it disagrees with Implementation notes, Implementation notes record what the reference CLI does today. A follow-on RFC records the gap.

### 58. Security model

V1 baseline:

- never execute package code
- reject path traversal
- reject absolute archive paths
- reject symlinks in packages
- verify artifact hashes
- verify tree hashes
- support optional signatures
- support offline mode
- support private sources
- support vendoring
- avoid secrets in lockfile
- avoid environment capture
- avoid timestamps and machine-specific data
- make generated files visibly generated
- make updates explicit

Agent packages affect automated behavior. They must be treated as supply-chain inputs.

---

### 59. Signatures

Optional v1 support for registry package versions.

#### Preferred: ed25519 package signature

When a publisher signing key is available, the reference CLI prefers an ed25519 signature over the package identity hashes:

- payload: `artifact_hash` UTF-8 bytes, a single `0x00` byte, then `tree_hash` UTF-8 bytes
- `signature`: `ed25519:` followed by standard Base64 of the 64-byte ed25519 signature
- `signer_public_key`: OpenSSH `ssh-ed25519` public key text for the verifying key
- `signer` / `signer_fingerprint`: human label and optional SSH fingerprint identity (unchanged)

Install and sync verify ed25519 signatures with `signer_public_key`. SSH transport auth does not substitute for this check.

Reference CLI key sources for publish:

- `--signing-key PATH` to a 32-byte raw ed25519 seed file
- or environment `PRAY_SIGNING_KEY` with the same path shape

#### Legacy: content digest

When no signing key is available, publish may still record a legacy content digest for compatibility:

- digest input: artifact bytes, `0x00`, tree hash UTF-8, `0x00`, signing identity UTF-8
- `signature`: `sha256:` prefixed hex digest of that input
- signing identity prefers `signer_fingerprint` when present, otherwise `signer`
- `signer_public_key` is omitted

Legacy digests bind published bytes to a claimed identity string. They are not public-key signatures. New publishers should prefer ed25519.

#### Other reserved forms

The data model may later add registry-key signatures over metadata or lockfile signature identity fields. Example lockfile placeholders:

```
signature = "ed25519:..."
signer = "sample-agent-packages-2026"
```

Signature support remains optional in v1, but remote integrity requires artifact and tree hashes.

---

### 60. Yanked packages

Registries may mark versions as yanked. The reference CLI flips the metadata flag with `pray yank <package> <version> --root PATH` (and `--undo` to clear). Artifact bytes stay immutable.

Rules:

- new resolution should not select yanked versions by default
- existing lockfile may continue using yanked version with warning
- `pray install --strict` fails when a selected version is yanked
- update should move away from yanked version when possible

---

### 71. Private context

Private context should be handled through: private registry, local path packages, ignored local files, optional local files, vendor mode

Do not put secrets, credentials, personal facts, private business data, or private customer data into public packages.

Do not put secrets into Prayfile.lock.

---

## Implementation notes

Remote install calls `require_remote_integrity_fields` in `package_integrity.rs` before unpack. Empty `artifact_hash` or `tree_hash` is an integrity error. Callers: HTTP and local paths in `registry.rs`, `registry_ssh.rs`, and `validate_and_unpack_registry_package` in `registry_cache.rs`. Test: `require_remote_integrity_fields_fails_when_hashes_missing` in `crates/pray-core/tests/package_signature.rs`.

Unpack compares SHA-256 of artifact bytes to metadata, unpacks, then compares `tree_hash`, then `verify_package_signature`. Missing `signature` returns Ok. `ed25519:` uses hashes and `signer_public_key`. Other values compare `artifact_content_digest`. `registry_artifact_signature` in `registry.rs` aliases that digest. `package_signature_for_publish` writes ed25519 when a signing key is present, else the digest.

`enforce_require_signed_packages` in `client_trust/package_sign.rs` requires a non-empty `signature` field when the source prefix matches. It does not require the `ed25519:` prefix.

Vendor and cache reuse in `registry_cache.rs` require `tree_hash` and verify ed25519 without artifact bytes. They skip the legacy digest. First unpack still verified hashes.

The 1.8.x CLI has `login`, `token`, passkey/SSH flags, and publish tokens (`PRAY_PUBLISH_TOKEN`). Completeness of web enrollment versus CLI login is reserved as RFC 0105.

## Security considerations

Confess notes and emails used at login MAY leave the machine. Implementations SHOULD keep secrets out of argv; prefer files and secret stores.

## Unresolved questions

Whether require-signed-packages is a lockfile field, a global config, or both.

Whether require-signed should accept only `ed25519:` signatures.

Whether vendor and cache reuse should re-check the legacy digest (needs artifact bytes).

How federation peers inherit trust (RFC 0104) without expanding the client threat model.
