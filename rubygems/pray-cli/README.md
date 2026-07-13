# pray-cli

Ruby client for the `pray` CLI and [Prayfile](https://pray.kisko.dev) workflow.

Resolves `Prayfile` dependencies, writes `Prayfile.lock`, renders managed guidance into target files, and verifies drift. Consumes local path packages and git distribution repositories.

**Website:** [pray.kisko.dev](https://pray.kisko.dev)

**Maintainer:** Andrei Makarov ([contact@kiskolabs.com](mailto:contact@kiskolabs.com))

**Repository:** [kiskolabs/pray](https://github.com/kiskolabs/pray)

**Community docs:** [CHANGELOG.md](CHANGELOG.md) · [LICENSE.md](LICENSE.md) · [SECURITY.md](SECURITY.md)

## Install

```bash
gem install pray-cli
```

From this repository:

```bash
cd rubygems/pray-cli
bundle install
bundle exec pray version
```

The executable is named `pray`.

## Commands

Core workflow:

```bash
pray init
pray install
pray plan
pray verify
pray drift
pray manifest
```

Package management:

```bash
pray add sample/base "~> 1.0" --path packages/base
pray remove sample/base
pray update
pray unlock sample/base
pray list
pray tree
pray explain sample/base
pray outdated
```

Package authoring:

```bash
pray prayer init
pray package
```

Maintenance:

```bash
pray render
pray format
pray clean
```

Deferred in this release (clear unsupported errors): `login`, `confess`, `sync`, `trust`, `pray_ssh` sources, `serve --stdio`.

Git sources clone into `.pray/cache/git`, discover distribution roots at `v1/packages` or `prayers/v1/packages`, and resolve packages through registry metadata.

Local publish writes `v1/index.json`, package metadata, and `.praypkg` artifacts. `serve` exposes the distribution tree over HTTP.

## Layout

```
lib/pray/
  literal.rb       Prayfile and prayspec literal parser
  manifest.rb      Prayfile parser and model
  package_spec.rb  prayspec parser
  lockfile.rb      Prayfile.lock read/write
  resolve.rb       dependency resolution
  render.rb        target rendering and skill materialization
  verify.rb        verify and drift checks
  registry.rb      HTTP and local registry resolution
  archive.rb       .praypkg archive read/write (requires tar and zstd)
  cli.rb           command dispatch
```

## Tests

```bash
bundle exec rspec
```

From the repository root:

```bash
make ruby-test
```

## Compatibility

Manifest hashing, lockfile fields, pray markers, and tree hashes match the Rust reference for path-based projects. Registry resolution uses HTTP or a local distribution tree with `v1/packages/*.json` metadata.
