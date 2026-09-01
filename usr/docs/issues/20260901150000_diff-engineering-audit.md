# Diff engineering audit after remediation

## Participants

Andrei Makarov

## Decisions

Audit radius is the uncommitted working tree on patch/fix-ruby-praypkg-unpack-cache versus HEAD 9bd41b4. That tree is the 1.9.2 unpack and cache work plus the EA-001 through EA-008 smallest-fix remediation.

Prior notes claimed those eight items closed. This pass re-read the current Rust, TypeScript, and Ruby sources rather than the remediation changelog.

Learned-systems mode skipped: these trees package inference input and do not execute a model, retriever, or tool-using agent.

No behavior change from this note alone.

## Effects

Account-takeover via a registration response that returned the verification code, plus unauthenticated HTTP enrollment, is closed on the HTTP and RPC surfaces. Session files moved to the user Pray home with owner-only mode. Remote artifact and tree hashes fail closed. Project and federation paths are contained at parse. Serve body, header, connection, and timeout ceilings exist in all three servers.

Residuals that remain after that scope are ranked below.

D-001. Severity high. Confidence high. Location crates/pray-core/src/auth_store_secrets.rs generate_verification_code, crates/pray-core/src/auth_store.rs verify_email, crates/pray-cli/src/server_auth.rs auth_verify_response. Kind observed. Pipeline ingress and authentication. Why it matters: public verify still accepts a six-digit code stored in plaintext, with a 15 minute lifetime, distinct missing versus mismatch errors, and no attempt ceiling. One million values fit inside the TTL on an unthrottled listener. HTTP no longer returns the code. Smallest credible fix: store a hash of a high-entropy secret, compare in constant time, return one error shape, and cap attempts per email. Deliver codes out of band or drop email-code authentication.

D-002. Severity high. Confidence high. Location npmjs/pray-cli/src/http/client.ts httpGet, rubygems/pray-cli/lib/pray/registry.rb http_get. Kind observed. Pipeline external API and resource. Why it matters: TypeScript buffers the entire arrayBuffer; Ruby keeps the full Net::HTTP body. Rust already stops at MAX_HTTP_RESPONSE_BYTES. This diff makes remote install a first-class path in the other two clients, so a hostile or oversized registry response can exhaust process memory. Smallest credible fix: cap the read at the existing 64 MiB constant, reject a larger Content-Length, and add one oversized-body test per client. Confirming bench for RSS: fetch a 128 MiB body on a named CI machine.

D-003. Severity high. Confidence medium. Location rubygems/pray-cli/lib/pray/archive_unpack.rb, rubygems/pray-cli/lib/pray/tar_validation.rb, npmjs/pray-cli/src/archive/praypkg.ts, npmjs/pray-cli/src/archive/tar.ts. Kind inference for exploit, observed for architecture. Pipeline unpack. Why it matters: a custom ustar walk with no checksum then shells out to tar -xf on the original bytes. The post-extract tree walk cannot see writes outside the output directory. Rust unpacks entry by entry in package_archive.rs. Confirming check: a member with a bad checksum that system tar skips, or a GNU sparse member, extracted on macOS bsdtar and GNU tar. Smallest credible fix: extract only validated member names, or copy the Rust entry loop; verify ustar checksums; reject sparse; share one hostile-archive fixture across the three clients.

D-004. Severity medium. Confidence high. Location crates/pray-cli/src/server_auth.rs auth_session_response, auth_passkey_enroll_response, auth_ssh_key_enroll_response. Kind observed. Pipeline auth. Why it matters: those handlers parse the body and always return 403. Verification codes never leave SQLite. There is no verified-session enrollment path. Operators will reopen the old API. Smallest credible fix: keep the 403 until out-of-band verify and bearer-gated enroll exist. Do not restore email-only session issue.

D-005. Severity medium. Confidence high. Location npmjs/pray-cli/src/registry/install.ts validateAndUnpack versus rubygems/pray-cli/lib/pray/registry.rb verify_registry_signature! and crates/pray-core/src/package_integrity.rs verify_package_signature. Kind observed. Pipeline integrity. Why it matters: when a signature is present, Ruby and Rust verify it; TypeScript still installs on hashes alone, so a wrong signature is accepted. Smallest credible fix: verify-if-present before unpack, plus a shared signature-mismatch fixture. TypeScript also skips publisher fingerprint checks that Ruby runs when source_url is set.

D-006. Severity medium. Confidence high. Location npmjs/pray-cli/src/registry/index.ts readArtifactBytes, npmjs/pray-cli/src/http/client.ts joinUrl, rubygems/pray-cli/lib/pray/registry.rb join_url. Kind observed. Pipeline external API. Why it matters: an absolute http(s) artifact field is fetched as a new origin. Hash binds bytes after the request. The request itself is still server-side fetch to an attacker-chosen URL. Smallest credible fix: allow only relative artifact paths under the source origin; reject absolute http(s) and file:// in remote metadata.

D-007. Severity medium. Confidence high. Location npmjs/pray-cli/src/registry/index.ts fetchPackageMetadata. Kind observed. Pipeline ingress. Why it matters: package names are interpolated into v1/packages/${name}.json. Ruby calls reject_unsafe_package_name! first. TypeScript sync validates names; resolveRegistryPackageRoot does not. Manifest path validation does not cover package names. Smallest credible fix: validatePackageName on every metadata URL and local join.

