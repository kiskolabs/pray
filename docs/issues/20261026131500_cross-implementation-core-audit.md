# Cross-implementation core audit and benchmark round

## Participants

Andrei Makarov

## Decisions

The audit covers the Rust reference implementation, the Ruby gem, and the TypeScript npm package against the stable RFC contracts. Correctness and trust-boundary defects take priority over throughput. A finding is closed only when a regression test failed before the change and passed after it, or when an existing test exposed the regression.

The shared warm benchmark uses the same one-package input in each implementation: 278 input bytes, 48 rendered output bytes, 20 warm-up iterations, and 500 measured iterations. It reports p50, p95, operations per second, whole-process wall and CPU time, and peak RSS. Parse, render, and lock are synchronous in every harness. Resolve remains asynchronous only in TypeScript because that is its public contract. The lock measurement omits layout work in Ruby and TypeScript so it matches the Rust embed API.

Package sizes are release gates, not informal observations. The new size check builds and measures the Rust release binary, Ruby gem, and npm tarball. The npm check also measures unpacked bytes and entry count. Source maps stay available to development builds but are absent from the published TypeScript build. Test compilation writes to dist-test and cannot contaminate dist.

RFC 0011 now states the source no-follow component rule. RFC 0010 now spells out Ruby pessimistic-version semantics: a two-component constraint permits later minor releases within the same major, while a three-component constraint permits later patches within the same minor.

No dependency was added. The dependency audit reuses the full 2026-09-04 reconnaissance and refreshes local advisory and freshness checks where this environment permits it.

## Effects

### Corrected findings

F1, high: Rust serve exposed any root-relative file over HTTP and stdio RPC, including .pray/auth.db, and followed a public symlink outside the distribution root. A distribution-layout allowlist and component no-follow walk now return 404 for private paths and symlinks while v1/index.json remains available. Ten HTTP/RPC bridge tests pass. A local writer can still race the metadata check and open; remote GET alone cannot create that race.

F2, high: declared source files and compose locals followed symlinks in Ruby and TypeScript. Both ports now inspect existing components before reading. Package-directory selection was handled separately in F14.

F3, high: Rust and Ruby treated filesx/secret.md as a member of the files folder export, which could map an undeclared files/x/secret.md into a destination. Matching now requires the path-component boundary files/. TypeScript already used the correct boundary. The regression tests reject or leave empty a folder with only the shared string prefix.

F4, high: the Rust resolver ignored a dependency constraint when the dependency also appeared explicitly in Prayfile. It now merges and requeues the constraint, then verifies an already resolved package. The new conflict test failed before the fix and passes after it.

F5, high: all three ports implemented ~> 1.4 as less than 1.5.0 even though the operator is specified as Ruby pessimistic and existing update behavior expects 1.5.0 to remain eligible. All ports now accept later minor releases below 2.0.0 for ~> 1.4 and retain the narrower patch rule for ~> 1.4.0. This correction also restored the selected-package update tests after dependency constraints began to be enforced.

F6, high: Ruby and TypeScript rendered only the first output in a legacy target and did not enforce max_bytes. Both now render every output and reject an oversized completed output before materialization. Focused Ruby and TypeScript tests pass.

F7, high: Ruby and TypeScript static servers followed symlinks outside the distribution root. Ruby also accepted a traversal-shaped federation package name. The contrary pass then found that an allowlisted prefix followed by parent components could return to a private file inside the root. Both servers now expose only the RFC 0060 static layout, reject parent and symlink components, and validate Ruby federation package identity. Focused static-path tests failed on the parent-component case before the second fix and pass after it. Descriptor-relative no-follow opening remains the stronger solution for a distribution root shared with a less-trusted local writer.

F8, medium: Ruby lock serialization emitted literal control characters, producing TOML its own parser rejected. Basic-string control characters now use TOML escapes or Unicode escapes. The round-trip regression passes.

