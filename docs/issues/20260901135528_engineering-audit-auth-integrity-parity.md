# Engineering audit: authentication, integrity, and parity

## Decisions

This audit covered the Rust workspace and the TypeScript and Ruby CLI implementations. It treated the command line interface, distribution servers, registries, package archives, project materialization, local session state, Git and HTTP transports, and release checks as one product system.

Modes run: core pipeline, boundary and control, resource and budget, trace and identification, product surface, privacy, performance, observability, security, and contracts.

Learned-systems mode was skipped. Pray packages inference inputs but does not execute a learned model, retrieval system, or tool-using agent in these trees.

The worktree already contained active 1.9.2 release changes. The audit preserved those changes and evaluated the final state available during this run.

## Effects

### EA-001: Public authentication endpoints allow account takeover

Severity: critical

Confidence: high

Location: `crates/pray-cli/src/server_http.rs:82`, `crates/pray-cli/src/server_auth.rs:14`, `crates/pray-cli/src/server_auth.rs:50`, `crates/pray-cli/src/server_auth.rs:68`, `crates/pray-cli/src/server_auth.rs:147`, `crates/pray-core/src/auth_store.rs:36`

Kind: observed

Why: registration returns the verification code in the HTTP response. Registering an existing email resets its verification state and code. Session issuance accepts only an email, while passkey and SSH-key enrollment accept an email plus attacker-selected key material without an authenticated session. The passkey and SSH-key endpoints also ignore trust settings that disable those methods by default. A remote caller can therefore register another person's email, read the returned code, verify it, enroll a key, and obtain a session as that identity. This collapses the intended identity boundary before signature verification can help.

Smallest credible fix: stop returning verification codes from the public API, require a verified bearer session for session issuance and key enrollment, reject endpoints for disabled methods, and add negative HTTP and RPC tests for existing-email registration, unauthenticated enrollment, disabled methods, and session issuance without proof.

Deeper fix: model pending registration, verified identity, authenticated session, and enrolled authenticators as an explicit state machine with guarded transitions and separate out-of-band email delivery.

### EA-002: Authentication secrets are deterministic and do not expire

Severity: high

Confidence: high

Location: `crates/pray-core/src/auth_store_support.rs:95`, `crates/pray-core/src/auth_store_support.rs:130`, `crates/pray-core/src/auth_store_support.rs:261`, `crates/pray-core/src/auth_store_support.rs:269`, `crates/pray-core/src/auth_store_tokens.rs:158`, `crates/pray-core/src/auth_store.rs:160`

Kind: observed

Why: challenges, verification codes, session tokens, and publish tokens are hashes of public or guessable values such as email, scope, current Unix second, and process id. Challenge loading and session resolution do not enforce an age limit. Predictable bearer material and replayable challenges weaken every authentication method and make compromise persist until manual revocation.

Smallest credible fix: generate at least 128 bits from the operating system cryptographic random source, store hashes of bearer tokens, enforce short challenge and verification-code lifetimes, enforce session and publish-token expiry, and make challenges single use.

Deeper fix: add rotation, revocation, rate limits, audit events, and tests with a controllable clock and deterministic random test source.

### EA-003: Every client stores live session tokens in an unignored project file

Severity: high

Confidence: high

Location: `crates/pray-cli/src/auth_client.rs:15`, `crates/pray-cli/src/auth_client.rs:32`, `crates/pray-cli/src/auth_client.rs:151`, `npmjs/pray-cli/src/auth/session.ts:19`, `npmjs/pray-cli/src/auth/session.ts:23`, `rubygems/pray-cli/lib/pray/session.rb:15`, `rubygems/pray-cli/lib/pray/session.rb:19`, `.gitignore:1`

Kind: observed

Why: Rust, TypeScript, and Ruby persist server URL, email, live bearer token, authentication kind, and sometimes a signer fingerprint in `.pray/session.json` under the repository. The repository ignores `.pray/cache` but not the session file. The writers use ordinary file creation without an explicit restrictive mode. Tokens and identifying data can enter commits, shared worktrees, backups, or files readable by other local users.

Smallest credible fix: ignore `.pray/session.json` immediately, move sessions to an operating-system credential store or user-scoped Pray data directory, use atomic creation with owner-only permissions for a fallback file, and migrate then remove existing project-local files.

