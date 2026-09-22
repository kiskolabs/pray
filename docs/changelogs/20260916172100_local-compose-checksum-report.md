# Local compose sources re-embed and report checksums

## Participants

Andrei Makarov

## Decisions

pray install re-embeds a changed compose local and prints its sha256 with checked or was. plan and outdated name the same path and checksums. The lockfile stores that checksum on the managed span for package local.

## Effects

AGENTS.md updates when .agents/project.md changes. A no-op reinstall reports the checksum as checked. Footer inventory names local files besides packages.

## Next

Covered by install_local CLI tests and the matching Ruby destination_render and TypeScript render wrap. Ships as 1.17.0.

## Source

usr/docs/issues/20260916172100_local-compose-checksum-report.md
CHANGELOG.md 1.17.0
