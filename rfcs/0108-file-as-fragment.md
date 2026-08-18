# RFC 0108: File export as compose fragment

- Feature Name: file-as-fragment
- Type: Standards Track
- Status: Experimental
- Created: 2026-08-18
- Author: Andrei Makarov
- Relates: RFC 0020, RFC 0030, RFC 0102

## Summary

A `type: "file"` export MAY be inlined into a `compose` destination as one managed span. Publishers keep shipping `file` for `file:` consumers. A fragment export MUST NOT satisfy `file:`.

## Motivation

Community-health packages export one UTF-8 document as `file` so `pray "name", file: "CONTRIBUTING.md"` stays exclusive and unmarked (RFC 0030, RFC 0102). A consumer that wants project notes around that document today gets a type mismatch: compose selects the Fragment role, `export_kind_matches_role` requires `kind == "fragment"`, `load_export_bodies` skips non-fragment kinds, and `should_inline_export` skips them (`destination.rs`, `resolve_exports.rs`, `render_compose.rs`). Shipping the same bytes again as `fragment` duplicates the export. Retargeting `file` to `fragment` breaks `file:` consumers. Nesting `file:` inside `compose` remains a parse error (`manifest_parse/blocks.rs`).

## Guide-level explanation

Publishers keep:

```ruby
spec.exports = {
  "contributing" => { type: "file", path: "exports/CONTRIBUTING.md", default_path: "CONTRIBUTING.md" }
}
```

Exclusive consumers keep `pray "sample/community-contributing", "~> 1.0", file: "CONTRIBUTING.md"`. Bytes stay unmarked after `((pray:…))` substitution.

Compose consumers write:

```ruby
compose "CONTRIBUTING.md" do
  pray "sample/community-contributing", "~> 1.0"
  pray ".agents/contributing-notes.md"
end
```

Pray inlines the file body as one managed span with the same marker pair, span checksum, and lock record as a fragment (RFC 0030). Local embeds stay unmarked. The same path still has one writer: `file:` or `compose`, not both.

`export: "contributing"` selects that file when the package also ships fragments. Auto-select without `export:` uses the file when the package has no fragment export and exactly one file export. Folder or skill exports beside that file do not block auto-select. Binary file bytes fail compose. Other kinds, including `template`, stay out of this downcast.

`render.header` stays project-wide (`RenderPolicy.header` in `manifest.rs`). A Prayfile that composes `AGENTS.md` with the default header also prepends the Agent context banner to `CONTRIBUTING.md`.

## Reference-level explanation

Key words follow RFC 2119.

When `export:` / `exports:` names an export, compose MUST inline it when the kind is `fragment` or `file` and the bytes are UTF-8.

When those keywords are omitted and the destination role is Fragment, implementations MUST:

1. If the package has one or more `fragment` exports, select among fragments only (today's rule: one succeeds, several require `export:`).
2. If the package has no `fragment` export and exactly one `file` export, select that file.
3. If the package has no `fragment` export and several `file` exports, fail resolution and require `export:`.
4. If the package has neither, fail with no compatible export.

The File role MUST match `kind == "file"` only. A `fragment` export MUST NOT be selected for `file:`.

Compose MUST wrap a selected file body with the same open and close markers as a fragment, run `substitute_pray_symbols`, and record a managed span whose `ideal_checksum` is the substituted body without markers. Non-UTF-8 file bytes MUST fail resolve or render with an integrity error. Folder, skill, `template`, and other kinds MUST NOT be inlined into compose under this RFC.

`file:` nested inside `compose` or `tree` MUST remain a parse error. Same-path `file:` plus `compose` stays exclusive.

Exclusive `file:` destinations MUST keep today's contract: unmarked UTF-8 after substitution, whole-file verify against reconstructed expected bytes.

## Implementation notes

Today Fragment matches `fragment` only. `load_export_bodies` continues past `kind != "fragment"`. Named `export:` of a file already selects the name (`select_exports` first branch) and then drops the body.

Reference changes belong in `export_kind_matches_role` or `select_exports` for auto-select, `load_export_bodies`, and `should_inline_export`. Fixtures: file-only package in compose; file plus fragment auto-selects the fragment; named file beside fragments; binary file fails; `file:` consumer byte-equal to 1.8.1.

Polyglot CLIs MUST match the Rust fixture after this RFC is Stable (RFC 0100).

## Security considerations

A file body in compose is a reconstructed span. Tampered span bytes fail verify. Exclusive `file:` verify is unchanged. Binary payloads MUST NOT be coerced into a span.

## Registrar

No new Prayfile keywords, lockfile fields, or marker grammar. The Fragment role gains `file` as a selectable kind under the auto-select rules above.

## Drawbacks

Compose output of a file export is marked and may carry the project header. It is not byte-equal to the exclusive `file:` destination of the same export. Contributors who need unmarked exclusive files keep `file:`.

## Rationale and alternatives

File downcasts to a span because the publisher already shipped a whole UTF-8 document. A fragment does not upcast to exclusive `file:` because that destination promises unmarked whole-file bytes.

Rejected: dual `fragment` plus `file` exports; retargeting community packages to `fragment`; Copier three-way merge (RFC 0030 reconstructs, it does not merge); named slots inside exclusive `file:` (better when the destination MUST stay unmarked; a later RFC). Doing nothing leaves compose consumers copying files by hand.

## Prior art

mdBook `{{#include}}` inlines a whole file into a chapter. AsciiDoc `include::` does the same. Bundler git gems still lock `revision` while the source stays a repository; here the source stays `type: "file"` while compose consumes it as a span.

## Unresolved questions

Whether auto-select of a lone file should warn that the kind is `file`, or stay silent.

Whether compose of a file export SHOULD default that destination's header off. That is a second contract (`RenderPolicy.header` is one boolean today). This RFC leaves header unchanged.

Whether Ruby and TypeScript land in the same implementation PR as Rust.

## Future possibilities

Per-destination `header:` on `compose`. Named slots inside exclusive `file:` for unmarked splice. Silence on a file-as-span uses the existing lock flag (RFC 0030).
