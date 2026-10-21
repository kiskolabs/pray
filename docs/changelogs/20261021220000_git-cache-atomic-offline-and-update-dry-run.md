# Git cache atomic publish, offline lazy fill, and update dry-run conflicts

## Decisions

Shallow plus partial clones stay the default for distribution git sources. Branding rename maps stay design-only until an RFC. Doctor migrate only quarantines project full clones; it does not delete the shared bare.

## Effects

Shared bare clones publish through a temporary directory, an advisory lock, and a ready marker. Offline materialize sets `GIT_NO_LAZY_FETCH` and prefers `git show` for lazy file fill at the catalog root only. `pray update --latest --dry-run` lists would-conflict path-fork paths and refuses write. Update confirms the new git revision is materializable on the catalog path verify uses before writing. `pray doctor` reports layout; `--migrate-git-cache` quarantines matching project full clones.

## Next

None for this ship slice beyond validation recorded in the live-work issue.

## Source

docs/issues/20261021220000_git-cache-concurrency-offline-and-update-simulation.md
