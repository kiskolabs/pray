# TypeScript Deno, Bun, and Workers parse surface

## Participants

Andrei Makarov

## Decisions

Ship a documented `./parse` export for Cloudflare Workers. Keep the root export for Node, Deno, and Bun. Prove runtimes in CI with smoke and curated Deno tests rather than claiming full Node test parity on every runner.

## Effects

Added `pray-core/parse` and jsr `@kiskolabs/pray-core/parse`. Split FS-free parse APIs from IO barrels. Replaced `node:crypto` hashing with an in-repo SHA-256. CI steps: `test:bun`, `test:deno`, `test:workers-parse`.

## Next

Publish a version that includes the parse export. Set JSR runtime badges. Consider Deno coverage for git-hook and transaction child-process tests.

## Source

docs/issues/20261026180500_typescript-runtime-parse-matrix.md
npmjs/pray-core
.github/workflows/ci.yml
