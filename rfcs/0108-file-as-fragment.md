# RFC 0108: File export as compose fragment

- Feature Name: file-as-fragment
- Type: Standards Track
- Status: Stable
- Created: 2026-08-18
- Author: Andrei Makarov
- Relates: RFC 0020, RFC 0030, RFC 0034, RFC 0102
- Requires: RFC 0030

## Summary

A `type: "file"` export MAY be inlined into `compose` as one managed span. Exclusive `file:` stays unmarked. Compose of JSON, binary, or an unknown dest type is a render error that names `file:` as the unmarked path.

## Motivation

Community-health packages export one UTF-8 document as `file` so `pray "name", file: "CONTRIBUTING.md"` stays exclusive and unmarked. A consumer that wants project notes around that document today gets a type mismatch: compose selects the Fragment role and skips non-fragment kinds. Dual `fragment` plus `file` exports duplicate bytes. Retargeting `file` to `fragment` breaks `file:` consumers.

## Guide-level explanation

Publishers keep `type: "file"` with `default_path`. Exclusive consumers keep `file: "CONTRIBUTING.md"`. Bytes stay unmarked after `((pray:…))`.

Compose consumers write `compose "CONTRIBUTING.md"` and bind the same package. Pray inlines the file body as one managed span. Local embeds stay unmarked. The same path still has one writer: `file:` or `compose`, not both.

Auto-select without `export:` uses the file when the package has no fragment and exactly one file export. Folder exports beside that file do not block auto-select. Binary file bytes fail compose. They are not copied as a span.

The Agent context banner defaults on `AGENTS.md` only. `compose "CONTRIBUTING.md", header: false` is explicit off. `header: true` forces the banner. Banner text mentions `.agents/` only when the dest basename is `AGENTS.md`.

`compose "config.json"` fails and names `file: "config.json"`. The same for a binary dest or an unknown type such as `.zshrc`. There is no `markers:` override until a dialect RFC claims `ids/0032`.

## Reference-level explanation

Key words follow RFC 2119.

When `export:` / `exports:` names an export, compose MUST inline it when the kind is `fragment` or `file` and the bytes are UTF-8.

When those keywords are omitted and the destination role is Fragment, implementations MUST:

1. If the package has one or more `fragment` exports, select among fragments only (one succeeds, several require `export:`).
2. If the package has no `fragment` export and exactly one `file` export, select that file.
3. If the package has no `fragment` export and several `file` exports, fail resolution and require `export:`.
4. If the package has neither, fail with no compatible export.

The File role MUST match `kind == "file"` only. A `fragment` export MUST NOT be selected for `file:`.

Compose MUST wrap a selected file body with the same open and close markers as a fragment, run `substitute_pray_symbols`, and record a managed span whose `ideal_checksum` is the substituted body without markers. Non-UTF-8 file bytes MUST fail compose with an integrity or render error. Folder and other kinds MUST NOT be inlined. Binary MUST NOT be copied as a span.

`file:` nested inside `compose` or `tree` MUST remain a parse error. Exclusive `file:` stays unmarked UTF-8 after substitution, or raw bytes when the export is not UTF-8.

Compose dests whose basename is `AGENTS.md` write the Agent context banner unless `render header: false` or the dest sets `header: false`. Other dests write that banner only when the dest sets `header: true`. `compose` MAY take `header: true` or `header: false`. Other dest keywords on `compose` MUST fail parse. Tree dests MUST reject `header`.

Compose MUST fail closed unless the dest extension is `md`, `markdown`, `html`, or `htm`. A `.json` dest MUST fail as JSON. A dest with a binary extension MUST fail as binary. Any other dest MUST fail as an unknown type. Each error MUST name `file: "<dest>"` as the unmarked path. Parse of the compose statement MAY succeed; render MUST fail.

## Implementation notes

`export_kind_matches_role` Fragment stays `fragment` only. Auto-select of a lone file lives in `select_exports`. `load_export_bodies` loads UTF-8 `file` bodies. `should_inline_export` inlines `fragment` and `file`.

## Security considerations

A file body in compose is a reconstructed span. Binary payloads MUST NOT be coerced into a span. JSON and unknown dests MUST NOT receive HTML comment markers.

## Registrar

Compose keyword `header` (boolean). No lockfile fields. No marker grammar. The Fragment role gains `file` as a selectable kind under the auto-select rules above.

## Drawbacks

Compose output of a file export is marked and is not byte-equal to exclusive `file:`. Destinations that used the project-wide banner on non-`AGENTS.md` files lose it unless they set `header: true`.

## Rationale and alternatives

File downcasts to a span because the publisher already shipped a whole UTF-8 document. A fragment does not upcast to exclusive `file:`. Fail closed keeps HTML comments off host-invalid files until a dialect RFC exists.

Rejected: dual exports; Copier merge; named slots inside exclusive `file:`; shebang preservation (not scheduled); claiming `ids/0032`.

## Prior art

mdBook `{{#include}}`. AsciiDoc `include::`. RFC 0034 unused kinds stay out of this downcast.

## Unresolved questions

Whether auto-select of a lone file should warn that the kind is `file`, or stay silent.

## Future possibilities

Named slots inside exclusive `file:`. Marker dialect RFC 0032 if scheduled. Conformance packs (RFC 0100).
