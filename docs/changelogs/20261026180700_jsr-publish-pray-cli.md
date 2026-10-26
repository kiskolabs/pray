# JSR publish for @kiskolabs/pray-cli

## Participants

Andrei Makarov

## Decisions

jsr.io publishes both TypeScript packages: @kiskolabs/pray-core (library) then @kiskolabs/pray-cli (CLI). Global pray install stays on npm; JSR pray-cli exposes runCli for Deno and other JSR consumers.

## Effects

npmjs/pray-cli/jsr.json returns. scripts/release/jsr.sh and publish-jsr.yml publish core then CLI with per-package skip. CI dry-runs both packages. Version sync covers both jsr.json files.

## Next

Run scripts/release/jsr.sh --publish (or resume make release-all) so @kiskolabs/pray-cli@1.24.0 lands. Set description and Node.js plus Deno compatibility on the jsr.io Settings tab for pray-cli.

## Source

docs/issues/20261026143206_core-api-vs-cli-package-split.md
scripts/release/jsr.sh
