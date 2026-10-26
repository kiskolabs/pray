# TypeScript runtime matrix for Deno, Bun, and Workers parse

## Participants

Andrei Makarov

## Decisions

Deno and Bun support the full TypeScript library entry (same Node contracts for resolve, render, git, archives, and serve). Cloudflare Workers and browsers support only the parse and validate subpath `pray-core/parse` / `@kiskolabs/pray-core/parse`.

JSR runtime badges: Node, Deno, and Bun Supported for the package; Workers and browsers Supported with the documented parse-subpath boundary.

## Effects

`src/parse.ts` is a Workers-safe barrel. Hashing no longer loads `node:crypto`. Absolute path checks no longer load `node:path`. Publish remote validation and implied source resolution sit on FS-free modules so `parseManifest` does not pull git or HTTP graphs.

CI runs Bun runtime smoke, Deno selected node:test files plus runtime smoke, and a browser-platform esbuild Workers parse smoke.

## Next

After the next publish, set JSR package Settings runtime compatibility to match the Decision (including browsers on the parse subpath). Optional later work: expand Deno coverage past the curated suite (git hook env and Node child-process transaction tests still fail under Deno). TypeScript pray-cli follows the same matrix; see docs/issues/20261026181000_typescript-pray-cli-runtime-matrix.md.

## Source

npmjs/pray-core/src/parse.ts
npmjs/pray-core/scripts/runtime-smoke.mjs
npmjs/pray-core/scripts/workers-parse-smoke.mjs
.github/workflows/ci.yml
