# Publish feedback and global quiet/verbose

## Participants

Andrei Makarov

## Decisions

Reuse the dry-run publish line formatter for live writes. Report published versus unchanged against the stored-publish match skip. Add global quiet and verbose flags that gate non-error stdout. Document in help and man.

## Effects

Tests: crates/pray-cli/tests/publish_feedback.rs, cli_ux help assertions, npm help.test.ts. Ruby and TypeScript publish paths print the same verbs.

## Next

Quiet policy for vendor and similar writers if operators ask.

## Source

docs/issues/20261021195500_publish-stdout-and-global-quiet-verbose.md
