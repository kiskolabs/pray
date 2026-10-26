# pray-core

TypeScript library for [Prayfile](https://pray.kisko.dev): resolve, lock, render, verify, publish, and serve shared inference input packages.

The CLI executable is the separate [`pray-cli`](https://www.npmjs.com/package/pray-cli) package, which depends on this library. Import the library from npm `pray-core` or jsr.io `@kiskolabs/pray-core`.

**Website:** [pray.kisko.dev](https://pray.kisko.dev)

**Maintainer:** Andrei Makarov ([contact@kiskolabs.com](mailto:contact@kiskolabs.com))

**Repository:** [kiskolabs/pray](https://github.com/kiskolabs/pray)

**Packages:** [npmjs.com/package/pray-core](https://www.npmjs.com/package/pray-core) · [jsr.io/@kiskolabs/pray-core](https://jsr.io/@kiskolabs/pray-core)

## Install

```sh
npm install pray-core
```

```sh
npx jsr add @kiskolabs/pray-core
```

```ts
import { parseManifest, resolveProject } from "pray-core";
```

## Development

```sh
cd npmjs
npm install
cd pray-core
npm test
npm run build
```

## Requirements

- Node.js 20+
- `git` for git sources
- `zstd` and `tar` for `.praypkg` archives