F9, medium: Ruby ContentBuilder counted the entire accumulated string whenever it needed the next line, making repeated managed spans quadratic. It now maintains the next line number as content is appended. A same-process, same-payload probe with nine samples and 4,000 append-and-line-number steps measured a median 2.859 ms for the old algorithm and 0.342 ms for the new algorithm on this machine. Functional line-number and render tests pass. This microprobe isolates the algorithm and is not an end-to-end render result.

F10, medium: the npm package shipped JavaScript and declaration source maps. A clean package was 250,430 packed bytes, 1,249,170 unpacked bytes, and 694 entries. Disabling production maps and separating test output produced 142,641 packed bytes, 630,844 unpacked bytes, and 356 entries after the frozen, render-check, and provisioned-owner corrections. This is 43.0 percent fewer packed bytes and 49.5 percent fewer unpacked bytes. The package no longer depends on whether tests ran before npm pack.

F11, medium: artifact sizes were measured but not gated. make size-check now caps the Rust binary at 9,500,000 bytes, the gem at 131,072 bytes, and the npm package at 150,000 packed bytes, 650,000 unpacked bytes, and 360 entries. The latest observed round passed at 8,892,336; 115,200; 142,641; 630,844; and 356 respectively. Four new npm entries are the JavaScript and declaration outputs for the shared render-check and provisioned-owner helpers.

F12, medium: Ruby's handwritten manifest JSON encoder emitted raw control bytes other than newline, carriage return, tab, quote, and backslash. The resulting value was not valid JSON. JSON.generate produced identical bytes for all 14 valid repository Prayfiles and encoded the complete control-character regression correctly. Replacing the encoder reduced its fixture p50 from about 24 us to 1 us and deleted the duplicate escaping implementation.

F13, performance: the Ruby literal and statement-surface parsers allocated substrings and scanning state on the common path even when an input had no comment, brace, semicolon, or parenthesized call. Direct character access and guarded fast paths reduced the probe's allocations per manifest parse from 1,118 to 570 and per prayspec parse from 1,006 to 481. These are allocation counts from the fixed local fixture, not scaling claims.

F14, high: Ruby and TypeScript accepted a package directory reached through a symlink below the selected project root. Public regressions failed before the changes. Both implementations now walk the declared directory components without following symlinks before package files are read; the focused source and conformance runs pass. Rust already enforced this case. All three implementations still have a check-to-open interval because portable descriptor-relative no-follow opening is not used for the complete read.

F15, performance: the shared scale fixture exposed quadratic scoped rendering. TypeScript recomputed the line count from all accumulated content for every managed span, and Ruby and TypeScript searched every declared package for every compose entry. At 100 packages, Ruby render p50 was 680.81 us and TypeScript 828.00 us. Incremental line tracking and per-render lookup maps reduced those results to 391.83 us and 190.29 us. Target-entry order and rendered SHA-256 remain unchanged. Focused render suites pass in both ports.

F16, performance: Ruby manifest parsing linearly searched the growing package and local arrays during each insertion and binding. Parser-owned name and path indexes now retain the public arrays and duplicate-merge behavior while avoiding the repeated scans. At 1,000 packages, parse p50 fell from 92,294 to 50,125 us and resolve p50 from 251,911 to 211,144 us. The 100-to-1,000 package parse increase fell from 17.6 times to 10.5 times for a tenfold input increase. The focused parser, formatter, conformance, and shared-corpus run passed 87 examples.

F17, benchmark correctness: the original common harness exercised only one package and validated only output length. All three harnesses now accept PRAY_BENCH_PACKAGES, PRAY_BENCH_ITERATIONS, and PRAY_BENCH_WARMUP, generate the same 1, 10, 100, and 1,000 package fixtures, and report input bytes, output bytes, and a SHA-256 of the concatenated render output. Ruby and TypeScript now compute the correct median for odd sample counts, and TypeScript rejects partially numeric environment values. All implementations produced matching digests at every scale. The final one-package fixture is 278 input bytes and 48 output bytes with digest sha256:5742e01bbf663d1d2f04c3562126ec6fd494cba46d081239fa216c3be39237a5.

