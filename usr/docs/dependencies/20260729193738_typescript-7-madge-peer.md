# TypeScript 7 blocked by madge peer and toolchain break

## Dependency

- typescript: Dependabot proposed ^7.0.2 (from ^5.8.3)
- @types/node: Dependabot proposed ^26.1.2 (from ^22.15.0)
- madge: ^8.0.0 (unchanged), peerOptional typescript@^5.4.4
- Package: npmjs/pray-cli
- Lockfile: npmjs/pray-cli/package-lock.json

## Symptom

npm install fails with ERESOLVE (madge peerOptional typescript@^5.4.4 vs typescript@7). With --legacy-peer-deps, tsc fails resolving node: builtins and madge crashes in ts-api-utils (TypeFlags.Intrinsic undefined).

## Evidence

- PR https://github.com/kiskolabs/pray/pull/9 CI npm job ERESOLVE
- Local: npm install fails on typescript@7.0.2 with madge@8.0.0
- Local: npm install --legacy-peer-deps then npm test / madge fail as above

## Suggested fix

Keep typescript on ^5.9.3 and @types/node on ^22 until madge (and precinct/typescript-eslint chain) support TypeScript 7. Revisit majors in a dedicated bump, not a Dependabot group with biome/smol-toml patches.

## Next

- Merged PR #9 with safe pins (typescript ^5.9.3, @types/node ^22.20.1) plus biome 2.5.6 and smol-toml 1.7.1
- Watch for madge release that widens typescript peer range

## Source

- Upstream PR: https://github.com/kiskolabs/pray/pull/9
- madge peer: typescript@^5.4.4 on madge@8.0.0
