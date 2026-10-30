# RFC 0119: Related prayers

- Feature Name: related-prayers
- Type: Standards Track
- Status: Experimental
- Created: 2026-10-30
- Author: Andrei Makarov
- Relates: RFC 0011, RFC 0040, RFC 0060, RFC 0117
- Requires: RFC 0011

## Summary

A prayspec MAY declare `related` as advisory references to other prayers. A reference can name a package, a neighbor in the same source, a source-root location, or an absolute URI. Implementations expose these references without adding resolver edges or fetching them.

## Motivation

Prayers already refer to related guidance in prose, but a parser cannot identify those relationships. A dependency is the wrong contract because related guidance need not be installed, version-compatible, or hosted by the same distribution point. Authors also need to refer to a neighboring prayer without repeating the source location.

## Guide-level explanation

An author lists related prayers in the prayspec:

```manifest
Package::Specification.new do |spec|
  spec.name = "sample/review"
  spec.related = [
    "sample/testing",
    "../security",
    "/shared/accessibility",
    "https://prayers.example/operations/incident"
  ]
  spec.files = ["README.md"]
end
```

`sample/testing` is a package-name shorthand. `../security` addresses a neighbor relative to the current prayer in the same source. `/shared/accessibility` starts at the current source root. The HTTPS value is self-contained and can name another source.

`pray explain sample/review` prints the references under `related prayers`. An empty list prints `none`.

## Reference-level explanation

### Prayspec and canonical model

`related=` is an allowed prayspec assignment. Its value MUST be an array of strings. The canonical JSON field is `related`, an array of unique strings in lexical order. An omitted assignment canonicalizes to an empty array.

Each reference MUST be non-empty after trimming and MUST NOT contain a NUL, an ASCII control character, or a backslash. Implementations preserve the trimmed reference. A package shorthand MUST NOT contain a query or fragment; URI and locator forms MAY contain them. An absolute URI MUST NOT contain a user-information component.

The accepted reference forms are:

- package shorthand: exactly two non-empty path segments, `namespace/name`, with neither segment equal to `.` or `..`;
- relative source reference: starts with `./` or `../` and contains at least one later non-empty segment;
- source-root reference: starts with one `/`, interpreted from the current distribution or path-source root rather than the host filesystem root;
- absolute URI: starts with an RFC 3986 scheme followed by `:`.

A relative or source-root reference is a locator, not a package name. Its target may be a prayspec, package directory, metadata document, or human-facing resource. This RFC does not infer a target kind from a suffix.

### Behavior

Related references are advisory metadata. They MUST NOT:

- enter dependency resolution or lockfile dependency arrays;
- cause install, update, search, verify, or render to fetch the target;
- imply compatibility, authorship, endorsement, trust, or installation order;
- fail because the target is absent;
- make a reference cycle a dependency cycle.

The parse APIs expose the canonical array. `pray explain` displays it for a resolved package. Package archives include the prayspec as before, so the field remains available after unpacking.

### Source resolution

This RFC records references but does not add a resolve-related command. A later resolver MAY use the source that supplied the declaring prayer as the base for package, relative, and source-root references. An absolute URI supplies its own base. Resolution MUST retain the no-install and no-trust semantics unless a later RFC changes them.

## Security considerations

Parsing and display do not open the reference, read a path, or make a network request. A consumer that later resolves a reference must treat it as untrusted input, enforce the selected source boundary after normalization, restrict network schemes, reject credential disclosure, and apply its normal package integrity and trust checks.

Authors MUST NOT place credentials, secrets, private filesystem paths, or person-identifying query values in a related reference. A prayspec can be published, and `pray explain` displays each reference as written.

## Registrar

This RFC claims the prayspec and canonical JSON field `related` and the `pray explain` label `related prayers`.

## Drawbacks

The first version records a relationship without proving that its target exists or that two authors mean the same relationship. Raw locators can become stale. Package shorthand is portable only when the receiving source can identify the named package.

## Rationale and alternatives

Dependencies were rejected because they affect resolution and installation. Mining prose was rejected because inferred links are not author declarations. A structured object with reason, label, and relationship kind was deferred because the current need is programmatic visibility of an explicit reference. A string array keeps the declaration compact and can later coexist with a separately named structured field.

The one-word field name `related` follows the generic related-resource term used by established link and metadata vocabularies. `references` was rejected because it usually asserts the narrower, directional relation that the current resource cites or points to the target. `associated` was rejected because it does not name the target kind or the direction of the relation.

## Prior art

The [IANA Link Relation registry](https://www.iana.org/assignments/link-relations/) defines `related` as identifying a related resource. [Dublin Core](https://www.dublincore.org/specifications/dublin-core/dcmi-terms/terms/relation/) defines `relation` as a related resource and narrows [`references`](https://www.dublincore.org/specifications/dublin-core/dcmi-terms/terms/references/) to a resource cited or otherwise pointed to by the described resource. [Schema.org](https://schema.org/relatedLink) uses `relatedLink` for related web pages. [JSON:API](https://jsonapi.org/format/) uses `related` for the link to resources participating in a relationship. This RFC adopts the broad term without adopting those formats' fetch behavior.

## Unresolved questions

Whether distribution metadata should copy related references into each version row so an index client can inspect them without downloading the package archive.

## Future possibilities

A later RFC may define relationship kinds, reason text, verified target identity, bounded related-reference traversal, catalog filters, or an explicit resolver.
