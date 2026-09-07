# Legacy lock update: destination recovery

## Participants

Andrei Makarov

## Decisions

Keep the destination ownership rules in RFC 0033. Existing bytes may be adopted when they equal the requested export, or replaced when they still match the previous lock's content hash. An empty provisioned array does not establish ownership. Force and destination overwrite prompts remain outside this change.

The implementation pass on 2026-09-07 narrows the earlier proposal: fix candidate resolution, write ordering, previews, and recovery first. Automatic adoption across a legacy version transition remains a separate contract question. Keep the implementation on the current branch. Maintain this issue and CHANGELOG.md as the documentation for this work.

## Effects

### Confirmed cause and recovery

The destination guard and provisioned ledger shipped in 1.10.0. The 1.9.1 and 1.9.2 writers lacked those ownership records. A legacy lock can parse successfully while providing no evidence that an existing destination belongs to a package.

When an update resolves new export bytes, unchanged old-version bytes differ from the candidate and have no recorded hash authorizing replacement. This correctly takes the unmanaged collision path. It does not establish that the operator edited the file. The synthetic regression fixture removes the provisioned array from a current lock; it exercises the missing-evidence transition without claiming to reproduce every historical lock field or another project's incident.

Install the original package version under its original Prayfile first. Identical destination bytes can then be recorded in the lock, after which update can replace them. For edited or independently authored files, inspect and move them aside before installing. Keep the moved files until their changes have been recovered.

Previously, Rust and TypeScript latest updates saved new constraints before destination checks. A refused update could therefore break the install-first recovery. Rust's normal update preview already rejected deterministic collisions before compose writes; the earlier claim that this specific failure necessarily changed compose output was incorrect. Other callers of the shared writer, and later filesystem failures, had different write boundaries.

### Implemented behavior

Rust and TypeScript now resolve the candidate Prayfile in memory. Latest update saves changed constraints only after destination writes succeed, then saves the resulting lock. A predictable destination collision preserves the original Prayfile, lock, and compose output.

Latest JSON updates follow the same materialization path even when no constraint changes are needed. A newer version already permitted by the existing constraint now installs and updates the lock instead of returning success early.

Latest dry-run resolves the candidate versions and checks their destinations. It reports the same predictable destination collisions without writing Prayfile, lock, or output files. Resolution may still refresh source caches. Rust and TypeScript update help describe the preview and recovery paths.

All three implementations collect destination render errors across planned leaves, including package and export context, before the shared writer changes compose output. Plan uses the same validation. Non-render errors can still stop collection immediately. Mutation-time path and ownership checks remain in place.

Verify now distinguishes a destination that still matches its recorded previous hash from one with unrecognized changes. The former can be restored with install. The latter receives inspect-and-move guidance before install, preserving the operator's edits. Verification can still fail earlier during source resolution; destination guidance is conditional on reaching that stage.

Ruby now rejects unsupported update options before writing: --latest and --major explain manual Prayfile constraint editing; --dry-run and --json report that those update options are unavailable. The latter points to plan for a preview of the current Prayfile. Ruby help lists the supported update form. Automatic latest constraint changes remain available only in Rust and TypeScript.

### Validation

Added regressions before implementation. Rust and TypeScript each first failed all four new cases, then passed: omitted-ledger collision and install-first recovery, candidate dry-run collision, JSON update within an existing constraint, and verify recovery that retains edited bytes. Ruby first failed the three new cases, then passed after destination validation, recovery, and flag handling changes. Extending the flag case to dry-run also failed before its rejection was implemented.

From the repository root:

- cargo test --offline -p pray-cli --test update_destinations: passed, 4 tests.
- cargo test --offline: passed, full workspace including documentation tests.
- cargo clippy --offline: passed.
- cargo fmt --check: passed.
- make loc-check: passed with 134 size warnings and 0 failures; existing size ratchets were unchanged.
- git diff --check: passed.

From npmjs/pray-cli:

