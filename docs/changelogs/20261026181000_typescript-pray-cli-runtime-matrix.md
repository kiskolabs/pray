# TypeScript pray-cli Deno, Bun, and Workers parse surface

## Participants

Andrei Makarov

## Decisions

Ship the same runtime matrix as pray-core for the CLI package: full CLI on Node, Deno, and Bun; parse re-export for Workers and browsers.

## Effects

`pray-cli/parse` re-exports `pray-core/parse`. Scripts `test:bun`, `test:deno`, and `test:workers-parse` cover the CLI. CI runs those after the pray-core runtime steps.

## Next

Publish a version that includes the parse export. Set JSR runtime badges for `@kiskolabs/pray-cli`.

## Source

docs/issues/20261026181000_typescript-pray-cli-runtime-matrix.md
npmjs/pray-cli
.github/workflows/ci.yml