F18, performance: Rust and TypeScript also searched the growing package collection and target entries during each manifest insertion. Parser-lifetime name indexes now preserve declaration order, duplicate merging, and the public ordered vectors while avoiding the repeated scans. At 1,000 packages, Rust parse p50 fell from 6,346.81 to 3,997.62 us and TypeScript from 14,976.63 to 4,535.02 us, improvements of 37.0 and 69.7 percent. The final 100-to-1,000 package parse increases are 9.9 times for Rust and 9.6 times for TypeScript. New repeated-package tests passed before and after the refactor, and the output digests remained unchanged.

F19, high: TypeScript install did not enforce the locked and frozen contracts. A missing lock was created, frozen could rewrite the lock and rendered destinations, and successful frozen execution opened a write transaction. Eleven new integration cases produced nine failures before the correction. Frozen now includes the locked full-lock comparison, requires an existing lock, verifies strict project state and rendered bytes, and returns before a write transaction. Locked requires and compares the existing complete lock. The focused combined regression set passes 40 tests. An existing transaction test also exposed a macOS path-alias mismatch for an explicit manifest path; canonicalizing its transaction root corrected the shared library path without weakening the one-project transaction guard.

F20, high: TypeScript render --check rendered only in memory and returned success even when the destination was stale. It also created .pray transaction state. Both new integration cases failed before the correction. The command now compares the complete rendered layout with regular files on disk through a shared read-only helper and does not enter a write transaction. Both cases pass, including the stale-output verification failure.

F21, medium: the TypeScript HTTP adapter printed URL userinfo in status errors, and several command parsers either ignored unknown arguments or reported caller mistakes as unsupported capability with exit code 8. The new HTTP regression and representative install, clean, update, and login argument regressions failed before their corrections. GET, PUT, and JSON POST errors now redact userinfo. Install, apply, render, verify, drift, clean, update, plan, inspect, login, publish, trust, distribution, package-name, and init-prayer parsing now use usage failures for the covered invalid arguments. Distribution value flags also reject missing values and malformed ports before IO. The remaining silent extra-argument paths are retained as O10 rather than claiming full parser closure.

F22, high: Rust, Ruby, and TypeScript silently discarded later provisioned owners of the same destination path. RFC 0033 requires exclusive leaf ownership. New regressions failed before the corrections in every port: one of one in Rust, one of one in Ruby, and two conflict cases of three in TypeScript. All ports now reject the plan before writing and name the path, package, and export for both owners; repeated binding of the identical owner remains valid. Focused results are 12 of 12 in Rust, 14 of 14 in Ruby, and 32 of 32 in TypeScript. The TypeScript change initially exceeded its 304-line ratchet at 314 lines; moving ownership validation into a 31-line module reduced the main provisioned module to 290 lines without relaxing the gate.

### Benchmark round

The exact command was make bench-compat on Darwin arm64. Rust results were parse p50 5.29 us and p95 5.50 us; resolve 72.85 and 103.71; render 1.58 and 2.00; lock 8.58 and 10.96. Whole-process measured work was 48.93 ms wall, 48.53 ms CPU, and 4,538,368 bytes peak RSS.

Ruby results were parse p50 93.46 us and p95 199.12 us; resolve 372.02 and 565.71; render 4.67 and 11.33; lock 84.35 and 180.58. Whole-process measured work was 328.99 ms wall, 326.05 ms CPU, and 33,193,984 bytes peak RSS.

TypeScript results were parse p50 11.54 us and p95 30.50 us; resolve 121.56 and 257.63; render 3.42 and 4.58; lock 12.54 and 18.92. Whole-process measured work was 98.11 ms wall, 159.75 ms CPU, and 81,854,464 bytes peak RSS. CPU can exceed wall time because the Node process uses more than one thread.

