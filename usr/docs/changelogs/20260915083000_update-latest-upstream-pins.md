## Decisions

pray update --latest rewrites a path package spec.upstream constraint when that constraint does not admit the latest resolved upstream version, using the same operator family as Prayfile --latest. It then refreshes the path tree. pray update --latest --dry-run prints the planned pin and does not write the prayspec or path files.

A failed three-way merge names the fork package, the previous and new upstream versions, and every conflicting content path.

Overlay directories stay future work.

## Effects

- Exact = pins stay on pray update.
- pray update --latest admits the latest upstream version, then refresh writes the new exact pin.
- Range constraints that already admit a newer version still move on pray update; --latest still rewrites the operator family when the latest version is outside that range.

## Next

Later pass 20260915103600: this work ships as 1.15.0. See usr/docs/issues/20260915103600_prepare-1-15-0-release.md. Path-tree refresh now runs in Ruby and TypeScript.

## Source

RFC 0114
CHANGELOG.md 1.15.0
usr/docs/issues/20260915103600_prepare-1-15-0-release.md
