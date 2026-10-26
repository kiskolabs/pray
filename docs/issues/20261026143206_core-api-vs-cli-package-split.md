# Core API vs CLI package split

## Participants

Andrei Makarov

## Decisions

- Default for every language port: publish `pray-core` (library only) and `pray-cli` (CLI / `pray` executable) that depends on `pray-core`. Core must not depend on CLI.
- Ruby is the unique exception. One gem (`pray-cli`) with `require "pray"` for the library and `require "pray-cli"` / `bin/pray` for CLI. Ruby is not a template for other languages.
- TypeScript cuts a real registry split: npm `pray-core` / jsr.io `@kiskolabs/pray-core` for the library; npm `pray-cli` / jsr.io `@kiskolabs/pray-cli` for the CLI, depending on core. Library imports leave the CLI packages.
- Rust already matches the default on crates.io; leave that shape as the model.
- Host-language ports (Elixir, Haskell, and others) live in the separate `pray_compat` repository (`pray.ex`, `pray.hs`, …). They are not trees under the main pray repository. Registry packaging for those ports follows the same two-package rule when they publish; the main repo does not ship Hex or Hackage scaffolds.

## Effects

Inventory: TypeScript library surface is `pray-core` / `@kiskolabs/pray-core` (parse, lock, resolve, render, materialize, verify, publish, serve, sync, vendor, confess). TypeScript CLI is `pray-cli` (`runCli`, `bin/pray`, depends on core). Ruby exception stays one gem with `require "pray"` versus `require "pray-cli"`. Rust already has crates.io `pray-cli` on `pray-core`.

Landed: `npmjs/pray-core` and slim `npmjs/pray-cli`; release scripts publish npm and JSR core then CLI. Root and releasing docs state the package map for the reference surfaces and the Ruby exception. Mistaken `elixir/` and `haskell/` scaffolds were removed from this repository; compat stays in `pray_compat`.

## Next

- First registry publish of npm `pray-core` / `pray-cli` and jsr.io `@kiskolabs/pray-core` / `@kiskolabs/pray-cli` after the next version cut.
- Do not add Hex or Hackage publish to this repository's `make release-all`.

## Source

- docs/releasing.md
- crates.io `pray-core` / `pray-cli`
- npmjs/pray-core, npmjs/pray-cli
- rubygems/pray-cli
- https://github.com/amkisko/pray_compat (host-language ports)
