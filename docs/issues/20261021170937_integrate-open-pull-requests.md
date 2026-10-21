# Integrate the October 2026 open pull requests

## Participants

- Vesa Vänskä
- Andrei Makarov

## Decisions

- Preserve every open pull request in one integration history on `trunk/2026w43`; resolve overlapping Cargo and npm lockfiles to the newest compatible versions.
- Keep `smol-toml` 1.9.1 from PR 47 instead of the overlapping 1.9.0 update in PRs 37 and 44.
- Keep the Git repository isolation contract consistent across the Rust, Ruby, and TypeScript CLIs. Repository-selecting environment variables are removed, caller Git configuration still applies, and a cache must resolve to its own Git directory before its remote can change.
- Pin Rust 1.91.0 as the tested minimum and make `render --check` use the same layout contract as plan and install.
- Raise the CI mutation-smoke timeout from 30 to 60 seconds. The 30-second limit stopped the unmutated baseline after the integrated test suite grew; the repository Makefile already used 60 seconds.

## Findings

- Engineering audit, low severity, high confidence: `crates/pray-core/tests/git_source_host_environment.rs` changed process-global Git environment variables in four tests without synchronization. Parallel execution could mix their state. A process-local mutex now guards each test, and 20 parallel repetitions passed.
- Engineering audit, medium severity, high confidence: the CI mutation smoke could not complete its unmutated baseline within 30 seconds. The timeout now matches the 60-second project command. The finding was observed before the change and the corrected command was run afterwards.
- Engineering audit, medium severity, high confidence: the hosted Rust suite exposed a port-allocation race in the peer-discovery integration test. It released two provisional ports before starting either server, so both servers could receive the same port on Linux. The test now starts the seed server before allocating the upstream port and reports the actual sync summary on failure.
- Prose audit, low severity, high confidence: the root changelog repeated the cross-language Git fix and described implementation history at greater length than the user outcome required. The entries now state the protected behavior, retained Git configuration contract, cache refusal, and render-check result once.
- Tooling audit, low severity, high confidence: `npmjs/pray-cli/biome.json` named schema 2.5.12 while the lockfile selected Biome 2.5.15. The schema now matches the installed CLI.
- Security and privacy review found no unresolved issue in the integrated changes. Git credentials, proxy configuration, and URL rewrites remain inherited but are not added to output. The changes add no analytics, persistent person identifier, browser storage, unsafe Rust, or native interface.

## Effects

- PRs 37, 41, 42, 43, 44, 45, 46, 47, 48, 49, and 50 are ancestors of the integration branch.
- Focused regression suites passed in Rust, Ruby, and TypeScript. Full Rust, npm, and Ruby suites passed.
- Rust line coverage measured 75.37 percent against the 65 percent floor. npm line coverage measured 75.57 percent against the 60 percent floor.
- All nine ignored scaling tests passed. The largest reported peak resident memory was 11,698,176 bytes in the index parsing case; the measured growth ratios stayed within their near-linear test budgets.
- Cargo advisories, licenses, bans, and sources passed. npm production and full audits reported no vulnerabilities. Bundler Audit reported no vulnerabilities. Cargo Machete reported no unused dependencies.
- Release changelog, coordinated-version, crate-status, RFC identifier, and drift checks passed.
- The CLI is the changed person-facing surface. Error copy identifies a damaged cache and the recovery action. No interactive, loading, or graphical state changed; narrow terminal wrapping remains the terminal's responsibility. A person can complete the operation after removing the named cache and rerunning the command.

## Next

- Let hosted CI verify the integration branch on Linux and with repository secrets that are unavailable locally.
- Obtain the required review before merging the integration branch into `main`.

## Source

- [PR 37](https://github.com/kiskolabs/pray/pull/37), [PR 41](https://github.com/kiskolabs/pray/pull/41), [PR 42](https://github.com/kiskolabs/pray/pull/42), [PR 43](https://github.com/kiskolabs/pray/pull/43), [PR 44](https://github.com/kiskolabs/pray/pull/44), [PR 45](https://github.com/kiskolabs/pray/pull/45), [PR 46](https://github.com/kiskolabs/pray/pull/46), [PR 47](https://github.com/kiskolabs/pray/pull/47), [PR 48](https://github.com/kiskolabs/pray/pull/48), [PR 49](https://github.com/kiskolabs/pray/pull/49), and [PR 50](https://github.com/kiskolabs/pray/pull/50)
- [Dependency assessment](../dependencies/20261021170938_october-2026-security-advisories.md)
