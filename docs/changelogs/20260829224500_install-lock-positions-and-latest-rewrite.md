# Install lock positions and latest rewrite

## Participants

Andrei Makarov

## Decisions

pray update --latest rewrites every matching package declaration, keeps indent and extra keywords, and parses the new Prayfile before writing. pray update --latest --dry-run prints planned constraint moves and skips write and install. --json with --dry-run still errors.

pray install records managed marker open_line and close_line from the patched file that was written, so pray verify matches after local unmarked text or marker order differs from a fresh compose. Locked install still compares Prayfile.lock to a fresh compose so a local compose source change is refused. Frozen install still compares on-disk bytes to the ideal render. TypeScript and Ruby CLIs share the constraint rewrite. Marker relocate stays on the Rust reference CLI.

## Effects

Consumer Prayfiles that declare the same package in compose and tree no longer get a half-updated constraint pair. After local unmarked lines, pray install refreshes lock positions and pray verify succeeds.

## Next

Publish 1.9.1 after review. Tag v1.9.1 and cut a GitHub Release so upgrade notices resolve.

## Source

usr/docs/changelogs/20260829222000_ci-gate-repairs.md
CHANGELOG.md 1.9.1
