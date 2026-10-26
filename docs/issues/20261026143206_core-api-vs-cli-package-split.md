# Core API vs CLI package split

## Participants

Andrei Makarov

## Decisions

- The problem is real: a core consumer may need parse, resolve, render, or verify without the CLI binary or CLI entrypoints.
- When packages are split, the CLI package depends on core. Core must not depend on CLI. Same direction as crates.io (`pray-cli` → `pray-core`).
- Do not cut separate registry packages only to mirror Rust names. First separate the contract inside each language tree: stable core API versus CLI. Document which exports are library surface.
- After that contract is stable, publish core where bin or CLI coupling hurts a consumer: most likely a Ruby gem (`pray-core` or equivalent) and `@kiskolabs/pray-core` on jsr.io; npm CLI stays `pray-cli` and depends on the core package (npm and/or JSR) rather than bundling core as the only install unit.
- Rust already publishes `pray-core` on crates.io with `pray-cli` depending on it; leave that shape as the model.

## Effects

### Inventory (current trees)

TypeScript library surface (`npmjs/pray-cli` package root / jsr.io `@kiskolabs/pray-cli`):
parse and edit manifest; lockfile read/write; resolve; render; materialize; verify and drift; publish to root or server; serve; sync; vendor; confess; errors and path helpers.

TypeScript CLI-only:
`runCli` (argv dispatch). Trust argv helper `runTrustCommand` stays under `src/trust` for the CLI module and is not re-exported from the library root. The `pray` binary loads `dist/cli/main.js`. Programmatic CLI import is `pray-cli/cli`.

Ruby library surface (`require "pray"`):
`Pray` module methods listed in `lib/pray.rb` (parse, resolve, render, materialize, verify, drift, lockfile helpers). Does not define `Pray::CLI`.

Ruby CLI (`require "pray-cli"` or `bin/pray`):
loads library plus `Pray::CLI`.

### Done this pass

- TypeScript: removed `runCli` and `runTrustCommand` from `src/index.ts`; added package export `./cli`; library test asserts CLI symbols stay off the root; READMEs document library versus CLI.
- Ruby: entry comments on `lib/pray.rb` and `lib/pray-cli.rb`; library specs cover `require "pray"` without CLI, `require "pray-cli"` with CLI, and core workflow methods; README documents the split.
- Registry packages are still single gems/npm packages that ship a `pray` executable. No new `pray-core` registry cut yet.

## Next

- Do not cut a new registry package until an external embed consumer needs library-without-CLI install semantics, or gem/npm bin install is clearly harmful.
- When cutting: publish core first, then make the CLI package declare a dependency on that core version; prefer gem and JSR for the library name, npm `pray-cli` as the CLI install that depends on core.
- Optional later: move `materialize` out of the TypeScript `cli/` folder name so path names match the library contract (behavior already library-facing).

## Source

- docs/releasing.md
- docs/issues/20261026053110_jsr-and-linux-os-packaging.md
- crates.io `pray-core` / `pray-cli`
- npm `pray-cli`, jsr.io `@kiskolabs/pray-cli`, RubyGems `pray-cli`
- npmjs/pray-cli/src/index.ts
- npmjs/pray-cli/src/library.test.ts
- rubygems/pray-cli/lib/pray.rb
- rubygems/pray-cli/lib/pray-cli.rb
- rubygems/pray-cli/spec/pray/library_spec.rb
