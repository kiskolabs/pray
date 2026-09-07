# Legacy lock update: destination recovery

## Participants

Andrei Makarov

## Decisions

Keep the destination ownership rules in RFC 0033. Existing bytes may be adopted when they equal the requested export, or replaced when they still match the previous lock's content hash. An empty provisioned array does not establish ownership. Force and destination overwrite prompts remain outside this change.

The implementation pass on 2026-09-07 narrows the earlier proposal: fix candidate resolution, write ordering, previews, and recovery first. Automatic adoption across a legacy version transition remains subject to the evidence rule below. Keep the implementation on the current branch. Maintain this issue and CHANGELOG.md as the documentation for this work.

The follow-up decision on 2026-09-07 extends recovery to interrupted processes in Rust, Ruby, and TypeScript together. Use one project ownership and recovery format across the implementations. Persist recovery before changing destination bytes. This replaces the earlier scope exclusion for rollback.

Automatic legacy adoption requires authenticated old payload bytes and export metadata, plus a reconstruction of the historical destination mapping, selected exports, symbols, and effective environment. Re-render those exact inputs and compare the old result with the existing destination before authorizing replacement. Each input must be bound to the historical lock or another authenticated historical record. Current configuration and cache presence alone do not meet that rule.

The existing legacy format does not supply that complete evidence. Its tree hash covers listed package files, which need not include the prayspec defining exports. A manifest hash cannot reconstruct missing inputs, and the canonical manifest changed when the unused render fields were removed in 1.10.0. Preserve the existing refusal across differing versions. Byte-identical adoption and replacement against a recorded destination hash remain available; this change adds no speculative legacy reconstruction.

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

### Initial validation

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

The first full Rust and Ruby runs failed because the sandbox denied localhost listeners used by HTTP fixtures. Both suites passed when rerun with localhost access. Ruby's locked gem was installed from an existing local cache into temporary storage; package manifests and lockfiles were unchanged. That earlier pass measured neither coverage percentage nor performance.

### Persistent recovery

Install, apply, update, unlock, add, remove, render, plan, verify, and drift acquire project ownership and recover an interrupted write before executing. The shared render writer also participates. Recovery covers compose files, exclusive destinations, dropped leaves, Prayfile edits, and the lock within the command. Plan and verification may therefore restore an interrupted command before inspecting the project.

All three implementations use .pray/write-state with an owner-only directory and journal. A fully written, synced owner record is published through an exclusive hard link. Its process ID, hostname, and random token coordinate cooperating writers across runtimes. A live owner or an owner on another host prevents takeover. A token-specific claim serializes removal of a dead owner's record; timeouts do not steal ownership.

The version 1 journal records start, write, undone, and commit transitions. Write records contain a relative path, original bytes, expected replacement hash, and original permission bits. Each record is synced before the corresponding file change. Replacement bytes are staged and synced, then installed by rename or exclusive hard link. Destination parent directories are synced up to the project root. The commit record is synced before recovery data is removed.

An ordinary failure attempts rollback immediately. A later invocation resumes rollback after a process exits. Recovery works backwards and persists each completed restoration, including repeated writes to the same path. A partial final journal record is discarded because its associated mutation had not been authorized yet. Complete malformed records stop recovery.

Recovery preserves bytes that match neither the original nor the recorded replacement. It retains recovery data and tells the operator to inspect the changed path, move it aside, and retry. A missing path can then be restored from the saved original. Directory synchronization must succeed before restoration is marked complete, including when an earlier recovery already restored the bytes.

### Resource bounds

Existing destination reads now reject files above 32 MiB before allocating their contents, and enforce the same ceiling while reading. Ruby and TypeScript read small files in small chunks. A transaction accepts at most 10,000 writes and 64 MiB of cumulative original plus replacement payload. Individual originals and replacements are capped at 32 MiB; the encoded journal is capped at 96 MiB. These are file and transaction limits, not a whole-process memory ceiling: source rendering and parsed project state have their own allocation costs.

