# pray-cli

TypeScript `pray` CLI for [Prayfile](https://pray.kisko.dev). Depends on [`pray-core`](https://www.npmjs.com/package/pray-core) for resolve, lock, render, verify, and publish.

Library consumers should depend on `pray-core` (or `@kiskolabs/pray-core` on jsr.io), not this package.

**Website:** [pray.kisko.dev](https://pray.kisko.dev)

**Maintainer:** Andrei Makarov ([contact@kiskolabs.com](mailto:contact@kiskolabs.com))

**Repository:** [kiskolabs/pray](https://github.com/kiskolabs/pray)

**Package:** [npmjs.com/package/pray-cli](https://www.npmjs.com/package/pray-cli)

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

## Library vs CLI

```ts
import { parseManifest, resolveProject } from "pray-core";
```

Programmatic CLI dispatch:

```ts
import { runCli } from "pray-cli/cli";
```

## Development

```sh
cd npmjs
npm install
cd pray-cli
npm test
npm run build
node bin/pray.js version
```

## Requirements

- Node.js 20+
- `git` for git sources
- `zstd` and `tar` for `.praypkg` archives (via `pray-core`)
