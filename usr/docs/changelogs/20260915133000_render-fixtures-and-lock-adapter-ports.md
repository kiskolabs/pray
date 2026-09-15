# Render fixtures and lock adapter ports

## Participants

Andrei Makarov

## Decisions

Ship a shared compose dest fixture. Port inspect_locked_destinations to Ruby and TypeScript. Keep RFC 0106 as a subset of embed names.

## Effects

Rust, Ruby, and TypeScript render the compose-fragment fixture to the same dest bytes. They inspect lockfile span packs without resolve. Ruby and TypeScript now export inspect_locked_destinations.

## Next

Resolver fixture pack. PyO3 after field freeze. Mix when a Phoenix repo asks.

## Source

usr/docs/issues/20260915133000_render-fixtures-and-lock-adapter-ports.md
rfcs/0100-conformance.md
rfcs/0106-host-language-lock-adapter.md
fixtures/render/compose-fragment/expected/INSTRUCTIONS.md
rubygems/pray-cli/lib/pray/verify_locked_dest.rb
npmjs/pray-cli/src/verify/locked-dest.ts
