# pray-core

TypeScript library for [Prayfile](https://pray.kisko.dev): a package manager for the language placed before inference.

Resolve, lock, render, verify, publish, and serve shared instructions, policies, templates, and related input as versioned packages. Consumes path, git, and registry distribution points.

The CLI executable is the separate [`pray-cli`](https://www.npmjs.com/package/pray-cli) package, which depends on this library. Import from npm `pray-core` or jsr.io [`@kiskolabs/pray-core`](https://jsr.io/@kiskolabs/pray-core).

**Website:** [pray.kisko.dev](https://pray.kisko.dev)

**Maintainer:** Andrei Makarov ([contact@kiskolabs.com](mailto:contact@kiskolabs.com))

**Repository:** [kiskolabs/pray](https://github.com/kiskolabs/pray)

**Packages:** [npmjs.com/package/pray-core](https://www.npmjs.com/package/pray-core) · [jsr.io/@kiskolabs/pray-core](https://jsr.io/@kiskolabs/pray-core)

**Community docs:** [CHANGELOG.md](CHANGELOG.md) · [LICENSE.md](LICENSE.md) · [SECURITY.md](SECURITY.md)

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

## Usage

```ts
import { parseManifest, resolveProject } from "pray-core";

const manifest = parseManifest(`prayfile 1\n`);
const project = await resolveProject("./Prayfile");
```

For the `pray` executable and CLI dispatch, install [`pray-cli`](https://www.npmjs.com/package/pray-cli).

### Cloudflare Workers and browsers (parse only)

Workers and browsers cannot run resolve, render, git materialize, or serve. Import the parse entry:

```ts
import { parseManifest, parseLockfile, manifestHash } from "pray-core/parse";
// jsr: @kiskolabs/pray-core/parse
```

## Development

```sh
cd npmjs
npm install
cd pray-core
npm test
npm run build
npm run test:bun
npm run test:deno
npm run test:workers-parse
```

## Requirements

- Full library (root export): Node.js 20+, Deno, or Bun
- `git` for git sources (full library)
- `zstd` and `tar` for `.praypkg` archives (full library)
- Cloudflare Workers and browsers: `@kiskolabs/pray-core/parse` / `pray-core/parse` only
