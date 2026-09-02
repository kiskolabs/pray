# Provisioned destination write safety

## Participants

Andrei Makarov

## Decisions

RFC 0033 is the contract for refuse-clobber, symlink dest reject, provisioned lock ledger, and hash-gated prune. Marker dialects stay unclaimed. Native include plus unmarked file: is the home splice. Compose into .zshrc is not required.

## Effects

Exclusive file: and tree: leaves refuse unmanaged dest bytes, refuse symlink dests, record path hash package export in Prayfile.lock, and prune a dropped leaf only when on-disk hash still matches. Lock paths are validated before prune. Destination writes finish before Prayfile.lock changes, so a failed update retains the previous ownership ledger for retry. Plan lists every provisioned path, compares dest to reconstructed expected bytes, and reports the same refusal as apply. TypeScript serialization writes explicit empty top-level lock arrays so its own generated lock can be read on retry. A leading tilde is rejected only in a destination string; source and local package paths may use a literal tilde directory. Documentation states compose is HTML comments only and that compose of .json or .zshrc is currently legal and host-invalid. Operator note uses pray --path HOME with file: ".zshrc".

## Next

Stabilize RFC 0033 after polyglot fixtures match and the Windows no-follow equivalent is specified. Marker dialects remain optional and must not claim ids/0032 until scheduled. RFC 0034 and RFC 0108 cover unused-spec strike, file-as-fragment compose, per-destination header, and fail-closed compose dests on this branch.

## Source

RFC 0033
usr/docs/issues/20260902104500_home-prayfile-backlog-claims-audit.md
crates/pray-core/src/render_dest.rs
rubygems/pray-cli/lib/pray/render_dest.rb
npmjs/pray-cli/src/render/dest.ts
schema/lockfile.schema.json provisioned array
