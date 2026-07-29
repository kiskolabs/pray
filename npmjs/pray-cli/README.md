# pray-cli

TypeScript `pray` CLI and library for [Prayfile](https://pray.kisko.dev): a package manager for the language placed before inference.

Resolve, lock, render, verify, publish, and serve shared instructions, policies, templates, and related input as versioned packages. The installed executable is `pray`.

**Website:** [pray.kisko.dev](https://pray.kisko.dev)

**Maintainer:** Andrei Makarov ([contact@kiskolabs.com](mailto:contact@kiskolabs.com))

**Repository:** [kiskolabs/pray](https://github.com/kiskolabs/pray)

**Community docs:** [CHANGELOG.md](CHANGELOG.md) · [LICENSE.md](LICENSE.md) · [SECURITY.md](SECURITY.md)

## Install

```sh
npm install -g pray-cli
```

Or run without installing:

```sh
npx pray-cli install
```

## Usage

From a project with a `Prayfile`:

```sh
pray manifest
pray install
pray verify --strict
pray drift
pray publish --root ./prayers
pray serve --root ./prayers --port 7429
```

## Commands

- `manifest`, `init`, `prayer init`, `repo init`
- `add`, `remove`, `update`, `unlock`
- `install`, `apply`, `plan`, `render`, `verify`, `drift`, `format`
- `package`, `publish`, `serve`, `sync`, `vendor`, `clean`, `tree`
- `list`, `outdated`, `explain`
- `trust`, `confess`, `login` (login: not yet implemented)

## Development

```sh
cd npmjs/pray-cli
npm install
npm test
npm run build
node bin/pray.js version
```

## Requirements

- Node.js 20+
- `git` for git sources
- `zstd` and `tar` for `.praypkg` archives

## Status

Registry HTTP(S), git sources, local publish/serve/sync, vendor/tree, and trust policy editing are implemented. SSH registry (`pray+ssh://`), `serve --stdio`, and `login` remain planned.

See `SPEC.md` in the repository root for the normative specification.
