# CLI parity for local file JSON, sidecars, and checksum lines

## Participants

Andrei Makarov

## Decisions

Canonical Prayfile JSON keeps file on exclusive local entries so manifest_hash matches across CLIs. schema/manifest.schema.json allows that field.

pray repo init writes v1/distribution.json with empty sidecars. pray serve honors Range with 206. Artifact .praytorrent.json write stays on the Rust CLI.

install, plan, and outdated print local checksum lines (checked / was) in the Ruby and TypeScript CLIs as well as Rust.

RFC 0010 compose and file: examples match RFC 0115. RFC 0033 records package local on provisioned leaves. RFC 0115 empty, symlink, and overlap refusals have tests in all three CLIs.

A later pass rejected RFC 0115. Local exclusive copies are not a product feature. Canonical local.file is unused.

## Effects

A Prayfile with pray ".agents/zshrc", file: ".zshrc" no longer hashes differently depending on which CLI computed manifest_hash.

Missing v1/distribution.json still means no swarm sidecar. Repo init no longer leaves that file absent on Ruby and TypeScript.

## Next

Covered by usr/docs/changelogs/20260916222000_publish-torrent-descriptor-parity.md.

## Source

usr/docs/issues/20260916193000_local-exclusive-provision.md
usr/docs/issues/20260916184000_distribution-sidecar-opt-in.md
usr/docs/issues/20260916172100_local-compose-checksum-report.md
rfcs/0010-core-formats.md
rfcs/0033-provisioned-destination-safety.md
rfcs/0062-distribution-protocols.md
schema/manifest.schema.json
CHANGELOG.md 1.17.0
