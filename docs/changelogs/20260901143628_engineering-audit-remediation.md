# Engineering audit remediation

## Decisions

Public registration responses no longer contain verification secrets. Email-only session issuance and unauthenticated authenticator enrollment are denied. Passkey and SSH-key routes honor the configured trust policy.

Authentication challenges, verification codes, sessions, and publish tokens use operating-system randomness. Bearer tokens are stored as hashes and every credential type has an expiry.

Client sessions live under the user Pray home, use owner-only fallback files, and migrate then remove legacy repository session files.

Rust, TypeScript, and Ruby reject project paths that escape the selected root. TypeScript and Ruby require both remote integrity hashes and recompute cached tree hashes. TypeScript federation validates peer paths and metadata before staging content.

TypeScript and Ruby servers enforce body, header, connection, and timeout ceilings. Archive readers reject duplicate paths and enforce entry, file, compressed, and expanded-size limits before extraction.

Shared negative manifest fixtures now run in all three implementations. Rust and TypeScript coverage floors follow measured baselines, Ruby retains its existing measured floor, and the Rust mutation smoke is blocking.

The Rust authentication store uses `getrandom` 0.4.3 directly for operating-system entropy. The locked HTTP graph moves from h2 0.4.15 to 0.4.16 to resolve RUSTSEC-2026-0258.

## Effects

The remediation closes engineering audit findings EA-001 through EA-008 at their stated smallest credible fix scope. Authentication delivery, rotation, rate limiting, revocation user experience, whole-product resource budgets, and richer operational counters remain deeper follow-up work.

An unisolated Rust test created a user-home session file containing only localhost fixture identities. The test helpers now set a repository-local test Pray home, and the verified test-only file was removed.

`cargo test --workspace` passed the full Rust workspace suite; four opt-in scaling guards remained ignored as declared. `cargo llvm-cov --workspace --summary-only --fail-under-lines 65` passed with 70.51 percent line coverage.

`npm run test:coverage` passed 96 tests with 67.11 percent line coverage against the 60 percent floor. `POLYRUN_COVERAGE=1 rbenv exec bundle exec polyrun parallel-rspec --workers 5 --merge-failures` passed all five Ruby shards and 139 examples. `rbenv exec bundle exec polyrun report-coverage -i coverage/merged.json --format console` reported 83.41 percent line coverage across 52 files.

`cargo mutants -p pray-core --timeout 30 --jobs 2 -f 'manifest_validate.rs' -f 'package_archive.rs' -F '(manifest_validate.rs.*replace != with ==|package_archive.rs:12:36: replace > with ==)'` tested four boundary mutations and caught all four.

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `npm run lint`, `rbenv exec bundle exec rubocop`, `rbenv exec bundle exec rbs -I sig validate`, `git diff --check`, and `make loc-check` passed. The file-length check reported 125 warnings and no failures.

`cargo deny check advisories licenses bans sources`, `npm audit --omit=dev`, `rbenv exec bundle exec bundler-audit check --update`, and `cargo machete` passed. Cargo deny retained only the repository's existing duplicate-version and allowed-license warnings; the JavaScript and Ruby advisory scans found no vulnerabilities, and cargo machete found no unused dependencies.

## Next

Specify and test verification delivery, credential rotation, authentication rate limits, revocation user experience, whole-product resource budgets, and richer operational counters before expanding those surfaces.

## Source

Audit: `usr/docs/issues/20260901135528_engineering-audit-auth-integrity-parity.md`.

Dependency evidence: `usr/docs/dependencies/20260901143628_h2-empty-data-frame-advisory.md`.
