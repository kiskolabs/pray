# Prayspec maintainers and pray_version alias

## Participants

Andrei Makarov

## Decisions

Parsers accept spec.maintainers. spec.pray_version stores prayfile_version. RFC 0011 example uses prayfile_version.

## Effects

Copying the RFC 0011 worked example no longer fails on pray_version. A maintainers list round-trips through render.

## Next

Covered by parser tests in Rust, Ruby, and TypeScript.

## Source

usr/docs/issues/20260916181000_prayspec-maintainers-and-pray-version-alias.md
rfcs/0011-prayspec-and-package.md
CHANGELOG.md 1.17.0
