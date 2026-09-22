# Publish version preservation audit

## Participants

Andrei Makarov

## Decisions

Audit scope is main at 41b82b6 and GitHub issue 26. The local publish paths in Rust, Ruby, and TypeScript are in scope. The Rust registry merge predicate is in scope because publish can send the same version to pray serve. Cache, database, queue, and worker stages are not on the local publish path.

RFC 0061 specifies the proposed contract. A verified existing artifact with the same expected path, version, tree, prayspec, signer, and signature makes local publish a no-op for that package. Publish preserves a prior yank and first-publish time when artifact encoding or signer metadata changes without a package-content change. Registry merge excludes published_at from identity and keeps the receiving row's timestamp. Registry and federation package-version JSON encode published_at as an integer containing whole UTC Unix seconds from 0 through 253402300799. Unknown values are absent, not null.

The fix applies to all three local publishers. Rust SSH publishing moved to publish_ssh.rs so publish.rs remains below the source-file hard limit.

Claims ledger follows.

C-001. Claim: local publish replaces the matching version row and assigns the current time on every run. Outcome supported. Evidence: the pre-fix Rust and TypeScript regression tests failed with the new timestamp, and all three publish implementations constructed a current timestamp before rejecting and appending the matching version. Action: fixed and covered in all three clients.

C-002. Claim: older rows remain unchanged while the project package's current version moves. Outcome supported for the mechanism. Evidence: each implementation rejected only the row whose version matched the resolved project package. Action: no separate repair needed after the no-op check.

C-003. Claim: the reported run changed 29 files and the motivating release hid three real files among 27 timestamp-only files. Outcome unverifiable in this checkout. Source is the issue author's report; no distribution snapshot or pull request is linked. A one-package reproduction confirms the mechanism, not those counts. Action: retain the quantities as testimony in issue 26.

C-004. Claim: published_at is outside the signed payload. Outcome supported. Evidence: package_hash_signing_payload covers artifact_hash and tree_hash; the legacy digest covers artifact bytes, tree hash, and signing identity. Neither includes published_at. Action: RFC 0061 names published_at as unsigned catalog metadata.

C-005. Claim: a timestamp-only difference causes Rust registry merge to report a conflicting package version. Outcome supported for the pre-fix predicate and now covered through the HTTP publish path. The pre-fix RegistryPackageVersion test rejected rows that differed only in published_at. Action: remove published_at from same_identity and preserve the receiving timestamp.

C-006. Claim: republish clears a yank. Outcome supported. Evidence: each replacement constructor set yanked false. Action: a verified no-op keeps the complete row; a rebuilt matching version carries the existing yanked value. Only explicit yank undo clears it.

C-007. Claim: the distribution contract did not define first-write or last-write semantics for published_at. Outcome supported. RFC 0060 and registry.schema.json listed an optional non-empty string without lifecycle semantics. Action: RFC 0061 proposes the lifecycle and the schema now describes preservation.

C-008. Claim: clock formats differ across implementations and Rust federation timestamp selection parses unsigned integer strings only. Outcome supported before the fix. Rust wrote Unix seconds as a string, Ruby wrote ISO 8601, and TypeScript wrote an ISO string with milliseconds. Action applied: all three producers and federation package-version payloads now emit a bounded JSON integer or omit an unknown value. The schema rejects strings, numbers with non-zero fractional parts, negatives, out-of-range integers, and null. Readers normalize their prior output during migration. The schema regression first failed because the prior string schema rejected canonical integer zero, then passed after the type change.

## Effects

EA-001. Severity high. Confidence high. Location local publish app logic in crates/pray-cli/src/publish.rs, rubygems/pray-cli/lib/pray/publish.rb, and npmjs/pray-cli/src/publish/index.ts. Kind observed. Why it matters: a no-op release changed the version people resolve and obscured real catalog edits. Smallest credible fix applied: verify the existing expected artifact, tree, prayspec, and signing metadata, then keep the row and bytes. Changed package files or prayspec rebuild and receive a new timestamp. Changed signer metadata rebuilds the row but keeps the first-publish time.

EA-002. Severity high. Confidence high. Location RegistryPackageVersion::same_identity and the pray serve merge path. Kind observed. Why it matters: timestamp-only input was rejected as conflicting package identity. Smallest credible fix applied: exclude published_at from identity and keep the receiver's field. A focused predicate test and an authenticated repeated HTTP publish cover the behavior.

EA-003. Severity medium. Confidence high. Location version-row yank state in all three publishers. Kind observed. Why it matters: publish could reverse an explicit operator command. Smallest credible fix applied: preserve yanked for an existing version, including when a damaged artifact requires a rebuild.

EA-004. Severity medium. Confidence high. Location package_integrity signature payload and RFC 0050. Kind observed. Why it matters: a reader could assume every field beside signature is authenticated. Smallest credible fix applied for this issue: stop changing the unsigned field on a no-op and name its status in RFC 0061. Signing published_at remains a separate contract choice.

EA-005. Severity medium. Confidence high. Location publish tests. Kind observed. Missing coverage was repeated publish with unchanged and changed package content. Existing first-publish tests were not futile, but they could not detect timestamp or yank replacement. Smallest credible fix applied: regression tests cover unchanged metadata, preserved yank, and replacement after file or prayspec changes in Rust, Ruby, and TypeScript.

