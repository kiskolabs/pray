# RFC 0121: Managed span adopt-if-identical

- Feature Name: managed-span-adopt-if-identical
- Type: Standards Track
- Status: Experimental
- Created: 2026-11-12
- Author: Andrei Makarov
- Relates: RFC 0030, RFC 0033
- Requires: RFC 0030

## Summary

On write, an active managed span whose on-disk body checksum equals the fresh render `ideal_checksum` MUST NOT fail the conflict gate when that checksum differs from the previous lock ideal. A body that equals neither MUST still fail closed. No general `--force` flag.

## Motivation

Operators hit a loop when `Prayfile.lock` ideals are stale and the destination managed body already matches what a fresh render would write (for example after a pull that updated the dest and a local compose source but kept an old lock). `install --locked` says to rerun install. Plain install reports `was edited` because the conflict check compared only to the previous lock ideal. Moving the destination aside works; it should not be required when the write would not discard divergent human edits.

## Guide-level explanation

Edit a listed local compose file. Update the matching managed block in the destination so its body matches the new source. Leave `Prayfile.lock` on the old `ideal_checksum`.

`pray install` refreshes the lock and leaves the matching body in place. It does not ask for `--force`.

Edit the managed block to different bytes that match neither the lock nor a fresh render. `pray install` fails and names move-aside or restore as recovery.

## Reference-level explanation

Key words follow RFC 2119.

This RFC amends RFC 0030 section 45 for active managed spans when a previous lock is present and `render conflict: :fail` applies.

For each previous-lock managed span that remains active in the fresh render and whose marker pair is present on disk:

1. Let `disk` be the semantic checksum of the on-disk managed body (same algorithm as `ideal_checksum`).
2. Let `locked` be that span's previous-lock `ideal_checksum`.
3. Let `fresh` be the `ideal_checksum` of the matching fresh-render managed span with the same id.
4. If `disk` equals `locked`, the conflict check for that span MUST succeed.
5. Else if `disk` equals `fresh`, the conflict check for that span MUST succeed (adopt). Materialization MAY skip rewriting identical bytes and MUST refresh the lock record when ideals move.
6. Else the implementation MUST fail with a render error. The error MUST name the span id, target, and `package::export`. The error MUST describe recovery: move the destination aside and rerun `pray install`, or restore the managed block to the locked bytes. The error MUST NOT offer a general force flag.

Retired and unknown marker rules in RFC 0030 are unchanged. Provisioned `file:` and `tree:` adopt rules in RFC 0033 are unchanged.

Implementations MUST NOT add a CLI flag whose purpose is to overwrite divergent managed bodies.

## Implementation notes

Rust: `render_conflict.rs`. Ruby: `render_reconcile.rb`. TypeScript: `render/conflict.ts`. Fresh ideals come from the rendered target span list already passed into the conflict gate.

## Drawbacks

A destination that already equals the fresh ideal while the lock is wrong will quiet the conflict. Operators who relied on install failing until they inspected a stale lock lose that signal. Plan and drift still see lock movement.

## Rationale and alternatives

Adopt-when-identical matches RFC 0033 provisioned step "bytes equal expected: adopt" without widening to marker dialects inside RFC 0033.

Rejected: general `--force`; scoped `--accept-identical-local` (redundant once adopt is default); comparing raw local source paths instead of fresh `ideal_checksum` (checksum semantics and compose rules live on the fresh span).

Doing nothing keeps the `--locked` / install loop and the move-aside workaround.

## Prior art

RFC 0033 provisioned refuse-clobber adopt when dest bytes equal expected. microsoft/apm content-identical-to-source checks. RFC 0030 nobody-edits for divergent managed bodies.

## Unresolved questions

Whether verify and drift should emit a distinct "stale lock, content agrees" finding instead of `custom_implementation` when disk equals fresh and differs from lock. Whether `silenced` on `[[managed_span]]` should be used or removed.