These figures compare warm small-input operations, not end-to-end CLI latency. The fixture is too small to rank scaling, allocation behavior, disk throughput, or cold startup. Ruby has the slowest warm parse and resolve in this sample. TypeScript has the largest process RSS and its eager command imports add about 55 ms to a version command in the audit probe. Rust has the smallest process footprint and lowest latency here. A shared multi-scale fixture corpus and isolated cold processes are still required before choosing optimization work from these ratios.

The second make bench-compat round, after the final source and test changes, measured Rust parse 5.58 us, resolve 79.69 us, render 1.58 us, and lock 8.25 us at p50; Ruby 88.42, 357.90, 4.62, and 149.58; and TypeScript 11.29, 120.06, 3.50, and 12.00. Peak RSS was 4,734,976 bytes for Rust, 34,324,480 for Ruby, and 82,296,832 for TypeScript. Ruby lock p50 moved from 84.35 to 149.58 us between the two one-process rounds while its other medians stayed close. This variance is evidence that the warm harness is useful for orientation but is not yet a regression gate; the next version needs repeated isolated processes and confidence intervals.

### Ruby performance follow-up

Three isolated benchmark processes before the Ruby follow-up produced median-of-run p50 values of 86.21 us for parse, 351.19 us for resolve, 4.75 us for render, and 144.29 us for lock. Three processes after the changes produced 47.46 us, 225.71 us, 4.71 us, and 142.44 us respectively. On this fixture, parse improved by 44.9 percent and resolve by 35.7 percent. Render was already about 5 us and did not materially move.

The final common make bench-compat round measured p50 parse, resolve, render, and lock at 5.08, 70.98, 1.58, and 8.25 us for Rust; 48.71, 222.73, 4.62, and 146.67 us for Ruby; and 12.38, 129.06, 3.42, and 12.75 us for TypeScript. Peak RSS was 4,571,136 bytes for Rust, 34,816,000 for Ruby, and 81,346,560 for TypeScript. Ruby reached the proposed 220 to 260 us resolve range but remained 3.1 times the Rust p50 and 1.7 times the TypeScript p50 in this one-process round.

Three post-change YJIT processes produced resolve p50 values of 181.46, 179.83, and 182.40 us, with about 40.5 to 40.9 MB peak RSS. A separate 30-process cold version-command probe measured p50 137.77 ms without YJIT and 176.24 ms with YJIT. YJIT therefore improves this warm loop but made the measured cold CLI path 27.9 percent slower and used more memory. It remains an opt-in runtime choice rather than the CLI or benchmark default.

Several proposed shortcuts did not survive measurement or contract review. Dir.glob took about 30 us where the existing prayspec search took about 18 us; extension-first Dir.each_child took about 15 us. Removing the duplicate post-parse canonicalization was valid but measured at about 0 to 1 us. Replacing component checks with one lstat would violate the no-follow contract for an intermediate symlink, so SourcePath.read now removes redundant exist and directory calls while retaining the component walk. The claim that about 73 percent of Rust resolve is kernel IO has no producing profile in this run and remains unverified.

Rust parity is not supported by the current evidence. Even YJIT leaves Ruby about 2.6 times slower than the final Rust resolve p50, while enabling it by default worsens the measured cold command. A native extension might lower the warm parser floor, but its maintenance and artifact-size costs are not justified by this sub-millisecond CLI path without a multi-scale end-to-end workload.

### Multi-scale benchmark follow-up

The common corpus now measures 1, 10, 100, and 1,000 packages with output-digest equality. For one package with 500 measured iterations and 20 warm-ups, p50 parse, resolve, render, and lock were 5.04, 80.60, 1.58, and 8.29 us for Rust; 50.27, 239.85, 4.79, and 144.75 us for Ruby; and 11.42, 129.69, 3.58, and 12.79 us for TypeScript. Peak RSS was 4,718,592; 36,880,384; and 80,822,272 bytes respectively.

