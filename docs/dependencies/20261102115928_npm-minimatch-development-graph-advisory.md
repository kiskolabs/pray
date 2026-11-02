# npm minimatch development graph advisory

## Participants

Andrei Makarov (@amkisko)

## Dependency

`minimatch` 10.2.6 is a transitive development dependency under `madge` 8.0.0 through `dependency-tree`, `precinct`, `detective-typescript`, and `@typescript-eslint/typescript-estree`. Both TypeScript packages use `madge` only for the circular-dependency lint command.

## Symptom

The 1.26.1 release dry run reported seven moderate npm audit entries. They are one advisory propagated through the development dependency path. The production-only audit reported no vulnerabilities.

## Evidence

### Trigger

`npm audit` matched GHSA-g3hv-75wg-m249, also published as CVE-2026-118764. The advisory covers `minimatch` before 11.0.1 and requires an attacker-controlled glob pattern with optimization level 2 or greater.

### Assessment target

- npm and JSR 1.26.1 release artifacts;
- the repository circular-dependency lint commands for `pray-core` and `pray-cli`.

### Advisory

GHSA-g3hv-75wg-m249: glob-part amplification can cause excessive CPU and memory consumption.

### Status

`not_affected` for both named targets.

### Applicability

The release artifacts do not contain `madge` or its transitive development graph. `npm audit --omit=dev --json` reported zero vulnerabilities across the production graph.

The lint commands invoke `madge` with explicit entry files, extensions, and a TypeScript config. They do not provide `detectiveOptions` or `projectService.allowDefaultProject`. In the locked graph, `minimatch` is called by `@typescript-eslint/typescript-estree` only when `allowDefaultProject` patterns are present. The vulnerable code is therefore not in the execute path for the named lint commands.

This assessment does not cover a future madge or parser configuration that accepts untrusted `allowDefaultProject` glob patterns.

### Priority

Low for the named release and automation targets. The scanner finding remains visible until the development graph receives a compatible patched version.

### Disposition

Keep madge 8.0.0 for this release. Do not apply npm's suggested major downgrade to madge 7.0.0. Reassess when madge or its parser graph adopts a patched minimatch, or before passing externally controlled parser glob patterns.

## Suggested fix

Upgrade the development graph when a compatible madge release resolves the advisory. Avoid an override across minimatch's major-version constraint without upstream compatibility evidence.

## Next

- Monitor madge and `@typescript-eslint/typescript-estree` for a compatible patched graph.
- Re-run the complete and production-only npm audits before publishing a later release.
- Reopen the applicability assessment if lint configuration begins supplying `allowDefaultProject` patterns.

## Source

- [GitHub advisory GHSA-g3hv-75wg-m249](https://github.com/advisories/GHSA-g3hv-75wg-m249)
- `npmjs/package-lock.json`
- `npmjs/pray-core/package.json`
- `npmjs/pray-cli/package.json`
- `scripts/release/npm.sh`
