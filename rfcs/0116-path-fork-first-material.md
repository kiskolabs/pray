# RFC 0116: Path-fork first material and overlay files

- Feature Name: path-fork-first-material
- Type: Standards Track
- Status: Experimental
- Created: 2026-09-16
- Author: Andrei Makarov
- Relates: RFC 0114, RFC 0011, RFC 0040
- Requires: RFC 0114

## Summary

A path fork with `spec.upstream` copies upstream content files into the path tree on first install or update when that tree has no content yet. Later `pray update` still merges as RFC 0114. Extra local overlay files stay. `pray outdated` lists fork files that differ from the locked upstream.

## Motivation

RFC 0114 install records upstream hashes and does not replace path files. Update refreshes only when the locked upstream version changes. After a first install the versions match, so update writes nothing. Catalog authors copy trees by hand. They cannot list overlay drift without triggering a merge conflict.

## Guide-level explanation

```manifest
Package::Specification.new do |spec|
  spec.name = "fork/base"
  spec.version = "1.0.0"
  spec.files = []
  spec.upstream "sample/base", "~> 1.4"
end
```

```manifest
source "sample", git: "https://example.com/sample-prayers.git"
pray "fork/base", path: "packages/base"
```

`spec.files` lists content. The `*.prayspec` on disk is identity and is not listed there. Empty `spec.files` means no content yet.

`pray install` copies every upstream content file listed in the resolved upstream `spec.files` into `packages/base`, including files under folder exports. It rewrites the fork prayspec so `spec.files` and exports match that replica. The fork name, version, and `spec.upstream` stay.

Add overlay files in the same tree and list them in `spec.files`. They stay on later `pray update`. Unlisted extras stay on disk and are not packed.

`pray outdated` lists fork paths that differ from the locked upstream: changed, local-only, or missing from the fork.

`pray update` still replaces a clean replica and three-way merges edits. `--locked` and `--frozen` do not write the path tree.

## Reference-level explanation

Key words follow RFC 2119.

A path package is eligible when it has `path:` and `spec.upstream`. Content paths are RFC 0114 identity-excluded `spec.files` entries.

The path tree is empty of content when every content path is absent on disk, or when there are no content paths. Mixed presence (some content files exist, some listed paths are missing) MUST fail as a missing package file.

When the tree is empty of content, `pray install` and `pray update` MUST copy the resolved upstream content files into the path tree and rewrite the fork prayspec as a clean replica (RFC 0114). That MUST run even when the locked upstream version already equals the resolved version. `--locked` and `--frozen` MUST NOT write those files.

When any content file exists, install MUST NOT replace path files (RFC 0114). Update MUST use the RFC 0114 replace-or-merge rules when the locked upstream version or tree hash changes.

Overlay files are listed content paths present in the fork and absent from upstream. They MUST remain through merge. Unlisted files MUST NOT be deleted by first material or merge, and MUST NOT be packed until listed.

`pray outdated` MUST report each content path whose bytes differ from the locked upstream tree, each overlay path, and each upstream content path missing from the fork. It MUST name the fork package and the locked upstream name and version.

## Implementation notes

`apply_path_upstream_refreshes` in `resolve_upstream.rs` and the matching Ruby and TypeScript refresh modules. Install calls that refresh when not locked or frozen. Outdated uses the same content maps.

## Drawbacks

An identity-only prayspec with empty `spec.files` is required before first install. Copying a full upstream `spec.files` list before the files exist is treated as empty and then filled.

## Rationale and alternatives

A sibling overlay directory was rejected; extras live in the path tree and `spec.files`. Materialize-only-on-update was rejected because install already pins the version. Dest `tree` / `file:` of a git checkout is not the writable overlay; that overlay is the path tree (RFC 0114).

## Prior art

RFC 0114 merge. Debian keeps downstream files beside an upstream tarball. Stow overlays extra files onto a tree without rewriting the source.

## Unresolved questions

Whether `pray plan` should print the would-copy file list. Whether unlisted extras should be offered for `spec.files` adoption.

## Future possibilities

A sibling overlay root with a prayspec field. Unified diffs per conflicting path on merge failure.
