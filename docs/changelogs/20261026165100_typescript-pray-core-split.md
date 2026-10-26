# TypeScript pray-core split and packaging map

## Participants

Andrei Makarov

## Decisions

Default packaging for every language except Ruby is two packages: pray-core (library) and pray-cli (CLI depending on core). Ruby stays a single gem with require "pray" versus require "pray-cli".

TypeScript ships the split on npm and jsr.io. Host-language ports stay in pray_compat; the main pray repository does not add elixir/ or haskell/ trees.

## Effects

npmjs/pray-core holds the TypeScript library and jsr.json as @kiskolabs/pray-core. npmjs/pray-cli is the CLI with jsr.json as @kiskolabs/pray-cli and depends on pray-core. Release scripts publish npm and JSR core then CLI. CI, dependabot, and package-size checks target the npmjs workspace.

docs/releasing.md states the reference package map and points other-language ports at pray_compat.

## Next

First registry publish of npm pray-core / pray-cli and jsr.io @kiskolabs/pray-core / @kiskolabs/pray-cli on the next version cut.

## Source

docs/issues/20261026143206_core-api-vs-cli-package-split.md
docs/releasing.md
npmjs/pray-core
npmjs/pray-cli
https://github.com/amkisko/pray_compat
