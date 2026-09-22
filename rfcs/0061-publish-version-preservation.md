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

The Rust reference CLI can sign an unchanged version with a new key in a local distribution root: `pray publish --root ./prayers --signing-key PATH --resign`. It republishes from the local package source while keeping the version's first-publish time and yank state.

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
- its recorded `signature` verifies against its own recorded signing identity.

An implementation with required protocol descriptors MAY require those descriptors to exist before treating the row as current. When the row is current, the publisher MUST preserve the artifact and complete version row.

When the row is absent or not current, the publisher builds and writes the artifact and version row. If a row for that version already exists, publish MUST preserve its `yanked` value. Publish MUST preserve `published_at` when the prior artifact passes its hash check and its package tree and prayspec match the current package. A changed tree or prayspec receives the current publish time. Artifact encoding changes do not change the first-publish time when package content is unchanged.

An ordinary publish MUST NOT rebuild a row because the current publisher differs from the one recorded in it. A verified row already attests the stored bytes under its recorded key or legacy signing identity, and a second publisher re-running publish over unchanged packages restates authorship without republishing anything. In a distribution tracked in version control, that rewrites every current row and buries a real release in identity churn. A row carrying no `signature` is not current, so publish repairs it.

An implementation that offers `--resign` for a local root MUST require an ed25519 signing key supplied by `--signing-key` or `PRAY_SIGNING_KEY`. It MUST rebuild each selected version from the current local package source, write its artifact, and replace the version row with the new signing fields. The same preservation rules for `yanked` and `published_at` apply. A publisher MUST NOT sign stored artifact bytes solely because their catalog hash and legacy digest match: those values can be rewritten together without proving that the artifact matches the current package tree. The Rust reference CLI rejects `--resign` for server destinations and with `--dry-run`.

Registry merge identity MUST exclude `published_at`. A merge of rows that differ only in `published_at` keeps the receiving registry's value.

`published_at` is optional. When present, it MUST be a JSON integer containing whole UTC seconds since `1970-01-01T00:00:00Z`. Its inclusive range is 0 through 253402300799, ending at `9999-12-31T23:59:59Z`. Producers MUST omit an unknown value. They MUST NOT emit a string, a number with a non-zero fractional part, a negative number, an out-of-range integer, or `null`. Registry and federation package-version payloads use this same representation.

Readers MAY accept numeric strings or RFC 3339 strings written by earlier reference clients for migration. They MAY treat JSON null as absent, matching catalogs that serialized an unknown value as null. They MUST truncate any legacy fractional second toward the preceding whole second and normalize the value to the canonical integer on the next metadata write. Other invalid representations fail validation.

The package signature contract in RFC 0050 remains unchanged. `published_at` is catalog metadata and is not part of the signed payload.

## Implementation notes

The reference implementation covers local preservation and legacy timestamp normalization in the Rust, Ruby, and TypeScript publish tests. Rust, Ruby, and TypeScript accept legacy numeric strings, RFC 3339 strings, and JSON null as absent. The registry schema test rejects strings, numbers with non-zero fractional parts, negatives, out-of-range integers, and null. The Rust registry identity test covers timestamp-only merge input. Only Rust exposes ed25519 publish signing and `--resign` today. Its publish tests cover explicit key upgrade, rotation, and replacement of a self-consistent but false stored artifact from local package source.

## Security considerations

An ed25519 package signature authenticates `artifact_hash` and `tree_hash`. It does not authenticate the human `signer` label or optional `signer_fingerprint`; those are catalog claims unless a client independently binds them to the signing key. A legacy `sha256:` digest is not a public-key signature. Preservation is not an authorization decision: whoever may publish a package may publish it, and declining to restate the signer neither grants nor withholds that. A publisher verifies stored artifact bytes against `artifact_hash` before skipping an unchanged version. It compares the stored artifact path with the expected relative path before reading, so catalog metadata cannot select another filesystem path for this check.

Yank remains an explicit metadata operation. Publish does not clear a yank.

## Drawbacks

An unchanged publish does not refresh derived annotations or replace a valid artifact with a newly encoded copy. Those operations need an explicit contract if distribution operators require them. Existing catalogs receive a one-time representation change when a reference client migrates a legacy timestamp. `--resign` replaces the version's previous signer and signature fields; it does not retain a signature history.

## Rationale and alternatives

Rebuilding every package and copying only `published_at` still performs unnecessary archive and catalog writes. It also leaves no-op behavior dependent on whether an archive encoder emits deterministic bytes.

Comparing only `tree_hash` could retain a missing or corrupted artifact. Verifying the expected stored artifact preserves repair behavior.

Comparing the row's signing fields against the publish input, rather than verifying the row against itself, was the earlier draft of this check. It makes a no-op publish depend on who runs it: the reference client derives `signer` from `PRAY_SIGNER`, `USER`, or `USERNAME`, so two maintainers of one distribution rewrite each other's rows on every release.

RFC 0050 already treats artifact bytes as immutable and yank as metadata-only. Preserving a verified row follows those existing rules.

## Prior art

RFC 0050 separates immutable artifact bytes from mutable yank metadata. This RFC applies that separation to repeated publish operations.

## Unresolved questions

Whether a later signature contract should sign catalog timestamps remains open. This RFC does not change the signed payload.