- ./node_modules/.bin/tsc -p tsconfig.test.json && node --test dist/update-destinations.test.js: passed, 4 tests.
- npm test: passed, 147 tests.
- npm run lint: passed, including cycle detection.
- npm run typecheck: passed.

From rubygems/pray-cli, using Ruby 4.0.0 and Bundler 4.0.15 with an isolated GEM_PATH supplement for the already locked toml-rb 4.2.1 gem:

- bundle exec rspec spec/pray/update_destinations_spec.rb: passed, 3 examples before extending unsupported-option coverage; the extended case subsequently passed in the full suite.
- bundle exec rspec: passed, 192 examples.
- bundle exec rubocop --cache false: passed, 132 files and no offenses.
- bundle exec rbs -I sig validate: passed.

The first full Rust and Ruby runs failed because the sandbox denied localhost listeners used by HTTP fixtures. Both suites passed when rerun with localhost access. Ruby's locked gem was installed from an existing local cache into temporary storage; package manifests and lockfiles were unchanged. No coverage percentage or performance measurement is claimed.

## Next

Automatic legacy adoption needs an explicit rule for authenticating old package bytes and reconstructing historical destination mappings, selected exports, symbols, and environment. A cached package hash plus the current Prayfile cannot prove all of those inputs. Preserve refusal when the evidence is missing.

Full rollback across compose files, exclusive destinations, Prayfile, and lock remains outside this fix. A later I/O failure or concurrent filesystem change can still leave partially written outputs. Preflight catches predictable destination conflicts; it does not provide a crash-safe transaction.

Validation repeats file reads and previous-lock lookups at preview and write time. Existing destination reads have no new size ceiling, and aggregated diagnostics grow with the number of collisions. Large-tree and oversized-file performance remains unmeasured.

## Source

- rfcs/0033-provisioned-destination-safety.md; rfcs/0031-ownership-and-generated-output.md; rfcs/0040-cli-surface.md.
- CHANGELOG.md, versions 1.9.1 and 1.10.0 and the current Unreleased entries.
- Historical writers: git show v1.9.1:crates/pray-core/src/render_provisioned.rs and git show v1.9.2:crates/pray-core/src/render_provisioned.rs.
- crates/pray-cli/src/commands_update.rs; crates/pray-cli/src/commands_update_latest.rs; crates/pray-core/src/render_dest.rs; crates/pray-core/src/render_write.rs; crates/pray-core/src/verify/provisioned.rs.
- npmjs/pray-cli/src/cli/commands/update-core.ts; npmjs/pray-cli/src/cli/commands/update-latest.ts; npmjs/pray-cli/src/render/dest.ts; npmjs/pray-cli/src/verify/provisioned.ts.
- rubygems/pray-cli/lib/pray/cli/commands/workflow.rb; rubygems/pray-cli/lib/pray/render_dest.rb; rubygems/pray-cli/lib/pray/verify_provisioned.rb.
- Regressions: crates/pray-cli/tests/update_destinations.rs; npmjs/pray-cli/src/update-destinations.test.ts; rubygems/pray-cli/spec/pray/update_destinations_spec.rb.

Prior-art sources checked on 2026-09-07 informed recovery and validation, without defining Pray's ownership contract:

- https://clig.dev/ describes scriptability, useful dry-run output, and recovery suggestions. It presents conventions rather than requiring one overwrite policy.
- https://raw.githubusercontent.com/nix-community/home-manager/release-26.05/modules/files.nix and https://raw.githubusercontent.com/nix-community/home-manager/release-26.05/modules/files/check-link-targets.sh show collision checks before the activation write boundary. This is source inspection, not an executed activation test.
- https://microsoft.github.io/apm/reference/cli/install/ documents grouped diagnostics and force behavior. https://github.com/microsoft/apm/pull/1313 describes byte-identical adoption after loss of deployed-file records, which does not prove ownership across a version change.
- https://www.chezmoi.io/reference/command-line-flags/global/ documents force, interactive, and error-on-conflict options. These illustrate alternative policies rather than a requirement to add prompts to Pray.
