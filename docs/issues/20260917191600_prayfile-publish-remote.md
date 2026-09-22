# Prayfile publish remotes

Prayfile may declare named publish remotes. This note started as research. Implementation follows RFC 0118.

## Participants

Andrei Makarov

## Decisions

Named remotes use the publish keyword. source and distribution: stay consume-side. A dest flag outside the allowlist is a usage error. No remotes keeps required --root or --server. yank, serve, and token share path remotes through --to.

## Effects

RFC 0118 is Experimental. Rust, Ruby, and TypeScript parse publish remotes. pray publish uses them when dest flags are omitted and packs path-owned packages only.

## Next

Ship with the next version. Whether sync --to shares remotes remains open on the RFC.

## Source

rfcs/0118-prayfile-publish-remote.md
usr/docs/changelogs/20260918071000_prayfile-publish-remote.md
rfcs/0010-core-formats.md
rfcs/0040-cli-surface.md
rfcs/0060-distribution.md
rfcs/0117-local-prayers.md
