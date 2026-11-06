# Dependabot TypeScript 7 and github-actions landing audit

## Decisions

Land Dependabot #61 (typescript 5.9.3 to 7.0.2) and #62 (github-actions group) through rebased patch PRs #65 and #64 after main rustfmt unblock #63.

Treat TypeScript 7 `compilerOptions.types` default of empty as a required contract change: list `"types": ["node"]` in pray-core and pray-cli tsconfigs.

Keep the upgrade resource compare samples in the TypeScript PR discussion rather than claiming cheaper or smaller without the measured series.

## Effects

Merged to main:

- #63 rustfmt HTTP client construction
- #64 github-actions pins (taiki-e/install-action 2.90.0, sonarqube-scan-action 8.4.1, codeql upload-sarif 4.39.2)
- #65 typescript 7.0.2 plus explicit Node types

Audit fix opened as #66: upgrade notice CLI test no longer lists against the real workspace Prayfile; a temporary empty project root is enough for the notice assertion. Observed local failure was interactive trust consent for https://github.com/amkisko/prayers.git when CI=0.

Upgrade resource compare (local darwin, npmjs worktrees, n=3 for tsc after npm ci --ignore-scripts):

- install size node_modules KiB: 120224 base to 127572 head (delta +7348)
- package-lock.json KiB: 36 to 48
- tsc real seconds samples base 1.35 1.17 1.19; head 0.78 0.30 0.30
- tsc RSS bytes samples base about 313e6 median; head about 173e6 median
- shipped npm package bytes: not changed by this devDependency-only bump
- binary and gem series: not applicable

Quality checks observed this session:

- cargo fmt --all --check: pass on current trees after #63
- cargo clippy workspace: finished without errors before the upgrade test failure surfaced
- cargo test -p pray-cli --test cli_upgrade: pass after fixture fix
- npm ci, npm test, npm run lint in pray-core and pray-cli on the TypeScript 7 branch: pass
- make loc-check: 137 warnings (>=150), 0 failures (>300 without ratchet)

Modes skipped with reason:

- boundary and control: no physical actuators in this change set
- product-surface / privacy / observability / learned-systems: no matching surface change in these dependency bumps
- security mode deep companions: no new hot-path runtime dependency; typescript and github-actions are build and CI pins

## Next

- Merge #66 when CI is green
- Watch first main CI run after #64 for sonarqube-scan-action 8.4.1 behaviour
- Optional: reduce dual-remote push confusion (GitLab push URL before GitHub) so PR head and CI always track the intended forge tip

## Source

- https://github.com/kiskolabs/pray/pull/61
- https://github.com/kiskolabs/pray/pull/62
- https://github.com/kiskolabs/pray/pull/64
- https://github.com/kiskolabs/pray/pull/65
- https://github.com/kiskolabs/pray/pull/66
- TypeScript 7 types default: https://www.typescriptlang.org/tsconfig/types.html