Batch destination validation and verification index previous destination hashes once. Plan reuses collected statuses. Mutation-time reads remain necessary to detect changes since preview. Collision diagnostics retain at most 100 details and 60 KiB of detailed text, followed by an omitted count; checking still visits every planned destination.

### Follow-up validation

The destination-size and diagnostic-limit regressions failed before implementation, then passed. Transaction tests cover ordinary failure, abrupt process exit, repeated writes, later edits, and exclusion of another writer. Late lock-write failures restore compose output, exclusive leaves, pruned files, manifest, and lock. The Ruby directory-sync regression first allowed a new operation after a failed restoration sync; it now refuses until recovery completes. TypeScript also rejects writes from asynchronous work that outlives its transaction.

From the repository root:

- cargo test --offline -p pray-core --test write_transaction --test destination_budget --test client_trust: passed, 7 tests.
- cargo test --offline: passed, 369 tests and 4 ignored tests across 74 reported test targets, including documentation-test targets. Localhost fixture access was enabled. Test subprocesses inherited commit.gpgsign=false and a disabled hooks path through GIT_CONFIG_COUNT, leaving repository configuration unchanged.
- cargo clippy --offline: passed.
- cargo fmt --check: passed.
- make loc-check: passed, 138 warnings and 0 failures; existing size ratchets were unchanged.
- git diff --check: passed.
- cargo build --offline -p pray-cli and cargo build --offline -p pray-bench --example destinations: passed.

From npmjs/pray-cli:

- npm test: passed, 153 tests.
- npm run lint: passed, including cycle detection.
- npm run typecheck: passed.

From rubygems/pray-cli, using Ruby 4.0.0 and Bundler 4.0.15 with the same isolated GEM_PATH supplement described above:

- bundle exec rspec: passed, 198 examples with localhost fixture access and isolated test Git settings.
- bundle exec rspec spec/pray/transaction_spec.rb spec/pray/update_destinations_spec.rb: passed, 9 examples after extracting per-entry restoration to satisfy the complexity check.
- bundle exec rubocop --cache false: passed, 137 files and no offenses.
- bundle exec rbs -I sig validate: passed.
- bundle exec rubocop --cache false --config .rubocop.yml ../../scripts/check-write-recovery.rb: passed.
- bundle exec ruby ../../scripts/check-write-recovery.rb: passed, 12 cross-implementation cases. Each implementation produced interrupted writes for both other implementations to recover, with and without later edits. The cases also retain original permissions, discard a partial final record, and resume a partially completed rollback.

The first Rust run encountered the host Git signing agent; the isolated test configuration removed that fixture dependency. Another run failed a trust fixture that passed alone; its timestamp-only temporary directory naming could collide between parallel tests, so a per-process counter now distinguishes those directories. The full suite passed afterward. No coverage percentage is claimed.

### Initial measurements

The destinations example constructs local package trees and matching old destination ledgers outside the timed region, then runs the full plan command. Measurements use Darwin arm64, rustc 1.97.0 with the debug build, Node 22.14.0, and Ruby 4.0.0. Each entry is one isolated sample including CLI startup, resolution, lock parsing, rendering, and destination comparison. They are observations for this fixture, not throughput guarantees or a controlled runtime comparison.

Commands from the repository root were target/debug/examples/destinations COUNT target/debug/pray and target/debug/examples/destinations COUNT node npmjs/pray-cli/bin/pray.js, for COUNT values 100, 1000, and 10000. The Ruby command used the same example with the Ruby 4.0.0 and Bundler launcher, loading rubygems/pray-cli/lib and invoking Pray::CLI.run(ARGV). The example invokes the system time utility with its resource-reporting flag and isolates Pray home and cache directories.

Results give wall seconds, user CPU seconds, system CPU seconds, and peak resident MiB:

- Rust, 100 files: 0.31, 0.03, 0.02, 15.52.
- Rust, 1,000 files: 0.38, 0.09, 0.08, 25.30.
- Rust, 10,000 files: 1.87, 0.71, 0.97, 86.50.
- TypeScript, 100 files: 0.28, 0.09, 0.04, 67.19.
- TypeScript, 1,000 files: 0.25, 0.16, 0.08, 80.92.
- TypeScript, 10,000 files: 1.05, 0.49, 0.65, 148.14.
- Ruby, 100 files: 1.07, 0.64, 0.12, 59.95.
- Ruby, 1,000 files: 51.27, 49.39, 0.68, 205.47.
- Ruby, 10,000 files: stopped at the 60-second process-group deadline; completion time and peak resident memory are unavailable.

The Ruby slowdown persisted after replacing the initial large read allocation with small chunks. A one-second native sample during an isolated 1,000-file run placed 828 samples in remove_class_from_subclasses within Ruby garbage collection. This located a hotspot in the measured workload; it did not establish an upstream defect or isolate its trigger. Earlier overlapping runs were discarded, and verified leftover benchmark workers were stopped before the reported measurements.

With PRAY_BENCH_OVERSIZED=1 and COUNT=1, the example creates a sparse 32 MiB + 1-byte destination and expects the CLI's render-refusal exit code 5. All three rejected it. The same wall/user/system/resident measurements were Rust 0.07/0.00/0.00/8.84, TypeScript 0.35/0.09/0.06/62.59, and Ruby 0.81/0.25/0.14/42.59. The Ruby launcher translated Pray::Error to its CLI exit code. These samples measure oversized-file refusal, not reading or writing a near-limit valid file.

### Ruby performance follow-up

Replace toml-rb 4.2.1 and its Citrus dependency with perfect_toml 0.9.1. The old parser allocated 1,214,484 objects while loading a 1,000-destination lock in the regression test. A direct parser comparison measured about 142,000 allocations with tomlrb 2.0.4 and 28,000 with PerfectTOML. Three isolated PerfectTOML parses took 0.0087 to 0.0092 seconds each. These parser measurements exclude CLI startup and destination work.

Resolution now retains its previous lock for the lifetime of the resolved project. Preview, installation, and locked or frozen checks reuse that snapshot. A new resolution reads the lock again. Tree collection computes the destination root relative to the project once, then normalizes each leaf. Destination ownership and mutation-time filesystem checks still run.

Prism parses Ruby source. The Ruby 4.0.0 runtime used here already reports Prism as its source parser. Pray's custom manifest and package grammar remains unchanged. In a 10,000-file phase profile with the intermediate tomlrb candidate, manifest parsing took 0.0002 seconds, package-spec parsing took 0.091 seconds, and three lock reads took 1.870 seconds. That profile directed the change toward TOML parsing and redundant work. A YJIT experiment with the same intermediate candidate reduced the 10,000-file median from 2.14 to 2.00 seconds but increased the 100-file median from 0.27 to 0.38 seconds. JIT remains optional; those intermediate measurements do not describe the final parser.

PerfectTOML 0.9.1 was the latest published release in the RubyGems metadata checked on 2026-09-07, dated 2026-02-25. Its gemspec declares MIT licensing, Ruby >= 3.1, no runtime dependencies, and no native extensions. The downloaded gem's SHA-256 matched the registry checksum recorded in Gemfile.lock. Source inspection found no installation hook or input evaluation in the parser. The application tests below establish compatibility with Pray's exercised inputs; this pass did not independently run the upstream TOML conformance suite.

The dependency's load_file convenience method opens an input without explicitly closing it. A local reproduction with garbage collection disabled retained 20 open handles after 20 calls. Pray uses File.read followed by PerfectTOML.parse instead, closing each input promptly. The upstream fix would be a block or ensure around the open file; no upstream report was sent. This issue records the evidence and workaround within the requested documentation scope.

### Ruby performance validation

