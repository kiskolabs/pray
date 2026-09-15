## Decisions

Path-fork update refresh now runs in the Ruby and TypeScript CLIs as well as Rust. All three rewrite spec.upstream pins on pray update --latest, including dry-run. The Ruby CLI still rejects --json and --major on update.

A rewritten fork prayspec keeps homepage, source_code_uri, changelog_uri, prayfile_version, and metadata in TypeScript the same as Rust and Ruby.

Packing skips a spec.files entry that names the auto-included prayspec so a refreshed fork can still be published. A repeated content path still fails.

Published registry metadata still does not echo upstream.

## Effects

- pray update refreshes a clean path fork in Rust, Ruby, and TypeScript.
- TypeScript pray update --latest rewrites an exact upstream pin and refreshes the tree.
- Ruby pray update --latest rewrites an exact upstream pin and refreshes the tree. --latest --dry-run prints the planned pin and does not write.
- TypeScript rewrite of a fork spec keeps homepage, source URIs, prayfile version, and metadata.
- package and publish accept spec.files that list the package spec and still write that spec once.

## Next

Closed in usr/docs/changelogs/20260915102100_registry-metadata-and-ruby-latest.md. Published registry metadata must not echo spec.upstream. Ruby pray update --latest now rewrites Prayfile constraints.

Later pass 20260915103600: this work ships as 1.15.0. See usr/docs/issues/20260915103600_prepare-1-15-0-release.md.

## Source

RFC 0114
RFC 0061
CHANGELOG.md 1.15.0
usr/docs/changelogs/20260915083000_update-latest-upstream-pins.md
usr/docs/issues/20260915103600_prepare-1-15-0-release.md
