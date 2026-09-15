## Participants

Andrei Makarov

## Decisions

Published registry metadata must not echo spec.upstream. Consumers of a published fork do not fetch upstream. Catalog JSON is what remote install reads. Echoing would invite a compose edge that RFC 0114 forbids. Remote install may copy name and constraint from the packaged spec after unpack. A second copy in catalog JSON would drift from the archived prayspec. RFC 0060 does not list upstream. The registry schema forbids extra version properties.

Ruby pray update --latest must rewrite Prayfile constraints the same way Rust and TypeScript do, including dry-run. --json and --major stay rejected on this CLI.

Stay on the current branch. No new RFC. The contract lives in RFC 0114.

## Effects

RFC 0114 unresolved questions are closed. Schema validation rejects an upstream object on a package version. Rust and Ruby publish of a path fork omit upstream from catalog JSON and keep the pin in the prayspec.

Ruby pray update --latest rewrites a Prayfile pessimistic constraint when the registry latest is outside it, then refreshes. --latest --dry-run prints the planned constraint and does not write.

## Next

None.

## Source

RFC 0114
RFC 0060
schema/registry.schema.json
usr/docs/changelogs/20260915102100_registry-metadata-and-ruby-latest.md
usr/docs/changelogs/20260915090000_align-upstream-publish-clients.md
usr/docs/changelogs/20260908140000_package-upstream.md