EA-006. Severity medium. Confidence high for archive bytes, medium for a live-server conflict. Location rubygems/pray-cli/lib/pray/archive.rb and npmjs/pray-cli/src/archive/praypkg.ts. Kind observed for non-deterministic bytes, inference for remote failure. Two builds of unchanged simple-project source separated by 1.1 seconds compared unequal in both clients because their system-tar staging files carry build-time metadata. A repeated remote publish can therefore change artifact_hash and still conflict after published_at leaves identity. Confirming check: publish twice from each client to one Rust pray serve instance. Smallest credible fix: emit deterministic archive headers in Ruby and TypeScript, matching the Rust archive builder. This is outside issue 26's local-root scope.

EA-007. Severity low. Confidence high. Location current_timestamp, Ruby and TypeScript publish constructors, federation package-version payloads, and latest_publish_timestamp. Kind observed. Why it matters: mixed catalogs did not share one sortable clock contract. Fix applied: RFC 0061, registry.schema.json, all producers, and all consumers use the same bounded integer seconds representation.

Resource and budget mode: before the fix, local publish rebuilt and wrote one archive and metadata row per project package. After the fix, a verified no-op reads, hashes, and decompresses the stored artifact, skips the artifact write, and serializes metadata and the registry index to equal bytes. The metadata write also migrates a legacy timestamp once. Ruby and TypeScript extract the bounded archive into a temporary directory to compare the prayspec; Rust compares it in memory. CPU time, storage writes, RSS, and energy or CPU-second proxy were not measured. Named next bench: a 29-package root with syscall counts and CPU time before and after.

Trace and identification mode: published_at combined with signer or signer_fingerprint records publisher activity in catalog JSON. The defect created a false common action time across every current version. No new identifier or telemetry was added. No packet capture or log-retention audit was run.

Boundary and control mode: the operator commands publish, the CLI writes catalog state, version control reports the diff, and consumers infer release history. Before the fix, intended no change diverged from commanded replacement and reported publish time without an alarm. The no-op integrity check now couples intended and stored state. Explicit yank remains the only command that changes yanked state.

Product surface mode: the relevant surface is the terminal plus version-control diff. A no-op local publish now leaves package files unchanged. No screen, pointer, keyboard, or accessibility surface applies.

Privacy mode: catalog rows already store signer labels, fingerprints, public keys, and timestamps. This change adds no person data and reduces false activity timestamps. Retention and deletion policy are outside this issue.

Performance mode: no elapsed-time benchmark was run, so a speedup claim would be inference. The code removes archive construction and writes from the verified no-op branch. The resource bench above would measure the effect.

Observability mode: the optional pray serve boundary returns publish failure to the CLI. The repeated HTTP publish regression test verifies success after a timestamp-only difference. Service logging and alerting are outside this issue.

Security mode: local metadata cannot redirect the no-op integrity read because the stored artifact path must equal the publisher's expected relative path. Stored bytes must hash to artifact_hash, pass bounded archive validation, and contain the current prayspec before publish skips work. No authentication or authorization policy changed.

Contract mode: RFC 0061 and registry.schema.json define preservation and the canonical timestamp representation. Signature coverage remains open. Learned-systems mode skipped because this path does not call a model, retriever, or tool-using agent; derived metadata is local executable logic.

Validation after the final change:

- cargo test -q -- --test-threads=1 passed the complete Rust workspace suite.
- cargo clippy --workspace --all-targets --all-features -- -D warnings passed.
- cargo fmt --all --check passed.
- npm test passed 166 tests with no failures.
- npm run typecheck passed.
- npm run lint passed Biome over 172 files and reported no circular dependencies.
- bundle exec rspec --format progress passed 218 examples with no failures under Ruby 3.4.7.
- bundle exec rubocop inspected 144 files with no offenses.
- bundle exec rbs validate passed.
- make loc-check reported 141 warnings and 0 failures; the changed Rust publish file is 268 lines.
- git diff --check passed.

The default parallel Rust suite also ran during validation. One run passed. A later run reported four update_destinations failures with shared temporary git fixture paths, including File exists and missing-directory errors. cargo test -p pray-cli --test update_destinations -- --test-threads=1 then passed all six examples, followed by the complete serial workspace pass above. No publish failure was reproduced from that result.

The Ruby Makefile's parallel test target also ran during validation. Its linters passed, but one provisioned-destination example collided with a temporary git-distribution fixture from another concurrently running example. The same example passed alone, and the complete serial RSpec run above passed. No product failure was reproduced from that result.

Later pass 20260914140500: this work ships as 1.14.0. See usr/docs/issues/20260914140500_prepare-1-14-0-release.md.

## Next

Review RFC 0061 through 2026-09-28.

Make Ruby and TypeScript archive output deterministic, then run the two-client repeated publish check against pray serve.

Decide whether catalog timestamps belong in a future signed payload. Do not change the current signature bytes under this issue.

## Source

https://github.com/kiskolabs/pray/issues/26

https://json-schema.org/draft/2020-12/json-schema-validation

crates/pray-cli/src/publish.rs

crates/pray-cli/src/publish_ssh.rs

crates/pray-cli/src/server_registry.rs

crates/pray-core/src/registry.rs

crates/pray-core/src/package_integrity.rs

rubygems/pray-cli/lib/pray/publish.rb

rubygems/pray-cli/lib/pray/archive.rb

npmjs/pray-cli/src/publish/index.ts

npmjs/pray-cli/src/archive/praypkg.ts

rfcs/0060-distribution.md

rfcs/0061-publish-version-preservation.md

schema/registry.schema.json

usr/docs/issues/20260914140500_prepare-1-14-0-release.md

usr/docs/changelogs/20260914142000_tag-1-14-0.md

CHANGELOG.md 1.14.0
