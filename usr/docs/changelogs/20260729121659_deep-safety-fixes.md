# Deep safety fixes

## Participants

- Andrei Makarov

## Decisions

- Land audit deep fixes for path safety, serve/sync trust boundaries, publisher allowlists, resource quotas, conflict:fail, and exit code 7 on branch feature/deep-safety-fixes.

## Effects

- Install and render refuse repository-escaping destinations and enforce conflict:fail against prior lockfile managed spans.
- Serve and sync gain connection, body, peer, and artifact resource ceilings; sync requires tree hashes.
- Network fetch failures exit with code 7; CI mutants examine the intended integrity surfaces.

## Next

- Follow with transitive resolve and transport unification once this branch is reviewed.

## Source

- Issue: usr/docs/issues/20260729121659_deep-safety-fixes.md
- Audit: usr/docs/issues/20260729115833_engineering-audit-post-quality-checks.md
