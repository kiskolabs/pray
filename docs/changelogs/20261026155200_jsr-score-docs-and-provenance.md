# JSR score docs and provenance

## Decisions

Ship public API JSDoc and remove `--allow-slow-types` so the next `@kiskolabs/pray-cli` publish can score documentation and no-slow-types. Prefer GitHub Actions OIDC for provenance. Description and runtime marks stay on the jsr.io Settings tab.

## Effects

- Explicit type on `defaultManifestPath` / `defaultProjectRoot` working-directory defaults.
- JSDoc on the library entrypoint and exported symbols.
- `scripts/release/jsr.sh` no longer passes `--allow-slow-types`.
- CI npm job runs `jsr publish --dry-run`.
- `.github/workflows/publish-jsr.yml` publishes with OIDC provenance when dispatched.
- docs/releasing.md and scripts/release/README.md note Settings and provenance.

## Next

Set description and Node.js plus Deno compatibility on jsr.io Settings. Link the GitHub repository if not already linked. Republish (workflow preferred) so the score updates.

## Source

docs/issues/20261026053110_jsr-and-linux-os-packaging.md
https://jsr.io/@kiskolabs/pray-cli/score
