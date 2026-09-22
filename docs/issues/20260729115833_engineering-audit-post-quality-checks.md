# Engineering audit after quality checks program

## Participants

- Andrei Makarov

## Decisions

- Scope is the pray reference CLI on main after the quality-checks merge: parse, resolve, fetch, trust, render, serve, sync, and quality gates.
- Rank by danger, certainty, impact, and fix cost. Prefer the smallest credible fix before structural rewrite.
- Separate missing coverage from futile coverage. Label inferences with the check needed to confirm.

## Effects

Audit completed as static review plus targeted code confirmation. Highest-risk findings:

1. Serve HTTP ingress allocates request bodies from Content-Length with no cap and spawns one thread per connection with no socket deadline (crates/pray-cli/src/server.rs handle_connection, run_server). Public serve can exhaust memory and threads.
2. Help documents serve --allow-open-push, but Command::Serve and parse_serve_command omit the flag and serve_command always passes false (crates/pray-cli/src/help.rs, cli_parse.rs, main.rs). Documented authorization mode is unreachable.
3. Federation sync removes peers from the in-flight set when processed, then requeues rediscovered peers with no permanent visited set or peer/depth/byte caps (crates/pray-cli/src/sync_command.rs synchronize_registry). Cyclic peer graphs can loop indefinitely.
4. Sync accepts blank tree_hash as None and runs signature verification only when tree_hash is present (sync_command.rs sync_package_version_from_transport). Artifact hash alone is enough to store federation packages.
5. Render joins project_root with target.path and file destinations without rejecting absolute or parent paths (crates/pray-core/src/render.rs). Package archive path safety does not cover render egress.
6. Resolver resolves only manifest.packages; dependency_graph ignores edges to undeclared names (resolve.rs, dependency_graph.rs). SPEC transitive resolve is not implemented; undeclared deps are silently skipped.
7. RenderPolicy conflict/churn/mode/section_markers/line_endings and target max_bytes parse but do not gate writes; only render.header is read (manifest.rs, render.rs). install_validation conflict tests are ignored.
8. Torrent fetch allocates vec![0; manifest.length] from untrusted length before full integrity bounds (registry_torrent.rs; transport torrent.rs similarly). Archive traversal is covered; decompression and size quotas are not.
9. SSH publisher allowlist gates the consumer active_ssh_user_fingerprint before opening a session, not package signer_fingerprint (ssh_client.rs, client_trust/ssh_host.rs). HTTP registry install does not apply allowed_publishers to downloaded metadata.
10. CI mutation smoke is continue-on-error and uses cargo-mutants -e on manifest.rs, package_spec.rs, dependency_graph.rs, package_integrity.rs, package_archive.rs, and paths.rs, which excludes those files from mutation. Job title claims parser and integrity coverage. llvm-cov floor is 20 percent workspace-wide.
11. PrayError has no network/fetch variant; SPEC exit code 7 is unreachable. Network failure tests accept exit 1 or 3.
12. Duplicate HTTP and torrent stacks diverge on timeouts and response limits (pray-core registry_http/registry_torrent versus pray-transport http/torrent versus CLI ad hoc fetches).
13. Several source files exceed the 300 LOC hard ceiling, including pray-cli main.rs (~2007), manifest.rs (~1148), auth.rs (~1031), transport torrent.rs (~683), render.rs (~595).
14. fixtures/ has only minimal parser and prayspec packs; SPEC section 73 resolver/render/verify/registry packs are not present. cargo-fuzz is local-only; CI proptest discards parse results for crash-only coverage.

Missing coverage (contracts absent or thin): render destination path safety; multi-output targets; transitive resolve and constraint merge; archive/torrent resource quotas; publisher allowlist on artifact metadata; federation peer caps and visited set; exit code 7; serve body/connection limits; allow-open-push CLI wiring.

Futile or non-protective coverage: ignored conflict/patch and scaling tests; mutation job excluding the named integrity surfaces; default-feature transport tests whose assertions sit behind feature cfg; network tests that lock in wrong exit codes; conformance fixtures asserting only a few parse fields.

## Next

- Remaining after resolve/transport/patch follow-ups: nightly fuzz, blocking mutants baseline.
- See usr/docs/issues/20260729121659_deep-safety-fixes.md and usr/docs/issues/20260729122836_audit-followups-resolve-transport-patch.md for what already landed.

## Source

- Quality checks issue: usr/docs/issues/20260729114148_quality-checks-program.md
- Quality checks changelog: usr/docs/changelogs/20260729114148_quality-checks-program.md
- Production readiness checklist: usr/docs/issues/20260629153000_production_readiness_checklist.md
- Normative contracts: SPEC.md sections on resolve, render paths, federation integrity, exit codes, conformance
- Code: crates/pray-core, crates/pray-cli, crates/pray-transport, .github/workflows/ci.yml, mutants.toml, fixtures/, fuzz/