For 10 packages with 100 measured iterations and 10 warm-ups, the same p50 sequence was 39.54, 538.62, 13.67, and 62.54 us for Rust; 437.50, 2,080.46, 36.19, and 1,452.73 us for Ruby; and 72.69, 1,060.35, 25.08, and 79.83 us for TypeScript.

For 100 packages with 30 measured iterations and three warm-ups, the final p50 sequence was 405.44, 6,396.06, 150.58, and 587.40 us for Rust; 4,782.06, 20,111.48, 391.83, and 14,665.79 us for Ruby; and 471.35, 10,936.19, 195.69, and 665.67 us for TypeScript.

For 1,000 packages with 10 measured iterations and two warm-ups, the final p50 sequence was 3,997.62, 63,980.92, 2,682.67, and 6,265.90 us for Rust; 50,124.94, 211,144.27, 4,899.12, and 150,836.40 us for Ruby; and 4,535.02, 100,695.10, 1,508.96, and 6,628.81 us for TypeScript. The Rust and TypeScript parser-index rounds ran separately from the earlier three-language memory round, so their peak RSS values are not compared here.

At 1,000 packages Ruby resolve was 3.30 times the Rust p50 and 2.10 times the TypeScript p50. Ruby therefore did not reach Rust parity, but its relative resolve gap did not grow materially across this corpus. TypeScript render had the lowest 1,000-package p50 after the quadratic fixes, but its p95 was noisy and this single-machine warm result does not establish a general ranking. The parser indexes materially improved parse but only slightly changed resolve, which directs the next profile toward per-package file validation and prayspec parsing rather than more manifest lookup work.

The final one-package make bench-compat smoke round after F19 through F22 measured p50 parse, resolve, render, and lock at 5.83, 69.96, 1.58, and 8.21 us for Rust; 47.67, 232.08, 4.71, and 142.77 us for Ruby; and 11.85, 132.13, 3.63, and 12.54 us for TypeScript. Peak RSS was 4,767,744; 36,339,712; and 82,853,888 bytes respectively. All three emitted the final F17 digest. Ruby resolve remained 3.32 times the Rust result and 1.76 times the TypeScript result in this warm round; no new parity claim follows from the smoke measurement.

### Open findings

O2, high: TypeScript resolve handles only manifest packages. It neither loads missing transitive dependencies nor validates all dependency edges. It needs the same explicit queue and revalidation contract as Rust.

O5, high: TypeScript does not implement the stable Ed25519 package path or enforce trust policy during resolve and install. Its signing-key option is parsed but not applied.

O6, high: TypeScript replaces a locked yanked version instead of retaining it with a warning and does not implement strict install. Both TypeScript and Rust HTTP registry paths fetch metadata before attempting artifact-cache reuse, so cache-first or lock-first offline behavior is a cross-port architecture issue rather than a TypeScript-only defect.

O7, high: Rust sync automatically fetches up to 64 peer URLs returned by an explicitly selected seed, including loopback, private, and link-local destinations, and promotes discovered peers to full trust. RFC 0104 describes explicit peer trust but also mentions topology discovery. The contract must distinguish discovery candidates from approved peers, then the implementation must enforce that distinction and validate every redirect hop.

O8, medium: confession clients in Rust, Ruby, and TypeScript use a public SHA-256 digest called signature. The Rust and Ruby servers append without cryptographic sender verification, replay protection, quota, or authenticated identity binding. Either implement an Ed25519 identity and replay contract or rename these as unverified digests and exclude them from verified aggregates.

O9, medium: federation JSON in pray-transport, individual pre-limit HTTP request and header lines, and the confession log still permit allocations or disk growth without an effective byte or record bound. Existing core HTTP body, RPC frame, connection, and static-file limits cover adjacent paths, so this is not a claim that all server ingress or auth responses are unbounded.

O10, medium: the shared TypeScript HTTP redaction path and the covered command parsers are corrected in F21. Some no-argument commands and extra positional-argument paths in the central dispatcher can still ignore input, while prayer and repo misuse still return unsupported code 8. Central arity validation and regressions for every command remain before this finding can close.

