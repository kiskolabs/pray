# Local exclusive tree and file provision

## Participants

Andrei Makarov

## Decisions

Local compose stays marked spans. Package file: and package tree stay.

RFC 0115 proposed exclusive copy of a project-local file or directory to another dest. Status Rejected means that proposal is not adopted. Fail-parse for those forms lives in RFC 0010.

A later pass rewrote RFC 0115 as the withdrawn proposal so Status and body name the same thing.

## Effects

pray ".agents/zshrc", file: ".zshrc" and a local path inside tree fail parse.

## Next

Ships as 1.17.0. See usr/docs/issues/20260916223500_prepare-1-17-0-release.md.

## Source

rfcs/0115-local-exclusive-provision.md
rfcs/0010-core-formats.md
rfcs/0033-provisioned-destination-safety.md
