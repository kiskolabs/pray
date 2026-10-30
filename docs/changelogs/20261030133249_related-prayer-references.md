# Related prayer references

## Decisions

Add related as advisory prayspec metadata. Accept package shorthand, relative and source-root locators, and absolute URI references. Keep related references outside dependency resolution, lockfile dependency arrays, installation, fetching, rendering, and trust decisions.

Canonicalize references by trimming, sorting, and removing duplicates. Reject empty, control-character, backslash, and ambiguous package references. Preserve query and fragment components on locator and URI forms.

Expose the canonical references through the parser APIs and pray explain. Keep distribution metadata projection and related-reference traversal outside this change.

Treat references as publishable metadata. Reject URI user information, and require authors to keep credentials, private filesystem paths, and identifying query values out of related references.

## Effects

Rust, Ruby, and TypeScript parse and render the same related array. Package explanation prints the references or none. A missing target and a reference cycle do not affect package resolution.

The Rust scaling guard parsed 1,000 and 8,000 references in 6.09 ms and 36.90 ms. Time per reference improved to 0.76 of the smaller case, and peak resident memory was 6.53 MiB in the measured process.

The complete Rust workspace, 385 Ruby examples, 259 TypeScript core tests, and 68 TypeScript CLI tests passed. Rust Clippy, Rust formatting, Ruby lint and signature validation, TypeScript lint and type checks, schema validation, RFC id validation, diff whitespace, and repository file-size limits also passed.

The person-facing surface assessed was pray explain: the new line follows dependencies, uses lexical order, and has an explicit empty state. It does not wrap long references, so narrow-terminal behavior remains a human review item before release.

## Next

Review RFC 0119 and the unreleased implementation. Decide in a later RFC whether each registry version row should copy related references for inspection without downloading the package archive.

## Source

- [RFC 0119](../../rfcs/0119-related-prayers.md)