O11, medium: TypeScript eagerly imports every command subsystem. The measured version path loaded about 71 MB peak RSS versus about 44 MB for an empty Node process. A small dispatcher with command-local dynamic imports is the next likely cold-start improvement.

O12, medium: TypeScript and Ruby retain a check-to-open race between component lstat and the later path-based read. Rust uses O_NOFOLLOW and descriptor metadata for the final leaf on Unix, but its ancestor checks are not directory-descriptor-relative, so a less-trusted local writer can still replace an already checked ancestor. Complete descriptor-relative opening remains absent across the supported implementations.

O13, medium: Ruby reads an HTTP request or header line with an unbounded gets before applying its 64 KiB aggregate header limit. The Rust server has the analogous read_line ordering issue.

O14, medium: aggregate coverage gates can conceal module-specific boundary gaps, but the claim that serve, sync, trust, and tree are generally thinly tested is not established. Tests exist for each area, and an ignored local Ruby report shows high trust, sync, and federation coverage while serve is lower. Publish current per-module Rust and TypeScript figures and identify missing negative states before adding tests or a per-module floor.

O15, performance: Rust and TypeScript parser indexes and digest-checked multi-scale before and after measurements exist in F17 and F18. There is no automated scaling threshold. Add one only after isolated-process variance supports a stable limit.

### Dependency and quality checks

cargo deny check advisories licenses bans sources passed: advisories, bans, licenses, and sources were all ok. It reported duplicate base64, getrandom, hashbrown, r-efi, and syn lines, plus three unused license allowances. cargo-libyear processed 307 packages and reported 0.00 libyears behind.

RBENV_VERSION=4.0.6 rbenv exec bundler-audit check . reported no vulnerabilities using the installed advisory database. The locked Ruby development bundle is incomplete in this environment, so the repository's bundle exec wrapper and Ruby libyear task could not run.

npm audit could not run in the sandbox. Network resolution first failed, and the escalated request was rejected because the command uploads the dependency graph to npm. npm libyear was stopped after one minute without output. The last full repository dependency audit is docs/issues/20260904113000_dependency-audit.md. This round therefore does not make a fresh zero-vulnerability claim for npm.

cargo clippy --workspace --all-targets --all-features -- -D warnings and cargo test --workspace pass on an observed complete tree. The first full test run exposed the pessimistic-constraint regression after the resolver fix; focused tests passed after the cross-port correction, and the full rerun passed. A concurrent source-read refactor briefly left incompatible call signatures and later changed files between library and rustdoc compilation, producing one transient doctest failure. On the settled source_path snapshot, cargo test -p pray-core, its doctests, and targeted Clippy all pass. The refactor initially pushed resolve.rs to 307 lines; moving that module's existing unit tests into resolve_unit.rs restored the line-count gate without changing production behavior. make loc-check finishes with 177 warnings and zero failures.

The latest npm full test run compiled the separated test tree and passed 310 of 310 tests when the local 127.0.0.1 listener was permitted. The earlier restricted run passed 306 of its then 307 tests and failed only the registry metadata listener with EPERM. Type checking, the complete Biome check, and the circular-dependency check pass. The final Ruby focused runs passed 22 source and conformance examples, 47 literal and parser examples, 29 render examples, 87 parser, formatter, conformance, and shared-corpus examples, and the later 34-example destination and render set. The raw full Ruby suite reported 357 examples and the same 17 environment failures: 12 subprocess tests lacked locked development gems and five HTTP tests could not bind a local port. The locked development bundle gained versions not present in the local gem set during this round, so the final changed-file RuboCop checks ran directly under Ruby 4.0.7 instead of through bundle exec; they passed. No changed-path assertion failed in either port.

