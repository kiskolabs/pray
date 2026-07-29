# Audit follow-ups for resolve, transport, and install patch

## Participants

- Andrei Makarov

## Decisions

- Ship transitive resolve, shared bounded fetch, managed-patch install, and torrent temp promote as one follow-up branch after deep safety fixes.

## Effects

- Required dependencies resolve transitively without silent skip.
- HTTP and torrent artifact download share one bounded core path.
- Install keeps unmarked user text in rendered files when managed spans are still present.
- Torrent download writes pieces to temp before cache promote.

## Next

- Review and merge feature/audit-followups-resolve-transport-patch.
- Then schedule nightly fuzz and mutants baseline work.

## Source

- Issue: usr/docs/issues/20260729122836_audit-followups-resolve-transport-patch.md
- Audit: usr/docs/issues/20260729115833_engineering-audit-post-quality-checks.md
