# RFC 0114: Package upstream

- Feature Name: package-upstream
- Type: Standards Track
- Status: Experimental
- Created: 2026-09-08
- Author: Andrei Makarov
- Relates: RFC 0011, RFC 0020, RFC 0040, RFC 0060
- Requires: RFC 0011, RFC 0020, RFC 0040

## Summary

A path package MAY declare one upstream package in its `.prayspec`. That pin is provenance for a catalog fork, not a compose dependency. `Prayfile.lock` records the resolved upstream version and hashes. `pray update` refreshes the pin within the constraint and writes upstream files into the path tree.

## Motivation

A catalog that republishes another distribution under its own namespace needs an exact locked base and an explicit upgrade. Copying trees by hand loses the pin. `add_dependency` would pull upstream into consumer compose. Federation sync keeps the same package name.

## Guide-level explanation

```manifest
Package::Specification.new do |spec|
  spec.name = "fork/base"
  spec.version = "1.0.0"
  spec.upstream "sample/base", "~> 1.4"
end
```

The catalog Prayfile names a source that can resolve `sample/base` and keeps the fork as a path package:

```manifest
source "sample", git: "https://example.com/sample-prayers.git"
pray "fork/base", path: "packages/base"
```

`pray install` resolves upstream, verifies hashes, and records them on the fork's lock entry. It does not compose upstream and does not replace path files.

`pray update` (or `pray update fork/base`) resolves a newer upstream within the constraint. When path content files still match the previously locked upstream tree, pray replaces those files and keeps fork identity in `*.prayspec` (name, fork version, upstream field). When path content differs, pray three-way merges and fails closed on overlap.

Consumers of `fork/base` do not fetch `sample/base`.

## Reference-level explanation

Key words follow RFC 2119.

`spec.upstream NAME, CONSTRAINT` is a prayspec method (RFC 0011). Implementations MUST reject `spec.upstream =`. At most one upstream. NAME MUST be a package name and MUST NOT equal `spec.name`. CONSTRAINT uses RFC 0010 rules; omitted CONSTRAINT is `*`. Upstream MUST NOT be treated as `add_dependency`.

Install MUST resolve upstream only when the declaration has `path:`. Remote installs MAY copy name and constraint from the spec as provenance and MUST NOT fetch upstream solely because the field is present.

When resolving a path fork, install MUST pin upstream to the locked version when a lock entry exists and the package is not being updated. `pray update` without a package name, or `pray update` of that fork, MUST resolve upstream from the spec constraint and MAY advance the git source revision (RFC 0020).

Lock field `package.upstream` is optional. When present it MUST include `name`, `version`, `tree_hash`, and `artifact_hash`. `source` is the source handle used to resolve it.

Identity files are `*.prayspec` under the package root. Update MUST NOT overwrite the fork prayspec with the upstream prayspec. After a successful file refresh, implementations MUST rewrite the fork prayspec so `spec.files` lists the fork prayspec path plus the new upstream content files, exports match the new upstream when the tree was a clean replica, and an exact upstream constraint (`=` or a bare version) becomes `= NEW_VERSION`. A range constraint MUST stay.

A content file is any `spec.files` path that is not identity. Clean replica: every old-upstream content file exists in the path tree with identical bytes, and the path tree has no extra content files. Clean replica MUST replace content files from the new upstream and delete content files that existed only in the old upstream.

Otherwise implementations MUST three-way merge each content path in the union of old upstream, new upstream, and the path tree. Take new when local equals old. Keep local when local equals new or old equals new. Otherwise fail with a resolution error that names the path. Extra local content files stay. A path file that equals old upstream and is absent from new upstream MUST be deleted; if it differs from old, fail.

`*.prayspec` is not merged as content.

## Registrar

- prayspec method `upstream`
- lockfile table `package.upstream` with fields `name`, `version`, `source`, `tree_hash`, `artifact_hash`

## Drawbacks

Catalogs must declare the upstream source. An exact `=` pin does not move until the spec changes or update rewrites that exact pin after a successful refresh.

## Rationale and alternatives

Prayfile `upstream:` was rejected; the fork contract belongs on the package. `add_dependency` was rejected; consumers would fetch and compose upstream. `pray sync` was rejected; it does not rename.

## Prior art

Bundler and Cargo lock exact versions while the manifest holds a constraint. Debian records an upstream version beside a downstream revision.

## Unresolved questions

Whether published registry metadata MUST echo upstream. Whether Ruby and TypeScript update refresh must match Rust in the same release.

## Future possibilities

`pray update --latest` rewriting a range into a new exact pin. Overlay directories beside the path tree.