cargo fmt --check -p pray-bench, cargo clippy -p pray-bench --example compat_bench -- -D warnings, and cargo test -p pray-bench pass. After F22, cargo test --workspace and cargo clippy --workspace --all-targets --all-features -- -D warnings pass. The latest make size-check passes at the sizes recorded in F11. The first post-F22 make loc-check correctly failed when the TypeScript provisioned module grew past its 304-line ratchet; after the module split it finishes with 175 warnings and zero failures. TypeScript main.ts is 287 lines, provisioned.ts is 290, and the new ownership helper is 31. The 153-line Rust compatibility example is above the soft 150-line preference but below the hard limit and remains one cohesive harness. git diff --check passes.

### Claim audit

The claims contrary pass rejected one earlier performance comparison because its before and after probes used different payload sizes; F9 now reports the same-payload result. It also produced the parent-component server regression described in F7, which failed before the second fix and passes after it. The Ruby follow-up confirmed the JSON and common parser-cost claims, qualified the YJIT claim as warm-loop-only, rejected the proposed glob and single-lstat shortcuts, and left the Rust kernel-IO share unverified. The predicted 220 to 260 us Ruby resolve range was observed, while the 150 to 180 us parser-refactor range was not observed without YJIT and remains a forecast rather than a result. The O1 and O4 contract claims were supported and are corrected in F19 and F20. The later O2 through O14 contrary pass supported O2, O3, O7, O8, and O13; classified O5, O6, O9, O10, O11, O12, and O14 as partial; and corrected the scope of O3, O6, O9, O12, and O14 above. It did not reproduce the recorded O11 RSS numbers, so they remain local observations rather than independently verified results. Interim Rust status claims became stale twice while the concurrent source-read refactor changed the shared tree, so this note distinguishes the complete workspace run from the final settled pray-core run. Quantities in this note come from named commands observed in this run. Raw benchmark and package-size stdout is not stored as a separate artifact, so another machine should treat the figures as reproducible observations, not immutable release records. No fresh npm vulnerability claim is made because the current advisory request did not run.

### Lessons

A common output shape is not enough for a common benchmark. The measured boundary, fixture bytes, sync versus async call cost, distribution statistic, and output validation must also match.

Aggregate coverage and aggregate speed both hide trust-boundary failures. The least-covered server, sync, trust, and source-path modules contained most high-severity findings.

An optimization can reveal a contract defect. Enforcing dependency constraints exposed that the shared pessimistic-constraint helper contradicted both Ruby semantics and existing update behavior.

Lexical containment does not establish filesystem containment. Source and server reads need component-aware no-follow handling, and higher-assurance local sharing needs descriptor-relative opening.

Package size depends on the build graph and on command history. Production and test output must have distinct directories, and the release check must inspect the actual archive rather than the source tree.

No single package-size number is comparable across ecosystems. A native executable, compressed gem, and npm tarball have different meanings, so each keeps a named metric and budget while changes are compared within that metric.

Fast paths should first remove work that the input proves unnecessary. Avoiding substring allocation and skipping scanners for lines without their trigger characters yielded more than replacing the parser architecture would have justified at this fixture size.

A one-package benchmark can hide the dominant algorithm. The first scale round exposed repeated full-content line counting and per-entry collection scans that were immaterial at one package and quadratic at 100 packages.

Comparable benchmarks need a content identity check, not just equal byte counts. Matching SHA-256 output digests caught the whole rendered payload while retaining low measurement overhead outside the timed operation.

Public ordered arrays and fast parser lookup are compatible. Parser-lifetime indexes can accelerate duplicate detection and binding without changing output order or exposing a second data model.

Cross-port agreement can preserve a shared defect. All three implementations silently coalesced provisioned owners until RFC 0033 and a cross-language regression were checked together.

A line-count ratchet should fail before a crowded module grows. The O3 helper split restored the gate without raising its allowance, and the package-size gate made the two-entry publication cost explicit.

A microbenchmark runtime mode is not automatically a product default. YJIT improved repeated warm resolution but lost on the measured cold command and memory budget, which matters more for a short-lived CLI.