D-008. Severity medium. Confidence high. Location crates/pray-core/src/auth_store_keys.rs login_with_passkey, login_with_ssh_key, enroll_passkey, enroll_ssh_key. Kind observed. Pipeline auth library. Why it matters: the store can issue a session from a credential id with no signature. HTTP is closed; tests and any future RPC can still call the shortcut. Enroll on conflict also overwrites email for an existing credential id. Smallest credible fix: confine proof-less helpers to tests; require an authenticated caller for enroll even in-process.

D-009. Severity medium. Confidence medium. Location auth_store_keys.rs respond_passkey_challenge and respond_ssh_key_challenge. Kind observed race. Pipeline auth. Why it matters: load, verify, mark used, and issue session use separate SQLite connections. Two in-flight logins can both see used_at IS NULL. Smallest credible fix: one transaction that selects the unused challenge, verifies, sets used_at, and inserts the session; set a busy timeout.

D-010. Severity medium. Confidence high. Location npmjs/pray-cli/src/serve/index.ts catch in runServer. Kind observed. Pipeline egress. Why it matters: 500 responses send error.message to the client. Smallest credible fix: fixed 500 body; keep the message on stderr next to request_id.

D-011. Severity low. Confidence high. Location crates/pray-core/src/auth_store.rs open, rubygems/pray-cli/lib/pray/serve.rb GET /. Kind observed. Pipeline store and product surface. Why it matters: session.json is 0600; auth.db and .pray/ follow umask. Ruby index HTML prints the filesystem root. Smallest credible fix: 0700 on .pray, 0600 on auth.db after create; drop the root path from the index body.

D-012. Severity low. Confidence high. Location crates/pray-cli/src/server_http.rs http_to_rpc_request. Kind observed. Pipeline observability. Why it matters: every Rust HTTP RPC id is the literal http. TypeScript mints a UUID. Smallest credible fix: mint a per-request id in handle_connection and pass it through.

Missing coverage, not futile: TypeScript unpack has no parent-escape example (Ruby does); no shared symlink, sparse, or checksum-differential tar fixture; existing-email register does not reset the code and is untested; no verify attempt-ceiling test; no TypeScript signature-mismatch install test; no oversized HTTP body test on TypeScript or Ruby clients.

Thin coverage, not futile: HTTP register only asserts the verification_code key is absent; enroll tests assert 403 while RegistryAuthStore enroll stays public; cache_ready? still rescues every Error, so a poison artifact can refetch on every install.

Accepted residual: loopback HTTP PUT without a bearer token, matching Rust authorize_distribution_push for 127.0.0.1.

Resource and budget: named archive 64 MiB, entry 32 MiB, serve body 16 MiB, 32 connections, 30 second sockets. TypeScript and Ruby client fetch have no cap. Unpack still buffers full zstd then full tar in those two clients. Peak RSS, CPU-seconds, disk, network bytes, and energy were not measured this run. Cheaper, smaller, or greener stays inference.

Trace and identification: no hidden analytics path found. session.json under the user Pray home holds live bearer token, email, server URL, and kind. auth.db holds email and plaintext verification codes. No delete or export path. Not verified with HAR or packet capture.

Boundary and control: zstd and tar are commanded processes. Encoding.default_internal was a wrong-unit fault at stdin and is now cleared around those calls. Staging rename reduces reported versus physical cache divergence after a failed unpack. The remaining split is commanded tar versus the custom validator's view of the same bytes.

## Next

D-001 through D-003 and D-005 through D-012 are implemented on this branch. D-004 stays: keep 403 on email-only session issue and unauthenticated enroll until out-of-band verify delivery exists.

Remaining residuals: Ruby trust feed GET still buffers the full body; unpack still expands full zstd then tar in TypeScript and Ruby; no shared sparse or symlink hostile-archive fixture across the three clients.

## Source

Working tree on patch/fix-ruby-praypkg-unpack-cache.

Prior notes: usr/docs/issues/20260901133800_audit-ruby-praypkg-unpack-cache.md, usr/docs/issues/20260901135528_engineering-audit-auth-integrity-parity.md, usr/docs/changelogs/20260901143628_engineering-audit-remediation.md.

Code read this run includes crates/pray-cli/src/server_auth.rs, crates/pray-core/src/auth_store.rs, crates/pray-core/src/auth_store_secrets.rs, crates/pray-core/src/auth_store_keys.rs, crates/pray-cli/src/auth_session_store.rs, crates/pray-core/src/package_archive.rs, rubygems/pray-cli/lib/pray/archive_unpack.rb, rubygems/pray-cli/lib/pray/tar_validation.rb, npmjs/pray-cli/src/archive/praypkg.ts, npmjs/pray-cli/src/registry/install.ts, npmjs/pray-cli/src/http/client.ts, npmjs/pray-cli/src/serve/index.ts.

Validation commands were not re-run in this audit pass. Earlier remediation note recorded cargo test, npm test:coverage, Ruby parallel-rspec with coverage, loc-check, lint, and cargo deny as passing on 2026-09-01. Those results are not re-claimed here.
