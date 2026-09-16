# RFC 0061: Publish version preservation

- Feature Name: publish-version-preservation
- Type: Standards Track
- Status: Proposed
- Created: 2026-09-14
- Author: Andrei Makarov
- Stakeholders: distribution maintainers
- Feedback until: 2026-09-28
- Relates: RFC 0050, RFC 0060, RFC 0104

## Summary

Publishing an unchanged package version preserves its stored artifact and version metadata. The `published_at` field uses one validated integer representation, records when the version's current package content was first written, and does not create package identity conflicts.

## Motivation

`pray publish --root` currently replaces the matching version row on every run. The replacement assigns the current time to `published_at` and clears `yanked`, even when the package tree and artifact are unchanged. A no-op release then changes catalog history and can undo an explicit yank.

The server merge path also compares `published_at` as identity. Two otherwise identical rows conflict when publishers assign their clocks at different times.

## Guide-level explanation

Publishing the same project twice leaves the distribution tree unchanged:

```sh
pray publish --root ./prayers
git add ./prayers
git commit -m "publish packages"
pray publish --root ./prayers
git diff --exit-code -- ./prayers
```

The final command exits successfully. Each version keeps its first `published_at` value. A prior `pray yank` remains in force until `pray yank --undo` clears it.

Registry JSON represents the time as whole UTC seconds since the Unix epoch:

```json
{
  "published_at": 1789344000
}
```

Changing package files or its prayspec causes publish to build the artifact again and replace the matching version row. The replacement receives a new `published_at` value.

## Reference-level explanation

For a local distribution root, a publisher MUST look for a row with the package version before building an artifact. The row is current when all of these checks pass:

- its `artifact` equals the expected relative artifact path;
- its `tree_hash` equals the resolved package tree hash;
- its stored artifact exists and hashes to `artifact_hash`;
- its archived prayspec filename and bytes equal the current package prayspec;
- its signer, signer fingerprint, public key, and signature equal the publish input.

An implementation with required protocol descriptors MAY require those descriptors to exist before treating the row as current. When the row is current, the publisher MUST preserve the artifact and complete version row.

When the row is absent or not current, the publisher builds and writes the artifact and version row. If a row for that version already exists, publish MUST preserve its `yanked` value. Publish MUST preserve `published_at` when the prior artifact passes its hash check and its package tree and prayspec match the current package. A changed tree or prayspec receives the current publish time. Artifact encoding, signer, or signature changes do not change the first-publish time when package content is unchanged.

Registry merge identity MUST exclude `published_at`. A merge of rows that differ only in `published_at` keeps the receiving registry's value.

`published_at` is optional. When present, it MUST be a JSON integer containing whole UTC seconds since `1970-01-01T00:00:00Z`. Its inclusive range is 0 through 253402300799, ending at `9999-12-31T23:59:59Z`. Producers MUST omit an unknown value. They MUST NOT emit a string, a number with a non-zero fractional part, a negative number, an out-of-range integer, or `null`. Registry and federation package-version payloads use this same representation.

Readers MAY accept numeric strings or RFC 3339 strings written by earlier reference clients for migration. They MUST truncate any legacy fractional second toward the preceding whole second and normalize the value to the canonical integer on the next metadata write. Other invalid representations fail validation.

The package signature contract in RFC 0050 remains unchanged. `published_at` is catalog metadata and is not part of the signed payload.

## Implementation notes

The reference implementation covers local preservation and legacy timestamp normalization in the Rust, Ruby, and TypeScript publish tests. Rust accepts legacy numeric strings. Ruby and TypeScript also accept prior RFC 3339 output. The registry schema test rejects strings, numbers with non-zero fractional parts, negatives, out-of-range integers, and null. The Rust registry identity test covers timestamp-only merge input.

## Security considerations

A publisher verifies stored artifact bytes against `artifact_hash` before skipping an unchanged version. It compares the stored artifact path with the expected relative path before reading, so catalog metadata cannot select another filesystem path for this check.

Yank remains an explicit metadata operation. Publish does not clear a yank.

## Drawbacks

An unchanged publish does not refresh derived annotations or replace a valid artifact with a newly encoded copy. Those operations need an explicit contract if distribution operators require them. Existing catalogs receive a one-time representation change when a reference client migrates a legacy timestamp.

## Rationale and alternatives

Rebuilding every package and copying only `published_at` still performs unnecessary archive and catalog writes. It also leaves no-op behavior dependent on whether an archive encoder emits deterministic bytes.

Comparing only `tree_hash` could retain a missing or corrupted artifact. Verifying the expected stored artifact preserves repair behavior.

RFC 0050 already treats artifact bytes as immutable and yank as metadata-only. Preserving a verified row follows those existing rules.

## Prior art

RFC 0050 separates immutable artifact bytes from mutable yank metadata. This RFC applies that separation to repeated publish operations.

## Unresolved questions

Whether a later signature contract should sign catalog timestamps remains open. This RFC does not change the signed payload.