The command-line and HTTP surfaces changed only in failure behavior: unsafe inputs now fail instead of exposing or writing data. There is no new browser screen. The visual-surface assessment covered CLI error clarity and HTTP success, not-found, and too-large states; no human visual review remains for these non-visual changes.

Audit modes applied: resource and budget, trace and identification, performance, observability, security, contracts, lineage, product and terminal surface, privacy, HTTP identity, and the Rust native boundary. Learned-system and browser-client modes were skipped because these executables neither run inference nor ship a browser client.

## Next

Implement O2, O5, and the TypeScript-specific O6 behavior before claiming stable-contract parity. Finish the remaining O10 arity paths. Each needs a failing test or shared fixture before the change.

Resolve the RFC 0104 trust ambiguity, then change sync discovery so an advertised peer is only a candidate until explicitly approved. Add private-address and redirect-hop tests.

Specify confession authentication and replay fields before replacing the current digest. Until then, label and aggregate the records as unverified.

Keep the shared 1, 10, 100, and 1,000 package corpus and output digests aligned. Add verify and unchanged-install operations; run cold CLI commands in at least 30 isolated processes; report p50, p95, p99, user and system CPU, peak RSS, input/output bytes, and runtime versions.

Profile per-package file validation and prayspec parsing at 100 and 1,000 packages before the next resolve change. Add an automated scaling guard only after isolated-process variance establishes a threshold that is strict enough to catch quadratic regressions without making CI hardware noise a product failure.

Add descriptor-relative no-follow helpers where supported, bounded federation and header readers, and quotas for append-only feedback.

Keep make size-check in the release path. Record the reason before relaxing any budget, and report both old and new artifact measurements.

Rerun the full Ruby suite in an environment that has the locked Ruby development bundle. Run a fresh npm advisory check only where sending the dependency graph to npm is approved.

## Source

rfcs/0010-core-formats.md

rfcs/0011-prayspec-and-package.md

rfcs/0020-resolve-and-lock.md

rfcs/0050-security-and-trust.md

rfcs/0060-distribution.md

rfcs/0070-reference-implementation.md

rfcs/0104-federation-transports.md

docs/issues/20260904113000_dependency-audit.md

docs/issues/20261026114500_source-symlink-rejection.md

docs/changelogs/20261026143848_core-audit-corrections-and-performance.md

crates/pray-bench/examples/compat_bench.rs

rubygems/pray-cli/benchmarks/compat_bench.rb

npmjs/pray-cli/scripts/compat-bench.mjs

scripts/check-package-sizes.rb

Commands: make bench-compat; PRAY_BENCH_PACKAGES set to 10, 100, and 1000 with matching PRAY_BENCH_ITERATIONS and PRAY_BENCH_WARMUP for each Rust, Ruby, and TypeScript harness; make size-check; make loc-check; git diff --check; RBENV_VERSION=4.0.7 /opt/homebrew/bin/rbenv exec ruby --yjit benchmarks/compat_bench.rb; RBENV_VERSION=4.0.7 /opt/homebrew/bin/rbenv exec rspec; focused Ruby parser, source, render, JSON, conformance, shared-corpus, and provisioned-owner specs; direct Ruby 4.0.7 RuboCop over changed Ruby files; cargo fmt --check -p pray-bench; cargo clippy -p pray-bench --example compat_bench -- -D warnings; cargo test -p pray-bench; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace; cargo test -p pray-cli --test install_update -- --nocapture; cargo deny check advisories licenses bans sources; cargo-libyear --sort libyear --top 10; RBENV_VERSION=4.0.6 /opt/homebrew/bin/rbenv exec bundler-audit check .; npm test with local listener permission; npm run --silent typecheck; complete Biome and circular-dependency checks; focused Node source, render, frozen, HTTP, CLI argument, and provisioned-owner tests; npm audit --omit=dev --json; npm run libyears.