Added executable allocation-budget regressions before the fixes. The initial lock parse exceeded its 300,000-object budget with 1,214,484 allocations. With the intermediate parser, planning exceeded that budget with 436,618 allocations before lock reuse. Tree collection exceeded its 100,000-object budget with 224,026 allocations before root normalization. The final tests pass with tighter 60,000-object budgets for parsing and planning, and the same 100,000-object tree budget. Allocation limits avoid timing assertions that depend on machine load.

Added syntax cases for quoted keys, Unicode escapes, comments, multiline arrays, and trailing commas. A duplicate-field case first leaked TomlRB::ValueOverwriteError; it now reports Pray::Error with lockfile context. Existing fixture round trips continue to exercise the unchanged serializer.

From rubygems/pray-cli, using isolated gem directories and Bundler 4.0.15:

- bundle exec rspec spec/pray/lockfile_performance_spec.rb spec/pray/lockfile_spec.rb spec/pray/trust_spec.rb: passed, 12 examples on Ruby 4.0.0.
- bundle exec rspec: passed, 203 examples on Ruby 4.0.0 and 203 on Ruby 3.4.8, with localhost fixture access and the isolated Git settings described above.
- bundle exec rubocop --cache false: passed, 138 files and no offenses.
- bundle exec rbs -I sig validate: passed.
- bundle exec ruby ../../scripts/check-write-recovery.rb: passed, all 12 cross-implementation recovery cases with the final parser.
- bundle exec bundler-audit check --database "$PRAY_ADVISORY_DATABASE": no vulnerabilities reported for the final locked graph. The temporary database was refreshed using check --update --database; snapshot e7179ad21701894b75796c5ddd72e5fbfc446165 contained 1,242 advisories and was last updated on 2026-09-05. This result covers published entries in that snapshot.

A separate Ruby 3.1.6 parser smoke test loaded PerfectTOML and parsed a Unicode destination record. The full application suite was run on 3.4.8 and 4.0.0 only. Initial 3.4.8 validation encountered a launcher PATH mismatch and then incompatible native gems in the shared temporary cache; separate runtime-specific gem directories resolved that setup problem.

From the repository root:

- cargo build --release --offline -p pray-cli: passed.
- make loc-check: passed, 138 warnings and 0 failures; size ratchets were unchanged.
- git diff --check: passed.

### Ruby performance measurements

The final comparison uses the same destinations example and local fixture described above, with 16-byte source and destination files. Rust uses the release build for this comparison. Node remains 22.14.0 and Ruby remains 4.0.0 without YJIT, on Darwin arm64. Each size and implementation ran three times sequentially, without other test suites or benchmark workers. Fixture creation is excluded; complete CLI startup, resolution, parsing, rendering, and destination comparison are included.

Commands were target/debug/examples/destinations COUNT target/release/pray, target/debug/examples/destinations COUNT node npmjs/pray-cli/bin/pray.js, and the same example with the Ruby and Bundler launcher described above, for COUNT values 100, 1000, and 10000. Results below are the three-sample median wall seconds:

- 100 files: Rust 0.29, TypeScript 0.32, Ruby 0.24.
- 1,000 files: Rust 0.27, TypeScript 0.23, Ruby 0.36.
- 10,000 files: Rust 1.31, TypeScript 1.00, Ruby 1.50.

At 10,000 files, wall-time ranges were Rust 1.19 to 1.36 seconds, TypeScript 1.00 to 1.05 seconds, and Ruby 1.49 to 1.56 seconds. Median peak resident memory was Rust 97.30 MiB, TypeScript 148.39 MiB, and Ruby 92.66 MiB. Ruby's earlier single 1,000-file sample was 51.27 seconds, and its earlier 10,000-file run exceeded 60 seconds. The final Ruby median is about 15 percent slower than release Rust and 50 percent slower than TypeScript at 10,000 files. These observations resolve the measured pathological slowdown while leaving a smaller runtime gap; they establish neither exact parity nor performance across other workloads.

## Next