Deeper fix: store only opaque credential references in project state and add logout, revocation, export, deletion, and retention behavior across all clients.

### EA-004: Ruby and TypeScript accept manifest-controlled writes outside the project

Severity: high

Confidence: high

Location: `npmjs/pray-cli/src/render/project.ts:24`, `npmjs/pray-cli/src/render/provisioned.ts:68`, `rubygems/pray-cli/lib/pray/render.rb:27`, `rubygems/pray-cli/lib/pray/render.rb:36`, `crates/pray-core/src/manifest_validate.rs:8`

Kind: observed

Why: TypeScript resolves target, skill, and exact-file destinations against the project root without checking containment. Ruby joins the same manifest-controlled values to the root, and an absolute or parent-relative value can escape it. Rust validates these paths before rendering. Running install or apply in a cloned repository can therefore overwrite files outside that repository in two of the three advertised clients.

Smallest credible fix: apply one project-relative path validator to every output, skill, command, rule, package path, exact file, and local path before planning or writing. Reject absolute paths, parent components, platform prefixes, and normalization escapes. Port Rust's negative tests to the shared corpus and both clients.

### EA-005: TypeScript and Ruby do not fail closed on registry integrity metadata

Severity: high

Confidence: high

Location: `rfcs/0060-distribution.md:132`, `npmjs/pray-cli/src/registry/install.ts:43`, `npmjs/pray-cli/src/registry/index.ts:289`, `rubygems/pray-cli/lib/pray/registry.rb:178`, `rubygems/pray-cli/lib/pray/registry.rb:237`, `crates/pray-core/src/package_integrity.rs:23`

Kind: observed

Why: the distribution contract requires remote installs to fail when either artifact hash or tree hash is absent. TypeScript and Ruby verify each field only when present. Their cache-ready checks compare only package version and do not recompute the selected tree hash. Rust requires both hashes and validates cached trees. A registry or modified cache can therefore supply content without the mandatory integrity binding to two clients.

Smallest credible fix: require non-empty artifact and tree hashes for remote metadata before download or cache reuse, recompute tree hash for cached content, and add shared negative fixtures for absent hashes, mismatches, and mutated caches.

### EA-006: TypeScript sync trusts peer-controlled filesystem paths

Severity: high

Confidence: high

Location: `npmjs/pray-cli/src/sync/index.ts:31`, `npmjs/pray-cli/src/sync/index.ts:40`, `npmjs/pray-cli/src/sync/index.ts:47`, `npmjs/pray-cli/src/sync/index.ts:56`, `npmjs/pray-cli/src/sync/index.ts:60`, `npmjs/pray-cli/src/registry/index.ts:318`

Kind: observed

Why: federation metadata controls package names and artifact paths that TypeScript joins directly to the distribution root. Parent-relative artifact paths and package names can escape the intended metadata, staging, or artifact directories. Local registry artifact loading also treats `file://` as an unrestricted filesystem path. Hashes remain optional in the sync path. A malicious peer or local distribution can direct reads and writes outside the selected root.

Smallest credible fix: validate package names and artifact paths as normalized relative distribution paths, perform containment checks after resolution, reject local `file://` paths outside the root, require both integrity hashes, and stage the whole sync before committing it.

### EA-007: Non-Rust servers lack bounded request handling and useful operational signals

Severity: medium

Confidence: high

Location: `npmjs/pray-cli/src/serve/index.ts:20`, `npmjs/pray-cli/src/serve/index.ts:47`, `npmjs/pray-cli/src/serve/index.ts:77`, `rubygems/pray-cli/lib/pray/serve.rb:17`, `crates/pray-cli/src/server_http.rs:43`

Kind: observed

Why: the TypeScript server buffers PUT bodies without a limit and accepts writes without authentication. The Ruby server has a connection ceiling but no request-body ceiling or socket timeout. Neither exposes readiness, structured request outcomes, or saturation signals. The Rust HTTP bridge assigns every request the same `http` identifier. A small number of oversized or slow requests can exhaust memory or connection capacity, and operators cannot distinguish traffic, latency, rejection, or saturation from process-level failures.

