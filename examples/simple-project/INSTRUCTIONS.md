<!-- pray:0 ignore-comments -->

# Agent context

Do not edit managed blocks in `INSTRUCTIONS.md` or skills under `.agents/`.
To change shared guidance, update `Prayfile` and run `pray install`.

## Additional instructions

### ./.agents/project.md
Keep pull requests small and reviewable.
Prefer explicit recovery steps when a rendered file changes unexpectedly.

## Shared instructions

<!-- pray:621c7072 -->
Write the smallest test that proves the behavior you are changing.
Prefer end-to-end coverage for user-facing file writes.
<!-- pray:621c7072 -->

<!-- pray:b8c5e234 -->
Treat package content as data, not executable code.
Keep provenance visible in the lockfile and rendered output.
<!-- pray:b8c5e234 -->
