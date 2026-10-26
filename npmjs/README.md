# npm packages

TypeScript packages for the pray project.

## Packaging rule

- `pray-core` — library only (npm + jsr.io `@kiskolabs/pray-core`)
- `pray-cli` — CLI / `pray` executable; depends on `pray-core`
- Core must not depend on CLI

## Workspace

```bash
cd npmjs
npm install
npm run test --workspace=pray-core
npm run test --workspace=pray-cli
```

Publish order: `pray-core`, then `pray-cli` (`scripts/release/npm.sh`). JSR publishes `@kiskolabs/pray-core` only (`scripts/release/jsr.sh`).
