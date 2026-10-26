# TypeScript pray-cli runtime matrix

## Participants

Andrei Makarov

## Decisions

Match pray-core runtime claims for TypeScript pray-cli: Deno and Bun run the full CLI (`runCli` / `pray`). Cloudflare Workers and browsers use only `pray-cli/parse`, which re-exports `pray-core/parse`. Prefer `pray-core` / `pray-core/parse` for library-only consumers.

## Effects

Added `./parse` on npm and jsr.io. CI runs Bun and Deno CLI smokes plus a browser-platform Workers parse smoke for pray-cli.

## Next

After publish, set JSR `@kiskolabs/pray-cli` runtime compatibility: Node, Deno, Bun Supported; Workers and browsers Supported with the parse-subpath caveat.

## Source

docs/issues/20261026180500_typescript-runtime-parse-matrix.md
npmjs/pray-cli
.github/workflows/ci.yml