Smallest credible fix: share explicit body, header, connection, and timeout limits across implementations; reject non-loopback writes without authentication; add unique request correlation, readiness, and counters for accepted, rejected, failed, active, and timed-out requests.

### EA-008: Contract parity checks do not exercise the dangerous boundaries

Severity: medium

Confidence: high

Location: `.github/workflows/ci.yml`, `testdata/shared/manifest`, `crates/pray-cli/tests/auth.rs`, `npmjs/pray-cli/src/shared-corpus.test.ts`, `rubygems/pray-cli/spec/pray/shared_corpus_spec.rb`

Kind: observed

Why: the shared corpus contains parser examples but no cross-language cases for output traversal, archive traversal, missing hashes, mutated caches, federation paths, authentication authorization, or resource ceilings. Rust coverage has a 20 percent floor, mutation is advisory, and TypeScript has no coverage gate. The passing implementation suites therefore do not support the repository's cross-client conformance claim at its trust boundaries.

Smallest credible fix: add shared executable fixtures for these negative contracts, require every client to run them, add TypeScript and Ruby coverage measurement for executable logic, raise floors from the measured baseline, and make security-boundary mutation checks blocking.

## Resource and budget

Rust defines point limits for HTTP responses, archives, entries, federation peers, server bodies, headers, concurrent connections, and socket timeouts. Equivalent limits are incomplete in Ruby and TypeScript. No whole-product memory, CPU, disk, network, or energy budget was found. No claim about being smaller, cheaper, or greener is supported by this run.

The ignored Rust scaling suite passed all four guards for render, orphan-marker lookup, resolve, and verify. A confirming production check remains peak RSS, CPU-seconds, disk bytes, and network bytes for a 64 MiB artifact and maximum-peer fixture on a named CI machine.

## Trace and identification

No hidden analytics or telemetry path was found. Explicit confession submission is a user-facing feature. Email addresses, tokens, keys, signer fingerprints, server URLs, and challenge records are stored locally. No complete retention, export, or deletion path was found for authentication records.

## Next

1. Disable or guard the public registration, session, and enrollment endpoints before exposing a distribution server beyond loopback.
2. Replace deterministic authentication material and move live client sessions out of repositories.
3. Port project-path and mandatory-integrity validation to TypeScript and Ruby through shared negative fixtures.
4. Contain all TypeScript federation and local-registry paths and require hashes before sync.
5. Align server resource limits and add minimal operational signals.
6. Restore TypeScript and Ruby lint before release.

## Remediation

On 2026-09-01, the smallest credible fix scope for EA-001 through EA-008 was implemented. The public authentication boundary is guarded, secrets are random and expiring, client sessions moved out of repositories, project and federation paths are contained, remote package integrity fails closed, request and archive budgets are explicit, shared negative fixtures cover path traversal, and measured coverage plus mutation gates are blocking.

The implementation and remaining deeper follow-up work are recorded in `usr/docs/changelogs/20260901143628_engineering-audit-remediation.md`.

## Source

Repository positioning and contracts: `README.md`, `rfcs/0011-prayspec-and-package.md`, `rfcs/0060-distribution.md`, `spec/README.md`.

Validation executed on 2026-09-01:

- `cargo test --workspace`: passed all workspace suites; four ignored scaling tests were not part of this command.
- `cargo fmt --all --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test -p pray-bench -- --ignored --nocapture`: passed all four scaling guards.
- `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=commit.gpgsign GIT_CONFIG_VALUE_0=false npm test`: passed 87 of 87 tests.
- `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=commit.gpgsign GIT_CONFIG_VALUE_0=false RBENV_VERSION=3.4.7 bundle exec polyrun parallel-rspec --workers 5 --merge-failures`: passed all five shards and 134 examples.
- `make loc-check`: passed with 123 warnings and no failures.
- `git diff --check`: passed.
- `npm run lint`: failed with three formatting or import-order errors in the active archive and registry changes; the Biome schema also reports version 2.5.4 while the CLI is 2.5.8.
- `RBENV_VERSION=3.4.7 bundle exec make lint`: failed with two offenses in `lib/pray/registry_install.rb` and `spec/pray/archive_spec.rb`.
