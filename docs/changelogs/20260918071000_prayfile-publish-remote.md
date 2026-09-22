# Prayfile publish remotes

## Participants

Andrei Makarov

## Decisions

Prayfile names produce dests with publish. source and distribution: stay consume-side. Tokens and signing-key paths stay out of the statement. Path-owned packages are the default publish set. CLI dest flags must match a declared remote when any remote exists.

A URL dest that ends in do is not a block. The block marker is a preceding space then do.

## Effects

pray publish uses declared remotes when dest flags are omitted. Mixed consumer Prayfiles no longer republish git or registry dependencies. yank, serve, and token accept --to for a path remote. pray repo init appends a prayers path remote when Prayfile has none.

## Next

Ship with the next version. Cite RFC 0118.

## Source

rfcs/0118-prayfile-publish-remote.md
usr/docs/issues/20260917191600_prayfile-publish-remote.md
