# Audit follow-ups: resolve, transport, patch, torrent

## Participants

- Andrei Makarov

## Decisions

- Close the four open audit items on branch feature/audit-followups-resolve-transport-patch: work-queue transitive resolve, unified bounded HTTP/torrent fetch, managed-patch install, and stream-to-temp torrent promote.
- Prayfile-declared packages keep their manifest constraints; dependency constraints apply only when synthesizing transitive packages.
- Managed patch preserves unmarked text when existing content already has overlapping managed spans; otherwise install rewrites wholesale so corrupted destinations can be repaired.

## Effects

- Resolve walks a work queue, loads required transitive path/source deps, merges constraints among transitive-only packages, and keeps Prayfile package order for render.
- Registry install and transport adapters share pray-core fetch::download_registry_artifact for bounded HTTP with torrent-sidecar preference.
- Install patches existing rendered files instead of always replacing them; unignored install_preserves_unmanaged_content_when_patching_rendered_files covers the contract.
- Torrent pieces stream into a temp file with incremental hash checks and size quotas before promote; registry unpack still stages then renames into cache.

## Next

- Nightly fuzz and blocking mutants baseline remain from the audit.
- Unpack/signature still load the full artifact after torrent download; stream unpack is a later cut if memory pressure shows up.

## Source

- Audit: usr/docs/issues/20260729115833_engineering-audit-post-quality-checks.md
- Prior deep fixes: usr/docs/issues/20260729121659_deep-safety-fixes.md
- Changelog: usr/docs/changelogs/20260729122836_audit-followups-resolve-transport-patch.md
