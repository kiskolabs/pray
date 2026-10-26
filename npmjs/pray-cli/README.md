# pray-cli

TypeScript `pray` CLI for [Prayfile](https://pray.kisko.dev). Depends on [`pray-core`](https://www.npmjs.com/package/pray-core) for resolve, lock, render, verify, and publish.

Library consumers should prefer `pray-core` (or `@kiskolabs/pray-core` on jsr.io). This package exposes the CLI and a parse re-export for runtime parity.

**Website:** [pray.kisko.dev](https://pray.kisko.dev)

**Maintainer:** Andrei Makarov ([contact@kiskolabs.com](mailto:contact@kiskolabs.com))

**Repository:** [kiskolabs/pray](https://github.com/kiskolabs/pray)

**Packages:** [npmjs.com/package/pray-cli](https://www.npmjs.com/package/pray-cli) · [jsr.io/@kiskolabs/pray-cli](https://jsr.io/@kiskolabs/pray-cli)

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

Programmatic CLI dispatch (Node, Deno, or Bun):

```ts
import { runCli } from "pray-cli/cli";
```

### Cloudflare Workers and browsers (parse only)

`runCli` needs a local filesystem, `git`, and archive tools. Workers and browsers use the parse entry (same surface as `pray-core/parse`):

```ts
import { parseManifest, parseLockfile } from "pray-cli/parse";
// jsr: @kiskolabs/pray-cli/parse
```

Prefer `pray-core/parse` when you only need the library.

## Development

```sh
cd npmjs
npm install
cd pray-cli
npm test
npm run build
node bin/pray.js version
npm run test:bun
npm run test:deno
npm run test:workers-parse
```

## Requirements

- Full CLI (`pray` / `runCli`): Node.js 20+, Deno, or Bun
- `git` for git sources
- `zstd` and `tar` for `.praypkg` archives (via `pray-core`)
- Cloudflare Workers and browsers: `pray-cli/parse` only
