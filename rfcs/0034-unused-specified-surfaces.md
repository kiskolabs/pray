# RFC 0034: Unused specified surfaces

- Feature Name: unused-specified-surfaces
- Type: Standards Track
- Status: Experimental
- Created: 2026-09-02
- Author: Andrei Makarov
- Relates: RFC 0010, RFC 0011, RFC 0030, RFC 0031, RFC 0033
- Requires: RFC 0010, RFC 0011, RFC 0030, RFC 0031

## Summary

Strike specified-but-unshipped render and package surfaces. Destination DSL maps paths. Do not load adapters. Do not add export roles or render modes. Writes ignore nothing that the parser still accepts.

## Motivation

RFC 0011 and RFC 0030 name adapters, extra export kinds, extra render modes, `section_markers`, `line_endings`, and origin-tagged dest files. Parsers store some of those fields. Render never reads them. Origin tags were never written. Leaving the claims invites implementing the wrong product.

## Guide-level explanation

A prayspec may still parse `spec.adapters`. Install does not open those paths. `compose` and `tree` already name dest files.

Supported export types remain `fragment`, `file`, and `folder`. `skill` is a deprecated folder alias and still parses until version 2. A `template`, `command`, `rule`, `asset`, or `bundle` type matches no destination role and is not selected.

`render mode: :managed` is the only render mode. Dry-run is `pray plan` or `pray render --check`. Offline copies are `pray vendor`. A personal dest is an in-root `compose` or `file:` path, not `mode: :local`.

`render section_markers:` and `render line_endings:` are parse errors. Hashing already normalizes line endings.

Package remove of exclusive `file:` and `tree:` leaves follows RFC 0033. Dest files are not tagged.

## Reference-level explanation

Key words follow RFC 2119.

`spec.adapters` MAY parse as a string map. Implementations MUST NOT load adapter files to choose dest paths or to spell markers.

Export `type` values other than `fragment`, `file`, `folder`, and the deprecated alias `skill` MUST NOT match a destination role. Implementations MUST NOT invent dest types for them.

Parsers MUST reject `render mode` other than `managed`. They MUST reject `section_markers` and `line_endings` on `render`. Supported `render` fields are `mode`, `conflict`, `churn`, and `header`.

RFC 0031 remove step 4 (origin-tagged directory delete) is superseded by RFC 0033 hash-gated prune. Implementations MUST NOT write `.pray-origin.toml` or origin front matter into dest files.

## Security considerations

Unused adapter paths MUST NOT become a load or execute surface. Origin tags in dest files MUST NOT reappear as a prune mechanism.

## Registrar

No new keywords. `render` drops `section_markers` and `line_endings`.

## Drawbacks

Prayfiles that set the dropped `render` fields fail parse. Packages that only export the dropped kinds still resolve as having no compatible role.

## Rationale and alternatives

Restate unused rather than ship adapters or extra kinds. Honoring `line_endings` on write would fight hash normalization. Mapping `template` onto `file` would invent a role the dest DSL does not have.

Rejected: loading adapter TOML; `mode: :check` / `:local` / `:vendor`; origin comments in dest files; claiming `ids/0032`.

## Prior art

RFC 0033 lock ledger instead of dest tags. Destination DSL (RFC 0102) instead of per-tool adapter files.

## Unresolved questions

Whether `spec.templates` should be struck in a later hygiene RFC the same way. This RFC leaves that map parsed and unused.

## Future possibilities

RFC 0108 file-as-fragment. Marker dialects remain unclaimed (`ids/0032`).