Validate crash recovery on additional operating systems and filesystems. This pass exercises process exit and I/O failures on macOS; it does not simulate power loss. Rust conservatively refuses stale-owner takeover outside Unix. Directory syncing is skipped on Windows in the ports. Automatic recovery on Windows remains unvalidated. The ownership protocol assumes a local filesystem and one host process namespace; network filesystems and duplicate hostnames need separate validation.

Cooperating Pray commands serialize writes, but readers can observe intermediate files. External writers can still race between the final check and replacement. Staging requires destinations to share a filesystem with the project recovery directory; cross-filesystem replacement fails and enters recovery. Restored state covers file contents and POSIX permission bits, not timestamps, inode or hard-link identity, ACLs, or extended attributes. Empty created directories and source-cache changes may remain.

Extend the final Ruby comparison to other package shapes, near-limit valid files, sustained writes, and recovery under disk pressure before setting a broader performance target. No whole-process memory, power-loss, or hostile-filesystem guarantee is claimed.

## Source

- rfcs/0033-provisioned-destination-safety.md; rfcs/0031-ownership-and-generated-output.md; rfcs/0040-cli-surface.md.
- CHANGELOG.md, versions 1.9.1 and 1.10.0 and the current Unreleased entries.
- Historical writers: git show v1.9.1:crates/pray-core/src/render_provisioned.rs and git show v1.9.2:crates/pray-core/src/render_provisioned.rs.
- crates/pray-cli/src/commands_update.rs; crates/pray-cli/src/commands_update_latest.rs; crates/pray-core/src/render_dest.rs; crates/pray-core/src/render_write.rs; crates/pray-core/src/verify/provisioned.rs.
- npmjs/pray-cli/src/cli/commands/update-core.ts; npmjs/pray-cli/src/cli/commands/update-latest.ts; npmjs/pray-cli/src/render/dest.ts; npmjs/pray-cli/src/verify/provisioned.ts.
- rubygems/pray-cli/lib/pray/cli/commands/workflow.rb; rubygems/pray-cli/lib/pray/render_dest.rb; rubygems/pray-cli/lib/pray/verify_provisioned.rb.
- Persistent recovery: crates/pray-core/src/transaction; npmjs/pray-cli/src/transaction; rubygems/pray-cli/lib/pray/transaction.
- Reproduction helpers: scripts/check-write-recovery.rb; crates/pray-bench/examples/destinations.rs.
- Regressions: crates/pray-cli/tests/update_destinations.rs; npmjs/pray-cli/src/update-destinations.test.ts; rubygems/pray-cli/spec/pray/update_destinations_spec.rb.
- Ruby performance regressions: rubygems/pray-cli/spec/pray/lockfile_performance_spec.rb; rubygems/pray-cli/spec/pray/lockfile_spec.rb.
- Parser source and metadata checked on 2026-09-07: https://github.com/mame/perfect_toml; https://rubygems.org/gems/perfect_toml/versions/0.9.1; https://rubygems.org/api/v1/gems/perfect_toml.json.
- Parser alternatives checked on 2026-09-07: https://ruby.github.io/prism/; https://github.com/fbernier/tomlrb; https://github.com/emancu/toml-rb.

Prior-art sources checked on 2026-09-07 informed recovery and validation, without defining Pray's ownership contract:

- https://clig.dev/ describes scriptability, useful dry-run output, and recovery suggestions. It presents conventions rather than requiring one overwrite policy.
- https://raw.githubusercontent.com/nix-community/home-manager/release-26.05/modules/files.nix and https://raw.githubusercontent.com/nix-community/home-manager/release-26.05/modules/files/check-link-targets.sh show collision checks before the activation write boundary. This is source inspection, not an executed activation test.
- https://microsoft.github.io/apm/reference/cli/install/ documents grouped diagnostics and force behavior. https://github.com/microsoft/apm/pull/1313 describes byte-identical adoption after loss of deployed-file records, which does not prove ownership across a version change.
- https://www.chezmoi.io/reference/command-line-flags/global/ documents force, interactive, and error-on-conflict options. These illustrate alternative policies rather than a requirement to add prompts to Pray.
